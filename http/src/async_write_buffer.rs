// use std::task::Context;
//
// use tokio::io::{AsyncWrite, Error};
// use tokio::macros::support::{Pin, Poll};
//
// use crate::escape_ascii;
//
// /// AsyncWriteBuffer supports writes and async-writes to its internal buffer.
// pub struct AsyncWriteBuffer<'a, T> where T: tokio::io::AsyncWrite + std::marker::Unpin {
//     mem: &'a mut [u8],
//     output: &'a mut T,
//     write_index: usize,
// }
//
// enum PollWriteBufferedBytesResult {
//     Err(Error),
//     Pending,
//     Ok,
// }
//
// impl<'a, T> AsyncWriteBuffer<'a, T> where T: tokio::io::AsyncWrite + std::marker::Unpin {
//     pub fn new(mem: &'a mut [u8], output: &'a mut T) -> AsyncWriteBuffer<'a, T> {
//         AsyncWriteBuffer {
//             mem,
//             output,
//             write_index: 0,
//         }
//     }
//
//     fn poll_write_buffered_bytes(self: Pin<&mut Self>, cx: &mut Context<'_>) -> PollWriteBufferedBytesResult {
//         while self.write_index > 0 {
//             match self.output.poll_write(cx, &self.mem[..self.write_index]) {
//                 Poll::Pending => {
//                     // self.output.poll_write() arranged to wake the task again.
//                     return PollWriteBufferedBytesResult::Pending;
//                 }
//                 Poll::Ready(Err(e)) => {
//                     return PollWriteBufferedBytesResult::Err(e);
//                 }
//                 Poll::Ready(Ok(bytes_written)) => {
//                     if bytes_written == 0 {
//                         return PollWriteBufferedBytesResult::Err(std::io::Error::new(
//                             std::io::ErrorKind::ConnectionReset, "connection reset"));
//                     }
//                     // Some bytes written.
//                     self.mem.copy_within(bytes_written..self.write_index, 0);
//                     self.write_index -= bytes_written;
//                 }
//             }
//         }
//         PollWriteBufferedBytesResult::Ok
//     }
// }
//
// impl<'a, T> std::io::Write for AsyncWriteBuffer<'a, T> where T: tokio::io::AsyncWrite + std::marker::Unpin {
//     fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
//         let space: usize = self.mem.len() - self.write_index;
//         if space < data.len() {
//             return Err(std::io::Error::new(
//                 std::io::ErrorKind::InvalidData, "AsyncWriteBuffer too full"));
//         }
//         let write_end_index = self.write_index + data.len();
//         *self.mem[self.write_index..write_end_index] = *data;
//         self.write_index = write_end_index;
//         Ok(data.len())
//     }
//
//     fn flush(&mut self) -> std::io::Result<()> {
//         panic!("AsyncWriteBuffer::flush() is not implemented. Use async_flush() instead.");
//     }
// }
//
// impl<'a, T> AsyncWrite for AsyncWriteBuffer<'a, T> where T: tokio::io::AsyncWrite + std::marker::Unpin {
//     fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Error>> {
//         println!("AsyncWriteBuffer::poll_write mem={:?} buf={:?}",
//                  escape_ascii(&self.mem[..self.write_index]),
//                  escape_ascii(buf));
//         match self.poll_write_buffered_bytes(cx) {
//             PollWriteBufferedBytesResult::Pending => Poll::Pending,
//             PollWriteBufferedBytesResult::Err(e) => Poll::Ok(Err(e)),
//             PollWriteBufferedBytesResult::Ok => {
//                 self.output.poll_write(cx, buf)
//             }
//         }
//     }
//
//     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
//         println!("AsyncWriteBuffer::poll_flush mem={:?}",
//                  escape_ascii(&self.mem[..self.write_index]));
//         match self.poll_write_buffered_bytes(cx) {
//             PollWriteBufferedBytesResult::Pending => Poll::Pending,
//             PollWriteBufferedBytesResult::Err(e) => Poll::Ok(Err(e)),
//             PollWriteBufferedBytesResult::Ok => {
//                 self.output.poll_flush(cx)
//             }
//         }
//     }
//
//     fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
//         println!("AsyncWriteLogger::poll_shutdown");
//         Poll::Ready(Ok(()))
//     }
// }
