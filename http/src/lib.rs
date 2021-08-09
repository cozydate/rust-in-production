use std::cmp::min;
use std::convert::TryFrom;
use std::pin::Pin;
use std::task::{Context, Poll};

use lazy_static::lazy_static;
use log::trace;
use string_wrapper::StringWrapper;
use tokio::io::AsyncWrite;
use tokio::prelude::AsyncRead;

use crate::fixed_buffer::FixedBuf;

pub mod buffer;
pub mod async_write_logger;
pub mod async_readable;
pub mod split_iterate;
pub mod async_write_buffer;
pub mod fixed_buffer;

pub fn escape_ascii(input: &[u8]) -> String {
    let mut result = String::new();
    for byte in input {
        for ascii_byte in std::ascii::escape_default(*byte) {
            result.push_str(std::str::from_utf8(&[ascii_byte]).unwrap());
        }
    }
    result
}

// impl std::fmt::Debug for HttpSession {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // https://doc.rust-lang.org/std/fmt/struct.Formatter.html
//         f.debug_struct("HttpSession")
//             .field("addr", &self.addr)
//             .finish()
//     }
// }

// async fn handle_tcp_stream(tcp_stream: TcpStream, addr: SocketAddr, handler: Arc<dyn HttpSessionHandler + Send + Sync>) {
//     let logger = slog_scope::logger().new(
//         slog::o!("request_id" => random_id(8), "ip" => addr.ip().to_string()));
//     slog_scope_futures::SlogScope::new(&logger, async move {
//         logging::info!("Handling request");
//         if let Err(e) = tcp_stream.set_keepalive(Some(Duration::from_secs(60))) {
//             warn!("Failed setting keepalive on tcp socket: {:?}", e);
//         }
//         let mut http_session = HttpSession { tcp_stream, addr };
//         handler.handle(&mut http_session).await;
//     }).await;
// }

// async fn accept_loop(mut listener: tokio::net::TcpListener, handler: Arc<dyn HttpSessionHandler + Send + Sync>) {
//     info!("Starting accept loop");
//     loop {
//         match listener.accept().await {
//             Ok((tcp_stream, addr)) => {
//                 let handler_clone = handler.clone();
//                 tokio::spawn(async move {
//                     handle_tcp_stream(tcp_stream, addr, handler_clone).await;
//                 });
//             }
//             Err(e) => {
//                 warn!("Failed accepting connection from socket: {:?}", e);
//                 match e.kind() {
//                     // Do not sleep on connection error.
//                     std::io::ErrorKind::ConnectionAborted
//                     | std::io::ErrorKind::ConnectionRefused
//                     | std::io::ErrorKind::ConnectionReset => {}
//                     // Sleep on accept error.
//                     _ => {
//                         tokio::time::delay_for(Duration::from_secs(1)).await;
//                     }
//                 }
//             }
//         }
//     }
// }


#[derive(Debug)]
pub enum HttpError {
    IoError(std::io::Error),
    ParseError(HttpCallerError),
    ProcessingError(HttpStatus),
}

impl HttpError {
    pub fn from_io_err(e: std::io::Error) -> HttpError {
        HttpError::IoError(e)
    }
}

#[derive(Debug)]
pub enum HttpCallerError {
    RequestLineMissing,
    RequestLineInvalid,
    MethodInvalid,
    MethodTooLong,
    PathInvalid,
    PathTooLong,
    HeaderLineInvalid,
    HeaderValueTooLong,
    ExpectHeaderInvalid,
    TransferEncodingHeaderInvalid,
    ContentLengthHeaderInvalid,
}

impl HttpCallerError {
    pub fn status(&self) -> HttpStatus {
        match self {
            Self::RequestLineMissing => HttpStatus::BadRequest400,
            Self::RequestLineInvalid => HttpStatus::BadRequest400,
            Self::MethodInvalid => HttpStatus::MethodNotAllowed405,
            Self::MethodTooLong => HttpStatus::MethodNotAllowed405,
            Self::PathInvalid => HttpStatus::NotFound404,
            Self::PathTooLong => HttpStatus::UriTooLong414,
            Self::HeaderLineInvalid => HttpStatus::BadRequest400,
            Self::HeaderValueTooLong => HttpStatus::RequestHeaderFieldsTooLarge431,
            Self::ExpectHeaderInvalid => HttpStatus::BadRequest400,
            Self::TransferEncodingHeaderInvalid => HttpStatus::BadRequest400,
            Self::ContentLengthHeaderInvalid => HttpStatus::BadRequest400,
        }
    }
}

