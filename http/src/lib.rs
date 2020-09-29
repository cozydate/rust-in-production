use lazy_static::lazy_static;
use string_wrapper::StringWrapper;

pub mod buffer;
pub mod async_write_logger;
pub mod async_readable;
pub mod split_iterate;

pub fn escape_ascii(input: &[u8]) -> String {
    let mut result = String::new();
    for byte in input {
        for ascii_byte in std::ascii::escape_default(*byte) {
            result.push_str(std::str::from_utf8(&[ascii_byte]).unwrap());
        }
    }
    result
}

// pub async fn send_100_continue<T>(output: &mut T) -> std::io::Result<()>
//     where T: tokio::io::AsyncWrite + std::marker::Unpin
// {
//     tokio::io::AsyncWriteExt::write(output, b"100 Continue\r\n\r\n").await?;
//     Ok(())
// }
//
//                     if req.expecting_100_continue {
//                         send_100_continue(&mut tcp_writer).await?;
//                     }


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
pub enum Http11Error {
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

#[derive(Debug)]
pub enum Http11Method {
    DELETE,
    GET,
    HEAD,
    POST,
    PUT,
    Other(string_wrapper::StringWrapper<[u8; 16]>),
}

impl Http11Method {
    pub fn from_str(s: &str) -> Result<Http11Method, Http11Error> {
        // HTTP/1.1 Request Methods https://tools.ietf.org/html/rfc7231#section-4
        //println!("Http11Method::from_str {:?}", s);
        lazy_static! {
            static ref METHOD_RE: regex::Regex = regex::Regex::new("^[A-Z][A-Z0-9]*$").unwrap();
        }
        if !METHOD_RE.is_match(s) {
            return Err(Http11Error::MethodInvalid);
        }
        match s {
            "DELETE" => Ok(Http11Method::DELETE),
            "GET" => Ok(Http11Method::GET),
            "HEAD" => Ok(Http11Method::HEAD),
            "POST" => Ok(Http11Method::POST),
            "PUT" => Ok(Http11Method::PUT),
            s => StringWrapper::from_str_safe(s)
                .map(|sw| Http11Method::Other(sw))
                .ok_or(Http11Error::MethodTooLong),
        }
    }
}

pub struct Http11RequestLine<'a> {
    pub method: &'a str,
    pub path: &'a str,
}

impl<'a> Http11RequestLine<'a> {
    pub fn parse(line_bytes: &[u8]) -> Result<Http11RequestLine, Http11Error> {
        //println!("Http11RequestLine::parse {:?}", escape_ascii(line_bytes));
        // HTTP/1.1 Request Line https://tools.ietf.org/html/rfc7230#section-3.1.1
        let line = std::str::from_utf8(line_bytes)
            .map_err(|_| Http11Error::RequestLineInvalid)?;
        lazy_static! {
            static ref REQUEST_LINE_RE: regex::Regex =
                regex::Regex::new("^([^ ]+) (/[^ ]*) HTTP/1.1$").unwrap();
        }
        let captures: regex::Captures = REQUEST_LINE_RE.captures(line)
            .ok_or(Http11Error::RequestLineInvalid)?;
        let method = captures.get(1).unwrap().as_str();
        let path = captures.get(2).unwrap().as_str();
        Ok(Http11RequestLine { method, path })
    }

    pub fn method(&self) -> Result<Http11Method, Http11Error> {
        Ok(Http11Method::from_str(&self.method)?)
    }

    pub fn path(&self) -> Result<StringWrapper<[u8; 512]>, Http11Error> {
        let cow_str = percent_encoding::percent_decode_str(self.path)
            .decode_utf8()
            .map_err(|_e| Http11Error::PathInvalid)?;
        let result = StringWrapper::from_str_safe(&cow_str)
            .ok_or(Http11Error::PathTooLong)?;
        Ok(result)
    }
}

pub fn is_chunked(header: &Header) -> Result<bool, Http11Error> {
    if header.value.is_empty() {
        return Ok(false);
    }
    if header.value.eq_ignore_ascii_case("chunked") {
        return Ok(true);
    }
    Err(Http11Error::TransferEncodingHeaderInvalid)
}

