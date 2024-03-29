// TODO(mleonhard) Add a blocking version of read_delimited.
// TODO(mleonhard) Move Async* code under #tokio feature, to make the crate usable without tokio.
// TODO(mleonhard) Move to its own directory.
// TODO(mleonhard) Add a README.md.
// TODO(mleonhard) Add runnable examples.
// TODO(mleonhard) Figure out how to publish to crates.io.
// TODO(mleonhard) Add an async-std feature?
// TODO(mleonhard) See if there's a good way to let users specify buffer length.
// TODO(mleonhard) Refactor read_delimited() a bit to make it easier to understand.

use core::pin::Pin;
use core::task::{Context, Poll};

/// The number of bytes that can be stored in an FixedBuf.
pub const BUFFER_LEN: usize = 4 * 1024;

/// FixedBuf is a byte buffer that holds a few KB.
/// You can write bytes to it and then read them back.
///
/// It supports tokio's AsyncRead and AsyncWrite.
///
/// Use `read_delimited` to read a socket, searching for a delimiter,
/// and buffer unused bytes for the next read.
/// This works like `tokio::io::AsyncBufReadExt::read_until()`
/// and `read_line()`, but it uses a fixed sized buffer so network peers cannot
/// OOM the process.
///
/// The buffer comes from the stack.  Put it in a `Box` to put it on the heap.
///
/// It is not a circular buffer.  You can call `shift()` periodically to
/// move unread bytes to the front of the buffer.
pub struct FixedBuf {
    buf: [u8; BUFFER_LEN],
    write_index: usize,
    read_index: usize,
}

impl FixedBuf {
    /// Makes a new buffer with a few KB of internal memory.
    ///
    /// Allocates on the stack by default.  Put it in a `Box` to use the heap.
    pub fn new() -> FixedBuf {
        FixedBuf {
            buf: [0; BUFFER_LEN],
            write_index: 0,
            read_index: 0,
        }
    }

    /// Makes a new FixedBuf which uses the provided memory.
    pub fn new_with_mem(buf: [u8; BUFFER_LEN]) -> FixedBuf {
        FixedBuf {
            buf,
            write_index: 0,
            read_index: 0,
        }
    }

    /// Drops the struct and returns its internal memory.
    pub fn internal_mem(self) -> [u8; BUFFER_LEN] {
        self.buf
    }

    /// Writes `s` into the buffer, after any unread bytes.
    /// Panics if the buffer doesn't have enough free space at the end for the whole string.
    pub fn append(&mut self, s: &str) {
        std::io::Write::write(self, s.as_bytes()).unwrap();
    }

    /// Writes `s` into the buffer, after any unread bytes.
    /// Returns None if the buffer doesn't have enough free space at the end for the whole string.
    pub fn try_append(&mut self, s: &str) -> Option<()> {
        std::io::Write::write(self, s.as_bytes()).ok().map(|_| ())
    }

    /// Returns a mutable slice of the writable part of the buffer.
    /// Modify bytes at the beginning of the slice
    /// and then call `wrote(usize)` to commit those bytes into the buffer.
    /// The bytes are then available for reading.
    ///
    /// Returns None when the end of the buffer is full.  See `shift()`.
    pub fn writable(&mut self) -> Option<&mut [u8]> {
        if self.write_index >= self.buf.len() {
            // buf ran out of space.
            return None;
        }
        Some(&mut self.buf[self.write_index..])
    }

    /// Commit bytes into the buffer.
    /// Call this after writing to the front of the `writable()` slice.
    pub fn wrote(&mut self, num_bytes: usize) {
        if num_bytes == 0 {
            return;
        }
        let new_write_index = self.write_index + num_bytes;
        if new_write_index > self.buf.len() {
            panic!("write would overflow");
        }
        self.write_index = new_write_index;
    }

    /// Returns the slice of readable bytes in the buffer.
    pub fn readable(&self) -> &[u8] {
        &self.buf[self.read_index..self.write_index]
    }

