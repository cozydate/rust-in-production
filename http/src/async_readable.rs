use std::io::Write;
use std::task::Context;

use tokio::io::AsyncRead;
use tokio::macros::support::{Pin, Poll};

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