pub fn is_100_continue(header: &Header) -> Result<bool, Http11Error> {
    // HTTP/1.1 Expect https://tools.ietf.org/html/rfc7231#section-5.1.1
    if header.value.is_empty() {
        return Ok(false);
    }
    if header.value.eq_ignore_ascii_case("100-continue") {
        return Ok(true);
    }
    Err(Http11Error::ExpectHeaderInvalid)
}

pub fn parse_content_length(header: &Header) -> Result<u64, Http11Error> {
    if header.value.is_empty() {
        return Ok(0);
    }
    let content_length: u64 = std::str::FromStr::from_str(&header.value)
        .map_err(|_e| Http11Error::ContentLengthHeaderInvalid)?;
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
pub struct Http11Request {
    pub method: Http11Method,
    pub path: StringWrapper<[u8; 512]>,
    pub expecting_100_continue: bool,
    pub content_length: u64,
    pub chunked: bool,
}

impl Http11Request {
    pub fn has_body(&self) -> bool {
        // The presence of a message body in a request is signaled by a Content-Length or
        // Transfer-Encoding header field.
        self.chunked || self.content_length > 0
    }

    fn save_header_value(name: &str, value: &str, headers: &mut [&mut Header]) -> Result<(), Http11Error> {
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
                .or(Err(Http11Error::HeaderValueTooLong))?;
        }
        Ok(())
    }

    pub fn parse_headers(
        lines: split_iterate::SplitIterator, headers: &mut [&mut Header], extra_headers: &mut [&mut Header])
        -> Result<(), Http11Error> {
        for line_bytes in lines {
            //println!("Http11Request::parse_headers line {:?}", escape_ascii(line_bytes));
            if line_bytes.is_empty() {
                break;
            }
            // HTTP/1.1 Header Fields https://tools.ietf.org/html/rfc7230#section-3.2
            let line = std::str::from_utf8(line_bytes)
                .or(Err(Http11Error::HeaderLineInvalid))?;
            lazy_static! {
                static ref LINE_RE: regex::Regex
                    = regex::Regex::new("^([^:]+):\\s*(.*?)\\s*$").unwrap();
            }
            let captures: regex::Captures = LINE_RE.captures(line)
                .ok_or(Http11Error::HeaderLineInvalid)?;
            let name = captures.get(1).unwrap().as_str();
            let value = captures.get(2).unwrap().as_str();

            Self::save_header_value(name, value, headers)?;
            Self::save_header_value(name, value, extra_headers)?;
        }
        Ok(())
    }

    pub fn parse_head(head: &[u8], extra_headers: &mut [&mut Header]) -> Result<Http11Request, Http11Error> {
        //println!("Http11Request::parse_head {:?}", escape_ascii(head));
        if extra_headers.len() > 10 {
            return Err(Http11Error::ExtraHeadersTooLong);
        }
        // "HTTP/1.1 Message Syntax and Routing" https://tools.ietf.org/html/rfc7230
        let mut lines = split_iterate::split_iterate(head, b"\r\n");

        let line_bytes = lines.next().ok_or(Http11Error::RequestLineMissing)?;
        let request_line = Http11RequestLine::parse(line_bytes)?;

        let mut content_length = Header::new("content-length");
        let mut expect = Header::new("expect");
        let mut transfer_encoding = Header::new("transfer-encoding");
        Self::parse_headers(
            lines,
            &mut [&mut content_length, &mut expect, &mut transfer_encoding],
            extra_headers)?;
        Ok(Http11Request {
            method: request_line.method()?,
            path: request_line.path()?,
            expecting_100_continue: is_100_continue(&expect)?,
            content_length: parse_content_length(&content_length)?,
            chunked: is_chunked(&transfer_encoding)?,
        })
    }
}

pub async fn read_http11_request<'a, T>(input: &'a mut T, buf: &'a mut buffer::Buffer)
                                        -> std::io::Result<Http11Request>
    where T: tokio::io::AsyncRead + std::marker::Unpin {
    // beatrice_http::buffer::fill_delimited(input, buf, b"\r\n\r\n").await?;
    // let head = beatrice_http::buffer::read_delimited(buf, b"\r\n\r\n")?;
    let head = buf.read_delimited(input, b"\r\n\r\n").await?;
    Http11Request::parse_head(head, &mut [])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{:?}", e).to_string()))
}
