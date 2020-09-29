use std::io::Write;
use std::task::Context;

use tokio::io::AsyncRead;
use tokio::macros::support::{Pin, Poll};

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