    /// Consume readable bytes.
    /// Call this after reading from the front of the `readable()` slice.
    pub fn consume(&mut self, num_bytes: usize) {
        let new_read_index = self.read_index + num_bytes;
        if new_read_index > self.write_index {
            panic!("read would underflow");
        }
        self.read_index = new_read_index;
        if self.read_index == self.write_index {
            self.read_all();
        }
    }

    /// Consume and return all readable bytes.
    /// The buffer becomes empty and subsequent writes can fill the whole buffer.
    pub fn read_all(&mut self) -> &[u8] {
        let start = self.read_index;
        let end = self.write_index;
        // All data has been read.  Reset the buffer.
        self.write_index = 0;
        self.read_index = 0;
        &self.buf[start..end]
    }

    /// Reads from a tokio `AsyncRead` struct into the buffer until it finds `delim`.
    /// Returns the slice up until `delim`.
    /// Consumes the returned bytes and `delim`.
    /// Leaves unused bytes in the buffer.
    ///
    /// If the buffer already contains `delim`,
    /// returns the data immediately without reading from `input`.
    ///
    /// If the buffer does not already contain `delim`, calls `shift()` before
    /// reading from `input`.
    ///
    /// Returns Err(InvalidData) if the end of the buffer fills up before `delim` is found.
    /// See `shift()`.
    pub async fn read_delimited<'b, T>(
        &'b mut self,
        input: &'b mut T,
        delim: &[u8],
    ) -> std::io::Result<&'b [u8]>
    where
        T: tokio::io::AsyncRead + std::marker::Unpin,
    {
        loop {
            if let Some(delim_index) = self
                .readable()
                .windows(delim.len())
                .enumerate()
                .filter(|(_index, window)| *window == delim)
                .map(|(index, _window)| index)
                .next()
            {
                let result_start = self.read_index;
                let result_end = self.read_index + delim_index;
                self.consume(delim_index + delim.len());
                return Ok(&self.buf[result_start..result_end]);
            }
            self.shift();
            let writable = self.writable().ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "end of buffer full",
            ))?;
            let num_bytes_read = tokio::io::AsyncReadExt::read(input, writable).await?;
            if num_bytes_read == 0 {
                if self.read_index == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "eof with no data read",
                    ));
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "eof before delim read",
                ));
            }
            self.wrote(num_bytes_read);
        }
    }

    /// shift() recovers buffer space.
    ///
    /// The buffer is not circular.
    /// After you read bytes, the space at the beginning of the buffer is unused.
    /// Call `shift()` to move unread data to the beginning of the buffer and recover the space.
    /// This makes the free space available for writes, which go at the end of the buffer.
    pub fn shift(&mut self) {
        if self.read_index == 0 {
            return;
        }
        self.buf.copy_within(self.read_index..self.write_index, 0);
        self.write_index -= self.read_index;
        self.read_index = 0;
    }
}

impl std::io::Write for FixedBuf {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        let writable = self.writable().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "end of buffer full",
        ))?;
        if writable.len() < data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not enough free space in buffer",
            ));
        }
        let dest = &mut writable[..data.len()];
        dest.copy_from_slice(data);
        self.wrote(data.len());
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::io::Read for FixedBuf {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let readable = self.readable();
        let len = core::cmp::min(buf.len(), readable.len());
        if len == 0 {
            return Ok(0);
        }
        let src = &readable[..len];
        let dest = &mut buf[..len];
        dest.copy_from_slice(src);
        self.consume(len);
        Ok(len)
    }
}

