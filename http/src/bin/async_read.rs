use std::pin::Pin;
use std::task::Context;

use tokio::io::AsyncRead;
use tokio::macros::support::Poll;

/// AsyncReadable is an example struct that implements tokio::io::AsyncRead.
struct AsyncReadable(u8);

impl AsyncRead for AsyncReadable {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, mut buf: &mut [u8]) -> Poll<tokio::io::Result<usize>> {
        self.0 += 1;
        use std::io::Write;
        match self.0 {
            1 => {
                cx.waker().clone().wake();  // Tell executor to wake the task again again.
                Poll::Pending
            }
            2 => Poll::Ready(buf.write(b"aaa")),
            3 => Poll::Ready(buf.write(b"bbb")),
            _ => Poll::Ready(tokio::io::Result::Ok(0))
        }
    }
}

pub async fn async_main() {
    let mut async_readable = AsyncReadable(0);
    let mut value = String::new();
    tokio::io::AsyncReadExt::read_to_string(&mut async_readable, &mut value).await.unwrap();
    println!("{:?}", value);
}

pub fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async_main());
}

// $ cargo run --bin async_read
// "aaabbb"
