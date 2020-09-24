use std::task::Context;

use tokio::io::{AsyncWrite, Error};
use tokio::macros::support::{Pin, Poll};

use crate::escape_ascii;

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
