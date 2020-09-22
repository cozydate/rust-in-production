use std::future::Future;
use std::pin::Pin;
use std::task::Context;

use tokio::io::AsyncRead;
use tokio::macros::support::Poll;

// ReadAll is an example Future that reads all the bytes from the provided AsyncRead struct,
// converts them to UTF-8, and returns them as a string.
struct ReadAll<'a, T> where T: AsyncRead {
    input: Pin<&'a mut T>,
    result: String,
}

impl<'a, T> ReadAll<'a, T> where T: AsyncRead {
    pub fn new(input: Pin<&'a mut T>) -> ReadAll<'a, T> where T: AsyncRead {
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
            // Note the "as_mut()" call below.
            // Without it, the compiler complains that it cannot move the Pin<...> value.
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

// Makes a Future that reads all the bytes from `input` and returns them as a String.
fn read_all<T>(input: &mut T) -> ReadAll<T>
    where T: AsyncRead + std::marker::Unpin {
    ReadAll::new(Pin::new(input))
}

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
    let value = read_all(&mut async_readable).await.unwrap();
    println!("{:?}", value);
}

pub fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async_main());
}

// $ cargo run --bin async_read_future
// poll_read
// Poll::Pending
// poll_read
// read 3 bytes
// read "aaa"
// poll_read
// read 3 bytes
// read "bbb"
// poll_read
// Poll::Ready("aaabbb")
// "aaabbb"
