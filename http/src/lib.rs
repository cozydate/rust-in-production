use std::convert::TryFrom;
use std::fmt::Formatter;

use lazy_static::lazy_static;
use string_wrapper::StringWrapper;

pub mod buffer;
pub mod async_write_logger;
pub mod async_readable;
pub mod split_iterate;
pub mod async_write_buffer;

pub fn escape_ascii(input: &[u8]) -> String {
    let mut result = String::new();
    for byte in input {
        for ascii_byte in std::ascii::escape_default(*byte) {
            result.push_str(std::str::from_utf8(&[ascii_byte]).unwrap());
        }
    }
    result
}


// use std::net::SocketAddr;
// use std::sync::Arc;
// use std::time::Duration;

// use tokio::net::TcpStream;

// use async_trait::async_trait;
// use logging::info;
// use logging::warn;

// #[async_trait]
// pub trait HttpSessionHandler {
//     async fn handle(&self, session: &mut HttpSession);
// }

// pub struct HttpSession {
//     pub tcp_stream: tokio::net::TcpStream,
//     addr: std::net::SocketAddr,
// }

// impl std::fmt::Debug for HttpSession {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // https://doc.rust-lang.org/std/fmt/struct.Formatter.html
//         f.debug_struct("HttpSession")
//             .field("addr", &self.addr)
//             .finish()
//     }
// }

// impl HttpSession {
// // pub fn shutdown(self) {
// //     if let Err(e) = self.tcp_stream.shutdown(std::net::Shutdown::Both) {
// //         warn!("Failed calling shutdown for tcp socket: {:?}", e);
// //     };
// // }
// }

// pub struct HttpServer {
//     pub socket_addr: std::net::SocketAddr,
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

// struct Handler {}

// #[async_trait]
// impl HttpSessionHandler for Handler {
//     async fn handle(&self, session: &mut HttpSession) {
//         info!("Got {:?}", session);
//         tokio::time::delay_for(Duration::from_secs(1)).await;
//     }
// }

#[derive(Debug)]
pub enum HttpError {
    IoError(std::io::Error),
    ParseError(HttpCallerError),
    ProcessingError(HttpRequest, HttpStatus),
}

impl HttpError {
    pub fn from_io_err(e: std::io::Error) -> HttpError {
        HttpError::IoError(e)
    }
}

#[derive(Debug)]
pub enum HttpCallerError {
    ExtraHeadersTooLong,
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
            Self::ExtraHeadersTooLong => HttpStatus::RequestHeaderFieldsTooLarge431,
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

#[derive(Debug, PartialEq)]
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
        //println!("Http11Method::from_str {:?}", s);
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
}

pub struct HttpRequestLine<'a> {
    pub method: &'a str,
    pub path: &'a str,
}

impl<'a> HttpRequestLine<'a> {
    pub fn parse(line_bytes: &[u8]) -> Result<HttpRequestLine, HttpError> {
        //println!("Http11RequestLine::parse {:?}", escape_ascii(line_bytes));
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
        let path = captures.get(2).unwrap().as_str();
        Ok(HttpRequestLine { method, path })
    }

    pub fn method(&self) -> Result<HttpMethod, HttpError> {
        Ok(HttpMethod::from_str(&self.method)?)
    }

    pub fn path(&self) -> Result<StringWrapper<[u8; 512]>, HttpError> {
        let cow_str = percent_encoding::percent_decode_str(self.path)
            .decode_utf8()
            .map_err(|_e| HttpError::ParseError(HttpCallerError::PathInvalid))?;
        let result = StringWrapper::from_str_safe(&cow_str)
            .ok_or(HttpError::ParseError(HttpCallerError::PathTooLong))?;
        Ok(result)
    }
}

pub fn is_chunked(header: &Header) -> Result<bool, HttpError> {
    if header.value.is_empty() {
        return Ok(false);
    }
    if header.value.eq_ignore_ascii_case("chunked") {
        return Ok(true);
    }
    Err(HttpError::ParseError(HttpCallerError::TransferEncodingHeaderInvalid))
}