impl tokio::io::AsyncWrite for FixedBuf {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Poll::Ready(std::io::Write::write(self.get_mut(), buf))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl tokio::io::AsyncRead for FixedBuf {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Poll::Ready(std::io::Read::read(self.get_mut(), buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructors() {
        let mut buf = FixedBuf::new();
        buf.append("abc");
        assert_eq!("abc", crate::escape_ascii(buf.readable()));
        let mem = buf.internal_mem();
        buf = FixedBuf::new_with_mem(mem);
        assert_eq!("", crate::escape_ascii(buf.readable()));
        buf.wrote(3);
        assert_eq!("abc", crate::escape_ascii(buf.read_all()));
        assert_eq!("", crate::escape_ascii(buf.readable()));
    }


    #[test]
    fn empty() {
        let mut buf = FixedBuf::new();
        assert_eq!("", crate::escape_ascii(buf.readable()));
        assert_eq!("", crate::escape_ascii(buf.read_all()));
        buf.shift();
        assert_eq!("", crate::escape_ascii(buf.readable()));
        assert_eq!("", crate::escape_ascii(buf.read_all()));
    }

    #[test]
    fn test_append() {
        let mut buf = FixedBuf::new();
        buf.append("abc");
        assert_eq!("abc", crate::escape_ascii(buf.readable()));
        buf.append("def");
        assert_eq!("abcdef", crate::escape_ascii(buf.readable()));
        buf.append(&"g".repeat(BUFFER_LEN - 6));
    }

    #[test]
    #[should_panic]
    fn test_append_buffer_full() {
        let mut buf = FixedBuf::new();
        buf.append(&"c".repeat(BUFFER_LEN + 1));
    }

    #[test]
    fn test_try_append() {
        let mut buf = FixedBuf::new();
        buf.try_append("a").unwrap();
        buf.append("b");
        assert_eq!("ab", crate::escape_ascii(buf.readable()));
        let many_cs = "c".repeat(BUFFER_LEN - 3);
        buf.try_append(&many_cs).unwrap();
        buf.try_append("d").unwrap();
        assert_eq!(
            "ab".to_string() + &many_cs + "d",
            crate::escape_ascii(buf.readable())
        );
        assert_eq!(None, buf.try_append("e"));
    }

    #[test]
    fn test_writable_and_wrote() {
        let mut buf = FixedBuf::new();
        assert_eq!(BUFFER_LEN, buf.writable().unwrap().len());
        buf.writable().unwrap()[0] = 'a' as u8;
        buf.wrote(1);
        assert_eq!("a", crate::escape_ascii(buf.readable()));
        let many_bs = "b".repeat(BUFFER_LEN - 1);
        assert_eq!(many_bs.len(), buf.writable().unwrap().len());
        buf.writable().unwrap().copy_from_slice(many_bs.as_bytes());
        buf.wrote(many_bs.len());
        assert_eq!(
            "a".to_string() + &many_bs,
            crate::escape_ascii(buf.readable())
        );
        assert_eq!(None, buf.writable());
    }

    #[test]
    #[should_panic]
    fn test_wrote_too_much() {
        let mut buf = FixedBuf::new();
        buf.wrote(BUFFER_LEN + 1);
    }

    #[test]
    fn test_readable_and_read() {
        let mut buf = FixedBuf::new();
        assert_eq!("", crate::escape_ascii(buf.readable()));
        buf.append("abc");
        assert_eq!("abc", crate::escape_ascii(buf.readable()));
        buf.consume(1);
        assert_eq!("bc", crate::escape_ascii(buf.readable()));
        buf.consume(2);
        assert_eq!("", crate::escape_ascii(buf.readable()));
        buf.append("d");
        assert_eq!("d", crate::escape_ascii(buf.readable()));
        buf.consume(1);
        assert_eq!("", crate::escape_ascii(buf.readable()));
    }

    #[test]
    #[should_panic]
    fn test_read_too_much() {
        let mut buf = FixedBuf::new();
        buf.append("a");
        buf.consume(2);
    }

    #[test]
    fn test_read_all() {
        let mut buf = FixedBuf::new();
        assert_eq!("", crate::escape_ascii(buf.read_all()));
        buf.append("abc");
        assert_eq!("abc", crate::escape_ascii(buf.read_all()));
        buf.append("def");
        assert_eq!("def", crate::escape_ascii(buf.read_all()));
        assert_eq!("", crate::escape_ascii(buf.read_all()));
    }

    #[tokio::test]
    async fn test_read_delimited_empty() {
        let mut buf = FixedBuf::new();
        let mut input = tokio::io::stream_reader(tokio::stream::empty::<std::io::Result<&[u8]>>());
        assert_eq!(
            std::io::ErrorKind::NotFound,
            buf.read_delimited(&mut input, b"b")
                .await
                .unwrap_err()
                .kind()
        );
    }

    #[tokio::test]
    async fn test_read_delimited_not_found_eof() {
        let mut buf = FixedBuf::new();
        let mut input = tokio::io::stream_reader(tokio::stream::iter(vec![Ok("abc".as_bytes())]));
        assert_eq!(
            std::io::ErrorKind::NotFound,
            buf.read_delimited(&mut input, b"d")
                .await
                .unwrap_err()
                .kind()
        );
        buf.read_all();
    }

    #[tokio::test]
    async fn test_read_delimited_not_found_buffer_almost_full() {
        let mut buf = FixedBuf::new();
        let many_bs = "b".repeat(BUFFER_LEN - 1);
        let mut input = tokio::io::stream_reader(tokio::stream::iter(vec![Ok(many_bs.as_bytes())]));
        assert_eq!(
            std::io::ErrorKind::NotFound,
            buf.read_delimited(&mut input, b"d")
                .await
                .unwrap_err()
                .kind()
        );
    }

    #[tokio::test]
    async fn test_read_delimited_not_found_buffer_full() {
        let mut buf = FixedBuf::new();
        let many_bs = "b".repeat(BUFFER_LEN);
        let mut input = tokio::io::stream_reader(tokio::stream::iter(vec![Ok(many_bs.as_bytes())]));
        assert_eq!(
            std::io::ErrorKind::InvalidData,
            buf.read_delimited(&mut input, b"d")
                .await
                .unwrap_err()
                .kind()
        );
    }

    #[tokio::test]
    async fn test_read_delimited_found() {
        let mut buf = FixedBuf::new();
        let mut input = tokio::io::stream_reader(tokio::stream::iter(vec![Ok("abc".as_bytes())]));
        assert_eq!(
            "ab",
            crate::escape_ascii(buf.read_delimited(&mut input, b"c").await.unwrap())
        );
    }

    #[tokio::test]
    async fn test_read_delimited_found_with_leftover() {
        let mut buf = FixedBuf::new();
        let mut input =
            tokio::io::stream_reader(tokio::stream::iter(vec![Ok("abcdef".as_bytes())]));
        assert_eq!(
            "ab",
            crate::escape_ascii(buf.read_delimited(&mut input, b"c").await.unwrap())
        );
        assert_eq!("def", crate::escape_ascii(buf.read_all()));
    }

    struct AsyncReadableThatPanics;

    impl tokio::io::AsyncRead for AsyncReadableThatPanics {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            mut _buf: &mut [u8],
        ) -> Poll<Result<usize, std::io::Error>> {
            panic!("AsyncReadableThatPanics::poll_read called");
        }
    }

    #[tokio::test]
    async fn test_read_delimited_already_in_buffer() {
        let mut buf = FixedBuf::new();
        buf.append("abc");
        let mut input = AsyncReadableThatPanics {};
        assert_eq!(
            "ab",
            crate::escape_ascii(buf.read_delimited(&mut input, b"c").await.unwrap())
        );

        buf.append("aaxbbx");
        assert_eq!(
            "aa",
            crate::escape_ascii(buf.read_delimited(&mut input, b"x").await.unwrap())
        );
        assert_eq!(
            "bb",
            crate::escape_ascii(buf.read_delimited(&mut input, b"x").await.unwrap())
        );
    }

    #[test]
    fn test_std_io_write() {
        let mut buf = FixedBuf::new();
        std::io::Write::write(&mut buf, b"abc").unwrap();
        assert_eq!("abc", crate::escape_ascii(buf.readable()));
        std::io::Write::write(&mut buf, b"def").unwrap();
        assert_eq!("abcdef", crate::escape_ascii(buf.readable()));
        buf.consume(1);
        std::io::Write::write(&mut buf, b"g").unwrap();
        assert_eq!("bcdefg", crate::escape_ascii(buf.readable()));
        std::io::Write::write(&mut buf, "h".repeat(BUFFER_LEN - 8).as_bytes()).unwrap();
        std::io::Write::write(&mut buf, b"i").unwrap();
        assert_eq!(
            std::io::ErrorKind::InvalidData,
            std::io::Write::write(&mut buf, b"def").unwrap_err().kind()
        );
    }

    #[tokio::test]
    async fn test_async_write() {
        let mut buf = FixedBuf::new();
        tokio::io::AsyncWriteExt::write_all(&mut buf, b"abc")
            .await
            .unwrap();
        assert_eq!("abc", crate::escape_ascii(buf.readable()));
        tokio::io::AsyncWriteExt::write_all(&mut buf, b"def")
            .await
            .unwrap();
        assert_eq!("abcdef", crate::escape_ascii(buf.readable()));
        buf.consume(1);
        tokio::io::AsyncWriteExt::write_all(&mut buf, b"g")
            .await
            .unwrap();
        assert_eq!("bcdefg", crate::escape_ascii(buf.readable()));
        tokio::io::AsyncWriteExt::write_all(&mut buf, "h".repeat(BUFFER_LEN - 8).as_bytes())
            .await
            .unwrap();
        tokio::io::AsyncWriteExt::write_all(&mut buf, b"i")
            .await
            .unwrap();
        assert_eq!(
            std::io::ErrorKind::InvalidData,
            tokio::io::AsyncWriteExt::write_all(&mut buf, b"def")
                .await
                .unwrap_err()
                .kind()
        );
    }

    #[test]
    fn test_std_io_read() {
        let mut buf = FixedBuf::new();
        let mut data: [u8; BUFFER_LEN] = ['.' as u8; BUFFER_LEN];
        assert_eq!(0, std::io::Read::read(&mut buf, &mut data).unwrap());
        assert_eq!("..........", crate::escape_ascii(&data[..10]));
        buf.append("abc");
        assert_eq!(3, std::io::Read::read(&mut buf, &mut data).unwrap());
        assert_eq!("abc.......", crate::escape_ascii(&data[..10]));
        assert_eq!(0, std::io::Read::read(&mut buf, &mut data).unwrap());
        let many_bs = "b".repeat(BUFFER_LEN);
        buf.append(&many_bs);
        assert_eq!(
            BUFFER_LEN,
            std::io::Read::read(&mut buf, &mut data).unwrap()
        );
        assert_eq!(many_bs, crate::escape_ascii(&data[..]));
        assert_eq!(0, std::io::Read::read(&mut buf, &mut data).unwrap());
    }

    #[tokio::test]
    async fn test_async_read() {
        let mut buf = FixedBuf::new();
        let mut data: [u8; BUFFER_LEN] = ['.' as u8; BUFFER_LEN];
        assert_eq!(
            0,
            tokio::io::AsyncReadExt::read(&mut buf, &mut data)
                .await
                .unwrap()
        );
        assert_eq!("..........", crate::escape_ascii(&data[..10]));
        buf.append("abc");
        assert_eq!(
            3,
            tokio::io::AsyncReadExt::read(&mut buf, &mut data)
                .await
                .unwrap()
        );
        assert_eq!("abc.......", crate::escape_ascii(&data[..10]));
        assert_eq!(
            0,
            tokio::io::AsyncReadExt::read(&mut buf, &mut data)
                .await
                .unwrap()
        );
        let many_bs = "b".repeat(BUFFER_LEN);
        buf.append(&many_bs);
        assert_eq!(
            BUFFER_LEN,
            tokio::io::AsyncReadExt::read(&mut buf, &mut data)
                .await
                .unwrap()
        );
        assert_eq!(many_bs, crate::escape_ascii(&data[..]));
        assert_eq!(
            0,
            tokio::io::AsyncReadExt::read(&mut buf, &mut data)
                .await
                .unwrap()
        );
    }
}
