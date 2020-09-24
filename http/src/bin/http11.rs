// This program shows how to handle HTTP 1.1 requests.
use std::println;
use std::str::FromStr;

use lazy_static::lazy_static;
use string_wrapper::StringWrapper;
use tokio::io::AsyncRead;

use beatrice_http::escape_ascii;
use beatrice_http::split_iterate::split_iterate;

#[derive(Debug)]
pub enum Http11Error {
    BadStatusLine,
    BadMethod,
    BadPath,
    BadHeader,
    BadTransferEncodingHeader,
    BadExpectHeader,
    DuplicateContentLengthHeader,
    MissingContentLengthHeader,
    BadContentLengthHeader,
}

#[derive(Debug)]
pub enum Http11Method {
    Delete,
    Get,
    Head,
    Post,
    Put,
    Other(string_wrapper::StringWrapper<[u8; 16]>),
}

impl Http11Method {
    pub fn from_str(s: &str) -> Option<Http11Method> {
        // HTTP/1.1 Request Methods https://tools.ietf.org/html/rfc7231#section-4
        println!("Http11Method::from_str {:?}", s);
        lazy_static! {
            static ref METHOD_RE: regex::Regex = regex::Regex::new("^[A-Z][A-Z0-9]*$").unwrap();
        }
        if !METHOD_RE.is_match(s) {
            return None;
        }
        match s {
            "DELETE" => Some(Http11Method::Delete),
            "GET" => Some(Http11Method::Get),
            "HEAD" => Some(Http11Method::Head),
            "POST" => Some(Http11Method::Post),
            "PUT" => Some(Http11Method::Put),
            s => StringWrapper::from_str_safe(s)
                .map(|sw| Http11Method::Other(sw)),
        }
    }
}

#[derive(Debug)]
struct Http11Request {
    pub method: Http11Method,
    pub path: StringWrapper<[u8; 512]>,
    pub expecting_100_continue: bool,
    pub content_length: u64,
    pub chunked: bool,
}

impl Http11Request {
    pub fn has_body(&self) -> bool {
        self.chunked || self.content_length > 0
    }

    fn parse_request_line(line_bytes: &[u8])
                          -> Result<(Http11Method, StringWrapper<[u8; 512]>), Http11Error> {
        println!("Http11Request::parse_request_line {:?}", escape_ascii(line_bytes));
        // HTTP/1.1 Request Line https://tools.ietf.org/html/rfc7230#section-3.1.1
        let line = std::str::from_utf8(line_bytes).map_err(|_| Http11Error::BadStatusLine)?;
        lazy_static! {
            static ref LINE_RE: regex::Regex =
                regex::Regex::new("^([^ ]+) (/[^ ]*) HTTP/1.1$").unwrap();
            static ref METHOD_RE: regex::Regex = regex::Regex::new("^[A-Z][A-Z0-9]*$").unwrap();
        }
        let captures: regex::Captures = LINE_RE.captures(line).ok_or(Http11Error::BadStatusLine)?;

        let method = Http11Method::from_str(captures.get(1).unwrap().as_str())
            .ok_or(Http11Error::BadMethod)?;

        let path_percent_encoded = captures.get(2).unwrap().as_str();
        let path_cow_str = percent_encoding::percent_decode_str(path_percent_encoded)
            .decode_utf8().map_err(|_e| Http11Error::BadPath)?;
        let path: StringWrapper<[u8; 512]> = StringWrapper::from_str_safe(&path_cow_str)
            .ok_or(Http11Error::BadPath)?;

        Ok((method, path))
    }

    fn parse_header_line(line_bytes: &[u8]) -> Option<(&str, &str)> {
        println!("Http11Request::parse_header_line {:?}", escape_ascii(line_bytes));
        // HTTP/1.1 Header Fields https://tools.ietf.org/html/rfc7230#section-3.2
        let line = std::str::from_utf8(line_bytes).ok()?;
        lazy_static! {
            static ref LINE_RE: regex::Regex = regex::Regex::new("^([^:]+):\\s*(.*?)\\s*$").unwrap();
        }
        let captures: regex::Captures = LINE_RE.captures(line)?;
        Some((captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str()))
    }

