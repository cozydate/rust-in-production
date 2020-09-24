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
        if self.read_index == self.write_index {
            // All data has been read.  Reset the buffer.
            self.write_index = 0;
            self.read_index = 0;
        }
    }

    // shift() moves data to the beginning of the buffer.
    pub fn shift(&mut self) {
        if self.read_index == 0 {
            return;
        }
        self.buf.copy_within(self.read_index..self.write_index, 0)
    }
}

impl AsyncRead for Buffer {
    fn poll_read(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, mut buf: &mut [u8]) -> Poll<tokio::io::Result<usize>> {
        println!("Buffer poll_read");
        let bytes_read = match buf.write(self.readable()) {
            Err(e) => {
                return Poll::Ready(Err(e));
            }
            Ok(bytes_read) => bytes_read,
        };
        self.read(bytes_read);
        Poll::Ready(Ok(bytes_read))
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

struct AsyncWriteLogger;

impl AsyncWrite for AsyncWriteLogger {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
        println!("AsyncWriteLogger::poll_write {:?}", escape_ascii(buf));
        match std::str::from_utf8(buf) {
            Ok(s) => {
                println!("AsyncWriteLogger::poll_write {:?}", s);
            }
            Err(_) => {
                println!("AsyncWriteLogger::poll_write {:?}", buf);
            }
        }
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        println!("AsyncWriteLogger::poll_flush");
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        println!("AsyncWriteLogger::poll_shutdown");
        Poll::Ready(Ok(()))
    }
}

pub enum AsyncReadableAction {
    Pending,
    Data(Vec<u8>),
    Error(std::io::Error),
}

struct AsyncReadable(Vec<AsyncReadableAction>);

impl AsyncRead for AsyncReadable {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, mut buf: &mut [u8]) -> Poll<tokio::io::Result<usize>> {
        if self.0.is_empty() {
            return Poll::Ready(tokio::io::Result::Ok(0));
        }
        match self.0.remove(0) {
            AsyncReadableAction::Pending => {
                cx.waker().clone().wake();
                Poll::Pending
            }
            AsyncReadableAction::Data(bytes) => Poll::Ready(buf.write(&bytes)),
            AsyncReadableAction::Error(e) => Poll::Ready(Err(e)),
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
        println!("Http11Request::parse_head {:?}", escape_ascii(head));

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

// ReadDelimitedAndParse reads from `input` into `buf` until it reads `delim`.
// Then it calls `parser` with the slice of `buf` up until `delim` and returns result.
// If `buf` already contains `delim`, this will not read from `input`.
// Returns Err(InvalidData) if `buffer` fills up before `delim` is found.
struct ReadDelimitedAndParse<'a, T1, T2> where T1: AsyncRead {
    input: Pin<&'a mut T1>,
    buf: Pin<&'a mut Buffer>,
    delim: &'a [u8],
    parser: fn(&[u8]) -> T2,
    cursor: NoBorrowCursor,
}

impl<'a, T1, T2> Future for ReadDelimitedAndParse<'a, T1, T2>
    where T1: AsyncRead {
    type Output = std::io::Result<T2>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("ReadDelimitedAndParse poll");
        let borrowed_self = self.get_mut();
        let input = &mut borrowed_self.input;
        let buf = &mut borrowed_self.buf;
        let delim = &borrowed_self.delim;
        let parser = &borrowed_self.parser;
        let cursor = &mut borrowed_self.cursor;
        loop {
            println!("ReadDelimitedAndParse data {:?}", escape_ascii(buf.readable()));
            let (head_len, option_result) = match cursor.split(buf.readable(), delim) {
                Some((head, rest)) => {
                    println!("ReadDelimitedAndParse found head={:?} rest={:?}",
                             escape_ascii(head),
                             escape_ascii(rest));
                    (head.len(), Some(parser(head)))
                }
                None => {
                    println!("ReadDelimitedAndParse head not found, pending");
                    (0, None)
                }
            };
            if head_len != 0 {
                buf.read(head_len);
            }
            if let Some(result) = option_result {
                return Poll::Ready(Ok(result));
            }

            let writable = match buf.writable() {
                Some(s) => s,
                None => {
                    return Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData, "buffer full")));
                }
            };
            match input.as_mut().poll_read(cx, writable) {
                Poll::Pending => {
                    println!("ReadDelimitedAndParse pending");
                    return Poll::Pending;
                }
                Poll::Ready(Ok(0)) => {
                    println!("ReadDelimitedAndParse eof");
                    return Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof, "eof before delim read")));
                }
                Poll::Ready(Ok(num_bytes_read)) => {
                    println!("ReadDelimitedAndParse read {} bytes", num_bytes_read);
                    buf.wrote(num_bytes_read);
                }
                Poll::Ready(Err(e)) => {
                    println!("Poll::Ready(Err({:?}))", e);
                    return Poll::Ready(Err(e));
                }
            }
        }
    }
}

fn read_delimited_and_parse<'a, T1, T2>(
    input: &'a mut T1,
    buf: &'a mut Buffer,
    delim: &'a [u8],
    parser: fn(&[u8]) -> T2) -> ReadDelimitedAndParse<'a, T1, T2>
    where T1: AsyncRead + std::marker::Unpin {
    ReadDelimitedAndParse {
        input: Pin::new(input),
        buf: Pin::new(buf),
        delim,
        parser,
        cursor: NoBorrowCursor::new(),
    }
}

async fn send_100_continue<T>(output: &mut T) -> std::io::Result<()>
    where T: AsyncWrite + std::marker::Unpin
{
    tokio::io::AsyncWriteExt::write(output, b"100 Continue\r\n\r\n").await?;
    Ok(())
}

pub async fn async_main() -> std::io::Result<()> {
    let mut async_readable = AsyncReadable(vec!(
        AsyncReadableAction::Data("aaa\r\nbbb\r\n".into()),
        AsyncReadableAction::Data("ccc\r\n".into()),
        AsyncReadableAction::Pending,
        AsyncReadableAction::Data("ddd\r\n\r\neee\r\n".into()),
        AsyncReadableAction::Error(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "eof")),
        AsyncReadableAction::Data("fff\r\n".into()),
    ));
    let mut buffer = Buffer::new();
    println!("buffer {:?}", escape_ascii(buffer.readable()));
    buffer.shift();
    println!("buffer {:?}", escape_ascii(buffer.readable()));
    let req = read_delimited_and_parse(
        &mut async_readable,
        &mut buffer,
        b"\r\n\r\n",
        Http11Request::parse_head)
        .await?
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{:?}", e).to_string()))?;
    println!("req {:?}", req);
    println!("buffer {:?}", escape_ascii(buffer.readable()));
    buffer.shift();
    println!("buffer {:?}", escape_ascii(buffer.readable()));
    let mut rest = String::new();
    tokio::io::AsyncReadExt::read_to_string(&mut buffer, &mut rest).await.unwrap();
    println!("buffer {:?}", rest);
    let mut logging_writable = AsyncWriteLogger;
    if req.expecting_100_continue {
        send_100_continue(&mut logging_writable).await?;
    }
    Ok(())
}

pub fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async_main()).unwrap();
}