/// An upper-case string starting with an English letter
/// and containing only English letters and digits.
/// Length is 1-16 bytes.
///
/// Examples: "GET", "HEAD", "CUSTOM123"
#[derive(Clone, Debug, PartialEq)]
pub enum HttpMethod {
    DELETE,
    GET,
    HEAD,
    POST,
    PUT,
    Other(string_wrapper::StringWrapper<[u8; 16]>),
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Result<HttpMethod, HttpError> {
        // HTTP/1.1 Request Methods https://tools.ietf.org/html/rfc7231#section-4
        lazy_static! {
            static ref METHOD_RE: regex::Regex = regex::Regex::new("^[A-Z][A-Z0-9]*$").unwrap();
        }
        if !METHOD_RE.is_match(s) {
            return Err(HttpError::ParseError(HttpCallerError::MethodInvalid));
        }
        match s {
            "DELETE" => Ok(HttpMethod::DELETE),
            "GET" => Ok(HttpMethod::GET),
            "HEAD" => Ok(HttpMethod::HEAD),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            s => StringWrapper::from_str_safe(s)
                .map(|sw| HttpMethod::Other(sw))
                .ok_or(HttpError::ParseError(HttpCallerError::MethodTooLong)),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::DELETE => "DELETE",
            HttpMethod::GET => "GET",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::Other(sw) => &sw,
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, PartialEq)]
pub struct HttpRequestLine<'a> {
    method: &'a str,
    raw_path: &'a str,
}

impl<'a> HttpRequestLine<'a> {
    pub fn parse(line_bytes: &[u8]) -> Result<HttpRequestLine, HttpError> {
        // HTTP/1.1 Request Line https://tools.ietf.org/html/rfc7230#section-3.1.1
        let line = std::str::from_utf8(line_bytes)
            .map_err(|_| HttpError::ParseError(HttpCallerError::RequestLineInvalid))?;
        lazy_static! {
            static ref REQUEST_LINE_RE: regex::Regex =
                regex::Regex::new("^([^ ]+) (/[^ ]*) HTTP/1.1$").unwrap();
        }
        let captures: regex::Captures = REQUEST_LINE_RE.captures(line)
            .ok_or(HttpError::ParseError(HttpCallerError::RequestLineInvalid))?;
        let method = captures.get(1).unwrap().as_str();
        let raw_path = captures.get(2).unwrap().as_str();
        Ok(HttpRequestLine { method, raw_path })
    }
}

pub struct Header<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> Header<'a> {
    pub fn new<'b>(name: &'b str, value: &'b str) -> Header<'b> {
        Header { name, value }
    }
}

/// Returns true if the header name matches headers that cannot carry PII:
/// `transfer-encoding`, `content-length`, `content-type`, and `content-encoding`.
pub fn is_non_pii_header(name: &str) -> bool {
    name.eq_ignore_ascii_case("transfer-encoding") ||
        name.eq_ignore_ascii_case("content-length") ||
        name.eq_ignore_ascii_case("content-type") ||
        name.eq_ignore_ascii_case("content-encoding")
}

impl<'a> std::fmt::Display for Header<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if is_non_pii_header(self.name) {
            write!(f, "{}:{}}}", self.name.to_ascii_lowercase(), self.value)
        } else {
            write!(f, "{}:<{} bytes>}}", self.name.to_ascii_lowercase(), self.value.len())
        }
    }
}

impl<'a> std::fmt::Debug for Header<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Header{{{}:{}}}", self.name.to_ascii_lowercase(), self.value)
    }
}

pub fn headers_to_string(headers: &[&Header]) -> String {
    let header_strings: Vec<String> = headers.iter().map(|&h| h.to_string()).collect();
    "[".to_string() + &header_strings.join(", ") + "]"
}