    fn parse_head(head: &[u8]) -> Result<Http11Request, Http11Error> {
        println!("Http11Request::parse_head {:?}", escape_ascii(head));
        // "HTTP/1.1 Message Syntax and Routing" https://tools.ietf.org/html/rfc7230
        let mut lines = split_iterate(head, b"\r\n");

        let line_bytes = lines.next().ok_or(Http11Error::BadStatusLine)?;
        let (method, path) = Self::parse_request_line(line_bytes)?;

        let mut chunked = false;
        let mut content_length: Option<u64> = None;
        let mut expecting_100_continue = false;
        for line_bytes in lines {
            if line_bytes.is_empty() {
                break;
            }
            let (name, value) =
                Self::parse_header_line(line_bytes).ok_or(Http11Error::BadHeader)?;
            let name_lowercase: StringWrapper<[u8; 64]> = StringWrapper::from_str_safe(name)
                .ok_or(Http11Error::BadHeader)?;
            match name_lowercase.as_ref() {
                "content-length" => {
                    if content_length.is_some() {
                        return Err(Http11Error::DuplicateContentLengthHeader);
                    }
                    content_length = Some(u64::from_str(value)
                        .map_err(|_e| Http11Error::BadContentLengthHeader)?);
                }
                "transfer-encoding" => {
                    if value.eq_ignore_ascii_case("chunked") {
                        chunked = true;
                    } else {
                        return Err(Http11Error::BadTransferEncodingHeader);
                    }
                }
                "expect" => {
                    // HTTP/1.1 Expect https://tools.ietf.org/html/rfc7231#section-5.1.1
                    if value.eq_ignore_ascii_case("100-continue") {
                        expecting_100_continue = true;
                    } else {
                        return Err(Http11Error::BadExpectHeader);
                    }
                }
                _ => {
                    println!("Http11Request::parse_head ignoring header {:?}={:?}", name_lowercase, value);
                }
            };
        }

        // The presence of a message body in a request is signaled by a Content-Length or
        // Transfer-Encoding header field.

        Ok(Http11Request {
            method,
            path,
            expecting_100_continue,
            content_length: content_length.unwrap_or(0),
            chunked,
        })
    }
}

async fn read_http11_request<'a, T>(input: &'a mut T, buf: &'a mut beatrice_http::buffer::Buffer) -> std::io::Result<Http11Request>
    where T: AsyncRead + std::marker::Unpin {
    beatrice_http::buffer::read_delimited_and_parse(
        input,
        buf,
        b"\r\n\r\n",
        Http11Request::parse_head)
        .await?
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{:?}", e).to_string()))
}

async fn async_main() -> () {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let mut listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!(
        "INFO server listening on {}",
        listener.local_addr().unwrap()
    );
    tokio::spawn(async move {
        loop {
            let (mut tcp_stream, _addr) = listener.accept().await.unwrap();
            let (mut tcp_reader, mut tcp_writer) = tcp_stream.split();
            let mut buffer = beatrice_http::buffer::Buffer::new();
            match read_http11_request(&mut tcp_reader, &mut buffer).await {
                Err(e) => {
                    println!("WARN server read error: {:?}", e);
                    return;
                }
                Ok(req) => {
                    println!("INFO server got req {:?}", req);
                }
            }
            use tokio::io::AsyncWriteExt;
            if let Err(e) = tcp_writer.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 5\r\n\r\nbody1").await {
                println!("WARN server write error: {:?}", e);
                return;
            }
        }
    });

    let response = reqwest::Client::new()
        .put("http://127.0.0.1:1690/path1")
        .body("req1")
        .send()
        .await
        .unwrap();
    println!("INFO client response {:?}", response);
    assert_eq!(200, response.status().as_u16());
    let body = response.bytes().await.unwrap();
    println!("INFO client response body {:?}", body);
    assert_eq!(bytes::Bytes::from_static(b"body1"), body);
}

pub fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async_main());
    runtime.shutdown_background();
}

// $ cargo run --bin tcp
// INFO server listening on 127.0.0.1:1690
// INFO client read "response"
