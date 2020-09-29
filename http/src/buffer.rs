use std::future::Future;
use std::io::Write;
use std::task::Context;

use tokio::io::AsyncRead;
use tokio::macros::support::{Pin, Poll};

use crate::escape_ascii;
use crate::no_borrow_cursor::NoBorrowCursor;

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

    /// Reads from `input` into the buffer until it finds `delim`.
    /// Returns the slice up until `delim`.
    /// Leaves unused bytes in the buffer.
    /// If the buffer already contains `delim`, returns the data immediately without reading from `input`.
    /// Returns Err(InvalidData) if the buffer fills up before `delim` is found.
    pub async fn read_delimited<'a, T>(&'a mut self, input: &'a mut T, delim: &[u8])
                                       -> std::io::Result<&'a [u8]>
        where T: AsyncRead + std::marker::Unpin {
        loop {
            //println!("read_delimited() data {:?}", escape_ascii(self.readable()));
            if let Some(delim_index) = self.readable()
                .windows(delim.len())
                .enumerate()
                .filter(|(_index, window)| *window == delim)
                .map(|(index, _window)| index)
                .next()
            {
                let result_start = self.read_index;
                self.read_index = delim_index + delim.len();
                //println!("read_delimited() rest {:?}", escape_ascii(self.readable()));
                return Ok(&self.buf[result_start..delim_index]);
            }
            let writable = self.writable()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData, "buffer full"))?;
            let num_bytes_read =
                tokio::io::AsyncReadExt::read(input, writable).await?;
            if num_bytes_read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof, "eof before delim read"));
            }
            //println!("read_delimited() read {} bytes", num_bytes_read);
            self.wrote(num_bytes_read);
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
        //println!("Buffer poll_read");
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

/// Future returned by `read_delimited_and_parse`.  Returns `std::io::Result<T>`.
pub struct ReadDelimitedAndParse<'a, T1, T2> where T1: AsyncRead {
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

/// Equivalent to:
/// ```
/// use tokio::io::AsyncRead;
/// use beatrice_http::buffer::Buffer;
/// async fn read_delimited_and_parse(input: &mut dyn AsyncRead + Unpin, buf: &mut Buffer, delim: &[u8], parser: fn(&[u8]) -> T) -> std::io::Result<T> {}
/// ```
///
/// Reads from `input` into `buf` until it finds `delim`.
/// Then calls `parser` with the slice of `buf` up until `delim` and returns result.
/// If `buf` already contains `delim`, this will not read from `input`.
/// Returns Err(InvalidData) if `buffer` fills up before `delim` is found.
/// Leaves unused bytes in `buf`.
pub fn read_delimited_and_parse<'a, T1, T2>(
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