pub struct HeaderReceiver<'a> {
    pub name: &'a str,
    pub value: StringWrapper<[u8; 256]>,
}

impl<'a> HeaderReceiver<'a> {
    pub fn new(name: &str) -> HeaderReceiver {
        HeaderReceiver { name, value: StringWrapper::from_str("") }
    }

    pub fn is_chunked(&self) -> Result<bool, HttpError> {
        if !self.name.eq_ignore_ascii_case("transfer-encoding") {
            panic!("is_chunked() called on {:?} header", self.name);
        }
        if self.value.is_empty() {
            return Ok(false);
        }
        if self.value.eq_ignore_ascii_case("chunked") {
            return Ok(true);
        }
        Err(HttpError::ParseError(HttpCallerError::TransferEncodingHeaderInvalid))
    }

    pub fn is_100_continue(&self) -> Result<bool, HttpError> {
        // HTTP/1.1 Expect https://tools.ietf.org/html/rfc7231#section-5.1.1
        if !self.name.eq_ignore_ascii_case("expect") {
            panic!("is_100_continue() called on {:?} header", self.name);
        }
        if self.value.is_empty() {
            return Ok(false);
        }
        if self.value.eq_ignore_ascii_case("100-continue") {
            return Ok(true);
        }
        Err(HttpError::ParseError(HttpCallerError::ExpectHeaderInvalid))
    }

    pub fn parse_content_length(&self) -> Result<u64, HttpError> {
        if !self.name.eq_ignore_ascii_case("content-length") {
            panic!("parse_content_length() called on {:?} header", self.name);
        }
        if self.value.is_empty() {
            return Ok(0);
        }
        let content_length: u64 = std::str::FromStr::from_str(&self.value)
            .map_err(|_e| HttpError::ParseError(HttpCallerError::ContentLengthHeaderInvalid))?;
        Ok(content_length)
    }
}

impl<'a> std::fmt::Display for HeaderReceiver<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if is_non_pii_header(self.name) {
            write!(f, "{}:{}}}", self.name.to_ascii_lowercase(), self.value)
        } else {
            write!(f, "{}:<{} bytes>}}", self.name.to_ascii_lowercase(), self.value.len())
        }
    }
}

impl<'a> std::fmt::Debug for HeaderReceiver<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HeaderReceiver{{{}:{}}}", self.name.to_ascii_lowercase(), self.value)
    }
}

pub fn header_receivers_to_string(header_receivers: &[&HeaderReceiver]) -> String {
    let header_strings: Vec<String> =
        header_receivers.iter().map(|&hr| hr.to_string()).collect();
    "[".to_string() + &header_strings.join(", ") + "]"
}

#[derive(Debug, Clone)]
pub enum HttpStatus {
    Continue100,
    Ok200,
    Created201,
    BadRequest400,
    NotFound404,
    MethodNotAllowed405,
    LengthRequired411,
    PayloadTooLarge413,
    UriTooLong414,
    RequestHeaderFieldsTooLarge431,
    InternalServerError500(String),
}

impl HttpStatus {
    pub fn as_line(&self) -> &'static str {
        match self {
            HttpStatus::Continue100 => "HTTP/1.1 100 Continue\r\n",
            HttpStatus::Ok200 => "HTTP/1.1 200 OK\r\n",
            HttpStatus::Created201 => "HTTP/1.1 201 Created\r\n",
            HttpStatus::BadRequest400 => "HTTP/1.1 400 Bad Request\r\n",
            HttpStatus::NotFound404 => "HTTP/1.1 404 Not Found\r\n",
            HttpStatus::MethodNotAllowed405 => "HTTP/1.1 405 Method Not Allowed\r\n",
            HttpStatus::LengthRequired411 => "HTTP/1.1 411 Length Required\r\n",
            HttpStatus::PayloadTooLarge413 => "HTTP/1.1 413 Payload Too Large\r\n",
            HttpStatus::UriTooLong414 => "HTTP/1.1 414 URI Too Long\r\n",
            HttpStatus::RequestHeaderFieldsTooLarge431 =>
                "HTTP/1.1 431 Request Header Fields Too Large\r\n",
            HttpStatus::InternalServerError500(_) => "HTTP/1.1 500 Internal Server Error\r\n",
        }
    }
}