pub fn is_100_continue(header: &Header) -> Result<bool, HttpError> {
    // HTTP/1.1 Expect https://tools.ietf.org/html/rfc7231#section-5.1.1
    if header.value.is_empty() {
        return Ok(false);
    }
    if header.value.eq_ignore_ascii_case("100-continue") {
        return Ok(true);
    }
    Err(HttpError::ParseError(HttpCallerError::ExpectHeaderInvalid))
}

pub fn parse_content_length(header: &Header) -> Result<u64, HttpError> {
    if header.value.is_empty() {
        return Ok(0);
    }
    let content_length: u64 = std::str::FromStr::from_str(&header.value)
        .map_err(|_e| HttpError::ParseError(HttpCallerError::ContentLengthHeaderInvalid))?;
    Ok(content_length)
}

pub struct Header<'a> {
    pub name: &'a str,
    pub value: StringWrapper<[u8; 256]>,
}

impl<'a> Header<'a> {
    pub fn new<'b>(name: &'b str) -> Header<'b> {
        Header { name, value: StringWrapper::from_str("") }
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: StringWrapper<[u8; 512]>,
    pub expect_100_continue: bool,
    pub content_length: u64,
    pub chunked: bool,
}

impl HttpRequest {
    pub fn has_body(&self) -> bool {
        // The presence of a message body in a request is signaled by a Content-Length or
        // Transfer-Encoding header field.
        self.chunked || self.content_length > 0
    }

    fn save_header_value(name: &str, value: &str, headers: &mut [&mut Header])
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

    pub fn parse_headers(
        lines: split_iterate::SplitIterator, headers: &mut [&mut Header], extra_headers: &mut [&mut Header])
        -> Result<(), HttpError> {
        for line_bytes in lines {
            //println!("Http11Request::parse_headers line {:?}", escape_ascii(line_bytes));
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

            Self::save_header_value(name, value, headers)?;
            Self::save_header_value(name, value, extra_headers)?;
        }
        Ok(())
    }

    pub fn parse_head(head: &[u8], extra_headers: &mut [&mut Header]) -> Result<HttpRequest, HttpError> {
        //println!("Http11Request::parse_head {:?}", escape_ascii(head));
        if extra_headers.len() > 10 {
            return Err(HttpError::ParseError(HttpCallerError::ExtraHeadersTooLong));
        }
        // "HTTP/1.1 Message Syntax and Routing" https://tools.ietf.org/html/rfc7230
        let mut lines = split_iterate::split_iterate(head, b"\r\n");

        let line_bytes = lines.next()
            .ok_or(HttpError::ParseError(HttpCallerError::RequestLineMissing))?;
        let request_line = HttpRequestLine::parse(line_bytes)?;

        let mut content_length = Header::new("content-length");
        let mut expect = Header::new("expect");
        let mut transfer_encoding = Header::new("transfer-encoding");
        Self::parse_headers(
            lines,
            &mut [&mut content_length, &mut expect, &mut transfer_encoding],
            extra_headers)?;
        Ok(HttpRequest {
            method: request_line.method()?,
            path: request_line.path()?,
            expect_100_continue: is_100_continue(&expect)?,
            content_length: parse_content_length(&content_length)?,
            chunked: is_chunked(&transfer_encoding)?,
        })
    }
}

