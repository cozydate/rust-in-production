use std::future::Future;
use std::io::Write;
use std::task::Context;

use tokio::io::{AsyncRead, AsyncWrite, Error};
use tokio::macros::support::{Pin, Poll};

struct NoBorrowCursor(usize);

impl NoBorrowCursor {
    pub fn new() -> NoBorrowCursor { NoBorrowCursor(0) }

    pub fn split<'a, 'b, 'c>(&mut self, data: &'b [u8], sep: &'c [u8]) -> Option<(&'b [u8], &'b [u8])> {
        if self.0 > data.len() {
            panic!("data is smaller than last call");
        }
        let start = if self.0 < sep.len() { 0 } else { self.0 - sep.len() };
        let region = &data[start..];
        for (region_index, window) in region.windows(sep.len()).enumerate() {
            if window == sep {
                let data_index = start + region_index;
                return Some((&data[..data_index], &data[data_index + sep.len()..]));
            }
        }
        self.0 = data.len();
        None
    }
}

pub struct Buffer {
    buf: [u8; 4 * 1024],
    write_index: usize,
    read_index: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: [0; 4 * 1024],
            write_index: 0,
            read_index: 0,
        }
    }

    pub fn writable(&mut self) -> Option<&mut [u8]> {
        if self.write_index >= self.buf.len() {
            // buf ran out of space.
            return None;
        }
        Some(&mut self.buf[self.write_index..])
    }

    pub fn wrote(&mut self, num_bytes: usize) {
        let new_write_index = self.write_index + num_bytes;
        if new_write_index > self.buf.len() {
            panic!("write would overflow");
        }
        self.write_index = new_write_index;
    }

    pub fn readable(&self) -> &[u8] {
        &self.buf[self.read_index..self.write_index]
    }

    pub fn read(&mut self, num_bytes: usize) {
        let new_read_index = self.read_index + num_bytes;
        if new_read_index > self.write_index {
            panic!("read would underflow");
        }
        self.read_index = new_read_index;
    }
}

pub fn escape_ascii(input: &[u8]) -> String {
    let mut result = String::new();
    for byte in input {
        for ascii_byte in std::ascii::escape_default(*byte) {
            result.push_str(std::str::from_utf8(&[ascii_byte]).unwrap());
        }
    }
    result
}

struct LoggingWritable;

impl AsyncWrite for LoggingWritable {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        println!("LoggingWritable::poll_write {:?}", escape_ascii(buf));
        match std::str::from_utf8(buf) {
            Ok(s) => {
                println!("LoggingWritable::poll_write {:?}", s);
            }
            Err(_) => {
                println!("LoggingWritable::poll_write {:?}", buf);
            }
        }
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        println!("LoggingWritable::poll_flush");
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        println!("LoggingWritable::poll_shutdown");
        Poll::Ready(Ok(()))
    }
}

struct AsyncReadable(u8);

impl AsyncRead for AsyncReadable {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, mut buf: &mut [u8]) -> Poll<tokio::io::Result<usize>> {
        self.0 += 1;
        match self.0 {
            1 => {
                cx.waker().clone().wake();
                Poll::Pending
            }
            2 => Poll::Ready(buf.write(b"aaa\r\nbbb\r\n")),
            3 => Poll::Ready(buf.write(b"ccc\r\n")),
            4 => Poll::Ready(buf.write(b"ddd\r\n\r\neee\r\n")),
            5 => Poll::Ready(buf.write(b"fff\r\n")),
            _ => Poll::Ready(tokio::io::Result::Ok(0))
        }
    }
}

pub struct ReadAll<'a, T> where T: AsyncRead {
    input: Pin<&'a mut T>,
    result: String,
}

impl<'a, T> ReadAll<'a, T> where T: AsyncRead {
    pub fn new(input: Pin<&'a mut T>) -> ReadAll<'a, T> {
        ReadAll {
            input,
            result: String::new(),
        }
    }
}

impl<'a, T> Future for ReadAll<'a, T>
    where T: AsyncRead {
    type Output = tokio::io::Result<String>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut buf: [u8; 1024] = [0; 1024];
        loop {
            println!("poll_read");
            match self.input.as_mut().poll_read(cx, &mut buf[..]) {
                Poll::Pending => {
                    println!("Poll::Pending");
                    return Poll::Pending;
                }
                Poll::Ready(Ok(0)) => {
                    println!("Poll::Ready({:?})", self.result);
                    return Poll::Ready(tokio::io::Result::Ok(self.result.clone()));
                }
                Poll::Ready(Ok(num_bytes_read)) => {
                    println!("read {} bytes", num_bytes_read);
                    match std::str::from_utf8(&buf[..num_bytes_read]) {
                        Err(e) => {
                            println!("Poll::Ready(Err({:?}))", e);
                            return Poll::Ready(
                                tokio::io::Result::Err(
                                    tokio::io::Error::new(
                                        tokio::io::ErrorKind::InvalidData,
                                        e.to_string())));
                        }
                        Ok(s) => {
                            println!("read {:?}", s);
                            self.result.push_str(s);
                        }
                    }
                }
                Poll::Ready(Err(e)) => {
                    println!("Poll::Ready(Err({:?}))", e);
                    return Poll::Ready(Err(e));
                }
            }
        }
    }
}

pub fn read_all<T>(input: &mut T) -> ReadAll<T>
    where T: AsyncRead + std::marker::Unpin {
    ReadAll::new(Pin::new(input))
}