pub struct HttpReaderWriter<'a> {
    addr: std::net::SocketAddr,
    input: Pin<&'a mut (dyn tokio::io::AsyncRead + std::marker::Send + std::marker::Unpin)>,
    buffer: FixedBuf,
    method: Option<HttpMethod>,
    pub raw_path: StringWrapper<[u8; 512]>,
    unsent_expect_100_bytes: &'static [u8],
    content_length: u64,
    unread_content_length: u64,
    chunked: bool,
    output: Pin<&'a mut (dyn tokio::io::AsyncWrite + std::marker::Send + std::marker::Unpin)>,
    status: Option<HttpStatus>,
    unsent_content_length: Option<u64>,
    bytes_written: u64,
}

impl<'a> HttpReaderWriter<'a> {
    pub fn new(
        input: Pin<&'a mut (dyn tokio::io::AsyncRead + std::marker::Send + std::marker::Unpin)>,
        output: Pin<&'a mut (dyn tokio::io::AsyncWrite + std::marker::Send + std::marker::Unpin)>,
        addr: std::net::SocketAddr)
        -> HttpReaderWriter<'a> {
        HttpReaderWriter {
            addr,
            input,
            buffer: FixedBuf::new(),
            method: None,
            raw_path: StringWrapper::from_str(""),
            unsent_expect_100_bytes: &[],
            content_length: 0,
            unread_content_length: 0,
            chunked: false,
            output,
            status: None,
            unsent_content_length: Some(0),
            bytes_written: 0,
        }
    }

    pub fn has_body(&self) -> bool {
        // The presence of a message body in a request is signaled by a Content-Length or
        // Transfer-Encoding header field.
        self.chunked || self.content_length > 0
    }

    pub fn content_length(&self) -> u64 { self.content_length }

    pub fn content_length_usize(&self) -> Result<usize, HttpError> {
        usize::try_from(self.content_length)
            .or(Err(HttpError::ProcessingError(HttpStatus::PayloadTooLarge413)))
    }

    pub fn method(&self) -> HttpMethod { self.method.as_ref().unwrap().clone() }

    pub fn decode_path(&self) -> Result<std::borrow::Cow<str>, HttpError> {
        if self.raw_path.is_empty() {
            panic!("HttpReaderWriter::decode_path alled before reading request");
        }
        Ok(percent_encoding::percent_decode_str(&self.raw_path)
            .decode_utf8()
            .map_err(|_e| HttpError::ParseError(HttpCallerError::PathInvalid))?)
    }

    fn save_header_value(name: &str, value: &str, headers: &mut [&mut HeaderReceiver])
                         -> Result<(), HttpError> {
        // For-loops call .iter() and cannot mutate the returned reference:
        // ```
        // for header in headers {...}  // error[E0382]: use of moved value: `headers`
        // ```
        // So we use .iter_mut() instead:
        if let Some(header) = headers.iter_mut()
            .filter(|header| name.eq_ignore_ascii_case(header.name))
            .next() {
            header.value.truncate(0);
            header.value.push_partial_str(value)
                .or(Err(HttpError::ParseError(HttpCallerError::HeaderValueTooLong)))?;
        }
        Ok(())
    }

    pub async fn read_request<'b>(&'b mut self, extra_headers: &'b mut [&mut HeaderReceiver<'b>])
                                  -> Result<(), HttpError> {
        if self.unread_content_length > 0 {
            return Err(HttpError::ProcessingError(HttpStatus::InternalServerError500(
                String::from("previous request body not completely read")
            )));
        }
        if let Some(unsent) = self.unsent_content_length {
            if unsent > 0 {
                return Err(HttpError::ProcessingError(HttpStatus::InternalServerError500(
                    String::from("previous response body not completely sent")
                )));
            }
        }
        self.buffer.shift();
        self.method = None;
        self.raw_path.truncate(0);
        self.unsent_expect_100_bytes = &[];
        self.content_length = 0;
        self.unread_content_length = 0;
        self.chunked = false;
        self.status = None;
        self.unsent_content_length = None;
        self.bytes_written = 0;

        // "HTTP/1.1 Message Syntax and Routing" https://tools.ietf.org/html/rfc7230
        let head = self.buffer.read_delimited(&mut self.input, b"\r\n\r\n")
            .await
            .map_err(|e| HttpError::IoError(e))
            ?;
        trace!("{:?} parsing HTTP request head {:?}", self.addr, escape_ascii(head));
        let mut lines = split_iterate::split_iterate(head, b"\r\n");

        let line_bytes = lines.next()
            .ok_or(HttpError::ParseError(HttpCallerError::RequestLineMissing))?;
        let request_line = HttpRequestLine::parse(line_bytes)?;

        let mut content_length = HeaderReceiver::new("content-length");
        let mut expect = HeaderReceiver::new("expect");
        let mut transfer_encoding = HeaderReceiver::new("transfer-encoding");
        for line_bytes in lines {
            if line_bytes.is_empty() {
                break;
            }
            // HTTP/1.1 Header Fields https://tools.ietf.org/html/rfc7230#section-3.2
            let line = std::str::from_utf8(line_bytes)
                .or(Err(HttpError::ParseError(HttpCallerError::HeaderLineInvalid)))?;
            lazy_static! {
                static ref LINE_RE: regex::Regex
                    = regex::Regex::new("^([^:]+):\\s*(.*?)\\s*$").unwrap();
            }
            let captures: regex::Captures = LINE_RE.captures(line)
                .ok_or(HttpError::ParseError(HttpCallerError::HeaderLineInvalid))?;
            let name = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str();

            Self::save_header_value(
                name, value,
                &mut [&mut content_length, &mut expect, &mut transfer_encoding], )?;
            Self::save_header_value(name, value, extra_headers)?;
        }

        self.method = Some(HttpMethod::from_str(request_line.method)?);
        self.raw_path.truncate(0);
        self.raw_path.push_partial_str(request_line.raw_path)
            .or(Err(HttpError::ParseError(HttpCallerError::PathTooLong)))?;
        if expect.is_100_continue()? {
            self.unsent_expect_100_bytes = HttpStatus::Continue100.as_line().as_bytes();
        }
        self.content_length = content_length.parse_content_length()?;
        self.chunked = transfer_encoding.is_chunked()?;
        Ok(())
    }

    fn append_content_length(mut buf: &mut FixedBuf, len: u64) -> Result<(), HttpError> {
        buf.append("content-length: ");
        itoa::write(&mut buf, len).unwrap();  // Write num without allocating.
        buf.append("\r\n");
        Ok(())
    }

    fn reject_header(name: &str, headers: &[&Header]) -> Result<(), HttpError> {
        for &header in headers {
            if header.name.eq_ignore_ascii_case(name) {
                return Err(HttpError::ProcessingError(HttpStatus::InternalServerError500(
                    String::from(format!("invalid extra header {:?}", header)))));
            }
        }
        Ok(())
    }

    fn push_header_internal(buf: &mut FixedBuf, header: &Header) -> Option<()> {
        buf.try_append(header.name)?;
        buf.try_append(": ")?;
        buf.try_append(header.value)?;
        buf.try_append("\r\n")
    }

    fn append_extra_headers(buf: &mut FixedBuf, headers: &[&Header]) -> Result<(), HttpError> {
        for &header in headers {
            Self::push_header_internal(buf, header)
                .ok_or_else(|| HttpError::ProcessingError(HttpStatus::InternalServerError500(
                    String::from("buffer full while pushing extra headers ".to_string() +
                        &headers_to_string(headers)))))?;
        }
        Ok(())
    }

    async fn send(&mut self, data: &[u8]) -> Result<(), HttpError> {
        trace!("{:?} sending {:?}", self.addr, escape_ascii(data));
        tokio::io::AsyncWriteExt::write_all(&mut self.output, data)
            .await
            .map_err(HttpError::from_io_err)?;
        self.bytes_written += u64::try_from(data.len()).unwrap();
        tokio::io::AsyncWriteExt::flush(&mut self.output)
            .await
            .map_err(HttpError::from_io_err)?;
        Ok(())
    }

    pub async fn send_simple(&mut self, status: HttpStatus) -> Result<(), HttpError> {
        let mut buf = fixed_buffer::FixedBuf::new();
        buf.append(status.as_line());
        buf.append("content-length: 0\r\n\r\n");
        self.unsent_content_length = Some(0);
        self.send(buf.read_all()).await?;
        self.status = Some(status);
        Ok(())
    }

    pub async fn send_without_body(&mut self, status: HttpStatus, extra_headers: &[&Header<'_>])
                                   -> Result<(), HttpError> {
        let mut buf = fixed_buffer::FixedBuf::new();
        buf.append(status.as_line());
        Self::append_content_length(&mut buf, 0)?;
        self.unsent_content_length = Some(0);
        Self::reject_header("transfer-encoding", extra_headers)?;
        Self::reject_header("content-length", extra_headers)?;
        Self::reject_header("content-type", extra_headers)?;
        Self::reject_header("content-encoding", extra_headers)?;
        Self::append_extra_headers(&mut buf, extra_headers)?;
        buf.append("\r\n");
        self.send(buf.read_all()).await?;
        self.status = Some(status);
        Ok(())
    }

    pub async fn send_text(&mut self, status: HttpStatus, extra_headers: &[&Header<'_>], body: &str)
                           -> Result<(), HttpError> {
        if body.len() == 0 {
            return self.send_without_body(status, extra_headers).await;
        }
        let mut buf = fixed_buffer::FixedBuf::new();
        buf.append(status.as_line());
        buf.append("content-type: text/plain; charset=UTF-8\r\n");
        Self::append_content_length(&mut buf, body.len() as u64)?;
        Self::reject_header("transfer-encoding", extra_headers)?;
        Self::reject_header("content-length", extra_headers)?;
        Self::reject_header("content-type", extra_headers)?;
        Self::reject_header("content-encoding", extra_headers)?;
        Self::append_extra_headers(&mut buf, extra_headers)?;
        buf.append("\r\n");
        self.send(buf.read_all()).await?;
        self.send(body.as_bytes()).await?;
        self.unsent_content_length = Some(0);
        self.status = Some(status);
        Ok(())
    }

    pub async fn send_with_content_length(
        &mut self, status: HttpStatus, extra_headers: &[&Header<'_>], content_length: u64)
        -> Result<(), HttpError> {
        let mut buf = fixed_buffer::FixedBuf::new();
        buf.append(status.as_line());
        //buf.append("transfer-encoding: chunked\r\n");
        Self::append_content_length(&mut buf, content_length)?;
        self.unsent_content_length = Some(content_length);
        Self::reject_header("transfer-encoding", extra_headers)?;
        Self::reject_header("content-length", extra_headers)?;
        Self::append_extra_headers(&mut buf, extra_headers)?;
        buf.append("\r\n");
        self.send(buf.read_all()).await?;
        self.status = Some(status);
        Ok(())
    }

    fn send_expect_100_bytes(&mut self, cx: &mut Context<'_>) -> Option<Poll<tokio::io::Result<usize>>> {
        // TODO(mleonhard) Try to merge this back into HttpReaderWriter::poll_read.  Use mut_self.
        while !self.unsent_expect_100_bytes.is_empty() {
            match tokio::io::AsyncWrite::poll_write(
                self.output.as_mut(), cx, self.unsent_expect_100_bytes) {
                Poll::Pending => {
                    return Some(Poll::Pending);
                }
                Poll::Ready(Err(e)) => {
                    return Some(Poll::Ready(Err(e)));
                }
                Poll::Ready(Ok(bytes_written)) => {
                    if bytes_written > self.unsent_expect_100_bytes.len() {
                        panic!("{:?} output.poll_write wrote more bytes than we asked it to", self.addr);
                    }
                    trace!("{:?} sent {:?}", self.addr, &self.unsent_expect_100_bytes[..bytes_written]);
                    self.unsent_expect_100_bytes = &self.unsent_expect_100_bytes[bytes_written..];
                }
            }
        }
        trace!("{:?} flush", self.addr);
        match tokio::io::AsyncWrite::poll_flush(self.output.as_mut(), cx) {
            Poll::Ready(Ok(())) => None,
            Poll::Pending => Some(Poll::Pending),
            Poll::Ready(Err(e)) => Some(Poll::Ready(Err(e))),
        }
    }
}

impl<'a> AsyncRead for HttpReaderWriter<'a> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8])
                 -> Poll<tokio::io::Result<usize>> {
        if let Some(result) = self.send_expect_100_bytes(cx) {
            return result;
        }
        if buf.len() == 0 {
            return Poll::Ready(Ok(0));
        }
        if self.unread_content_length == 0 {
            return Poll::Ready(Ok(0));  // EOF
        }
        let num_to_read = min(buf.len() as u64, self.unread_content_length) as usize;
        let dest = &mut buf[..num_to_read];
        let readable = self.buffer.readable();
        if readable.len() > 0 {
            let num_bytes = min(readable.len(), dest.len());
            dest.copy_from_slice(&readable[..num_bytes]);
            trace!("{:?} read {} body bytes from buffer", self.addr, num_bytes);
            self.buffer.consume(num_bytes);
            return Poll::Ready(Ok(num_bytes));
        }
        match self.input.as_mut().poll_read(cx, dest) {
            Poll::Ready(Ok(num_bytes)) => {
                trace!("{:?} read {} body bytes", self.addr, num_bytes);
                Poll::Ready(Ok(num_bytes))
            }
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        }
    }
}

impl<'a> AsyncWrite for HttpReaderWriter<'a> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8])
                  -> Poll<tokio::io::Result<usize>> {
        if let Some(unsent_len) = self.unsent_content_length {
            if unsent_len < buf.len() as u64 {
                return Poll::Ready(
                    tokio::io::Result::Err(
                        tokio::io::Error::new(
                            tokio::io::ErrorKind::InvalidInput,
                            "cannot write more than content-length bytes")));
            }
        }
        // https://docs.rs/tokio-util/0.3.1/tokio_util/codec/struct.FramedWrite.html
        let mut_self = &mut self.get_mut();
        match tokio::io::AsyncWrite::poll_write(Pin::new(&mut mut_self.output), cx, buf) {
            Poll::Ready(Ok(bytes_written)) => {
                if bytes_written > buf.len() {
                    panic!("{:?} output.poll_write wrote more bytes than we asked it to", mut_self.addr);
                }
                trace!("{:?} sent {} body bytes", mut_self.addr, bytes_written);
                mut_self.bytes_written += bytes_written as u64;
                if let Some(unsent_len) = mut_self.unsent_content_length {
                    mut_self.unsent_content_length = Some(unsent_len - bytes_written as u64);
                }
                Poll::Ready(Ok(bytes_written))
            }
            other => other
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<tokio::io::Result<()>> {
        trace!("{:?} flush", self.addr);
        tokio::io::AsyncWrite::poll_flush(Pin::new(&mut self.get_mut().output), cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<tokio::io::Result<()>> {
        trace!("{:?} shutdown writer", self.addr);
        tokio::io::AsyncWrite::poll_shutdown(Pin::new(&mut self.get_mut().output), cx)
    }
}

impl<'a> std::fmt::Debug for HttpReaderWriter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut dbg = f.debug_struct("HttpReaderWriter");
        if let Some(method) = self.method.as_ref() {
            dbg.field("method", method);
        }
        if !self.raw_path.is_empty() {
            dbg.field("raw_path", &self.raw_path);
        }
        if self.chunked {
            dbg.field("chunked", &self.chunked);
        }
        if self.content_length > 0 {
            dbg.field("content_length", &self.content_length);
        }
        if self.unread_content_length > 0 {
            dbg.field("unread_content_length", &self.unread_content_length);
        }
        if let Some(status) = self.status.as_ref() {
            dbg.field("status", status);
        }
        if !self.unsent_expect_100_bytes.is_empty() {
            dbg.field("unsent_expect_100_bytes",
                      &escape_ascii(self.unsent_expect_100_bytes));
        }
        if let Some(unsent_content_length) = self.unsent_content_length {
            if unsent_content_length > 0 {
                dbg.field("unsent_content_length", &unsent_content_length);
            }
        }
        if self.bytes_written > 0 {
            dbg.field("bytes_written", &self.bytes_written);
        }
        dbg.finish()
    }
}