pub async fn read_http_request<'a, 'b, T>(input: &'a mut T, buf: &'a mut buffer::Buffer<'b>)
                                          -> Result<HttpRequest, HttpError>
    where T: tokio::io::AsyncRead + std::marker::Unpin {
    // beatrice_http::buffer::fill_delimited(input, buf, b"\r\n\r\n").await?;
    // let head = beatrice_http::buffer::read_delimited(buf, b"\r\n\r\n")?;
    let head = buf.read_delimited(input, b"\r\n\r\n")
        .await
        .map_err(|e| HttpError::IoError(e))
        ?;
    HttpRequest::parse_head(head, &mut [])
}

#[derive(Debug, Clone, Copy)]
pub enum HttpStatus {
    Continue100,
    Ok200,
    Created201,
    BadRequest400,
    NotFound404,
    MethodNotAllowed405,
    LengthRequired411,
    UriTooLong414,
    RequestHeaderFieldsTooLarge431,
    InternalServerError500,
}

impl HttpStatus {
    pub fn as_line(&self) -> &'static str {
        match self {
            HttpStatus::Continue100 => "HTTP/1.1 100 Continue",
            HttpStatus::Ok200 => "HTTP/1.1 200 OK",
            HttpStatus::Created201 => "HTTP/1.1 201 Created",
            HttpStatus::BadRequest400 => "400 Bad Request",
            HttpStatus::NotFound404 => "HTTP/1.1 404 Not Found",
            HttpStatus::MethodNotAllowed405 => "HTTP/1.1 405 Method Not Allowed",
            HttpStatus::LengthRequired411 => "HTTP/1.1 411 Length Required",
            HttpStatus::UriTooLong414 => "414 URI Too Long",
            HttpStatus::RequestHeaderFieldsTooLarge431 => "431 Request Header Fields Too Large",
            HttpStatus::InternalServerError500 => "HTTP/1.1 500 Internal Server Error",
        }
    }
}

pub struct HttpResponseWriter<'a, T> where T: tokio::io::AsyncWrite + std::marker::Unpin {
    output: &'a mut T,
    status: Option<HttpStatus>,
    bytes_written: u64,
}

impl<'a, T> std::fmt::Debug for HttpResponseWriter<'a, T> where T: tokio::io::AsyncWrite + std::marker::Unpin {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Http11ResponseWriter{{{:?}, bytes_written={}}}",
               self.status, self.bytes_written)
    }
}

impl<'a, T> HttpResponseWriter<'a, T> where T: tokio::io::AsyncWrite + std::marker::Unpin {
    pub fn new<'b>(output: &'b mut T) -> HttpResponseWriter<'b, T> {
        HttpResponseWriter {
            output,
            status: None,
            bytes_written: 0,
        }
    }

    pub async fn send_without_body(&mut self, status: HttpStatus) -> std::io::Result<()> {
        let mut line: StringWrapper<[u8; 64]> = StringWrapper::from_str("");
        line.push_str(status.as_line());
        line.push_str("\r\n\r\n");
        let line_bytes = line.as_bytes();
        tokio::io::AsyncWriteExt::write(self.output, line_bytes).await?;
        self.status = Some(status);
        self.bytes_written += u64::try_from(line_bytes.len()).unwrap();
        Ok(())
    }

    pub async fn send_text(&mut self, status: HttpStatus, body: &str) -> Result<(), HttpError> {
        let mut mem: [u8; 100] = [0; 100];
        let mut buf = buffer::Buffer::new(&mut mem[..]);
        buf.append(status.as_line());
        buf.append("\r\n");
        buf.append("content-type: text/plain; charset=UTF-8\r\n");
        buf.append("content-length: ");
        itoa::write(&mut buf, body.len()).unwrap();  // Write num without allocating.
        buf.append("\r\n");
        buf.append("\r\n");
        let to_write = buf.readable();
        tokio::io::AsyncWriteExt::write_all(self.output, to_write)
            .await
            .map_err(HttpError::from_io_err)?;
        self.bytes_written += u64::try_from(to_write.len()).unwrap();
        buf.read_all();

        let body_bytes = body.as_bytes();
        tokio::io::AsyncWriteExt::write(self.output, body_bytes)
            .await
            .map_err(HttpError::from_io_err)?;
        self.bytes_written += u64::try_from(body_bytes.len()).unwrap();

        self.status = Some(status);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.status = None;
        self.bytes_written = 0;
    }
}