#[derive(Debug)]
pub enum Http11Error {
    StdIoError(std::io::Error),
    UnexpectedEof,
    RequestHeaderTooLong,
    BadStatusLine,
    MissingContentLengthHeader,
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

#[derive(Debug)]
struct Http11Request {
    pub expecting_100_continue: bool,
    pub method: Http11Method,
    pub path: string_wrapper::StringWrapper<[u8; 512]>,
    pub has_body: bool,
    pub content_length: Option<u64>,
    pub chunked: bool,
}

impl Http11Request {
    fn parse_head(head: &[u8]) -> Result<Http11Request, Http11Error> {
        println!("ReadHttp11Request head {:?}", escape_ascii(head));
        // match std::str::from_utf8(&buf[..num_bytes_read]) {
        //     Err(e) => {
        //         println!("ReadHttp11Request Poll::Ready(Err({:?}))", e);
        //         return Poll::Ready(
        //             tokio::io::Result::Err(
        //                 tokio::io::Error::new(
        //                     tokio::io::ErrorKind::InvalidData,
        //                     e.to_string())));
        //     }
        //     Ok(s) => {
        //         println!("read {:?}", s);
        //         result.push_str(s);
        //     }
        // }
        Ok(Http11Request {
            expecting_100_continue: true,
            method: Http11Method::Head,
            path: string_wrapper::StringWrapper::from_str(""),
            has_body: false,
            content_length: None,
            chunked: false,
        })
    }
}

// ReadHttp11Request reads the full HTTP header from `input` into an internal buffer,
// then parses it and returns an `Http11Request` struct.
// Returns Err(RequestHeaderTooLong) if the header is longer than 4 KiB.
struct ReadHttp11Request<'a, T> where T: AsyncRead {
    input: Pin<&'a mut T>,
    buf: Pin<&'a mut Buffer>,
    cursor: NoBorrowCursor,
}

impl<'a, T> ReadHttp11Request<'a, T> where T: AsyncRead {
    pub fn new(input: Pin<&'a mut T>, buf: Pin<&'a mut Buffer>) -> ReadHttp11Request<'a, T> {
        ReadHttp11Request {
            input,
            buf,
            cursor: NoBorrowCursor::new(),
        }
    }
}

impl<'a, T> Future for ReadHttp11Request<'a, T>
    where T: AsyncRead {
    type Output = Result<Http11Request, Http11Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let borrowed_self = self.get_mut();
        let input = &mut borrowed_self.input;
        let buf = &mut borrowed_self.buf;
        let cursor = &mut borrowed_self.cursor;
        loop {
            println!("ReadHttp11Request poll_read");
            let writable = match buf.writable() {
                Some(s) => s,
                None => {
                    return Poll::Ready(Err(Http11Error::RequestHeaderTooLong));
                }
            };
            match input.as_mut().poll_read(cx, writable) {
                Poll::Pending => {
                    println!("ReadHttp11Request pending");
                    return Poll::Pending;
                }
                Poll::Ready(Ok(0)) => {
                    println!("ReadHttp11Request eof");
                    return Poll::Ready(Err(Http11Error::UnexpectedEof));
                }
                Poll::Ready(Ok(num_bytes_read)) => {
                    println!("ReadHttp11Request read {} bytes", num_bytes_read);
                    buf.wrote(num_bytes_read);
                    println!("ReadHttp11Request data {:?}", escape_ascii(buf.readable()));
                    let (head_len, result) = match cursor.split(buf.readable(), b"\r\n\r\n") {
                        Some((head, rest)) => {
                            println!("ReadHttp11Request found head={:?} rest={:?}",
                                     escape_ascii(head),
                                     escape_ascii(rest));
                            (head.len(), Http11Request::parse_head(head))
                        }
                        None => {
                            println!("ReadHttp11Request head not found, pending");
                            cx.waker().clone().wake();
                            return Poll::Pending;
                        }
                    };
                    buf.read(head_len);
                    return Poll::Ready(result);
                }
                Poll::Ready(Err(e)) => {
                    println!("Poll::Ready(Err({:?}))", e);
                    return Poll::Ready(Err(Http11Error::StdIoError(e)));
                }
            }
        }
    }
}

fn read_http11_request<'a, T>(input: &'a mut T, buf: &'a mut Buffer) -> ReadHttp11Request<'a, T>
    where T: AsyncRead + std::marker::Unpin {
    ReadHttp11Request::new(Pin::new(input), Pin::new(buf))
}

async fn send_100_continue<T>(output: &mut T) -> Result<(), Http11Error>
    where T: AsyncWrite + std::marker::Unpin
{
    tokio::io::AsyncWriteExt::write(output, b"100 Continue\r\n\r\n")
        .await
        .map_err(|e| Http11Error::StdIoError(e))?;
    Ok(())
}

pub async fn async_main() -> Result<(), Http11Error> {
    let mut async_readable = AsyncReadable(0);
    let mut buffer = Buffer::new();
    let req = read_http11_request(&mut async_readable, &mut buffer).await?;
    println!("req {:?}", req);
    println!("buffer rest {:?}", escape_ascii(buffer.readable()));
    let mut logging_writable = LoggingWritable;
    if req.expecting_100_continue {
        send_100_continue(&mut logging_writable).await?;
    }
    Ok(())
}

pub fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async_main()).unwrap();
}
