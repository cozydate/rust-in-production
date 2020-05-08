# http

Opinion: Write HTTP servers and clients like [`opinion.rs`](src/bin/opinion.rs) which uses [`lib.rs`](src/lib.rs).

That code does a lot of things.  Here are the things split into separate binaries:
- [`tcp.rs`](src/bin/tcp.rs)
- [`handler_trait.rs.rs`](src/bin/handler_trait.rs.rs)


- [`graceful_shutdown.rs`](src/bin/graceful_shutdown.rs)
- [`runtime_shutdown.rs`](src/bin/runtime_shutdown.rs)
- [`get.rs`](src/bin/get.rs)
- [`streaming_response.rs`](src/bin/streaming_response.rs)
- [`ipv4_and_ipv6.rs`](src/bin/ipv4_and_ipv6.rs)
- [`port.rs`](src/bin/port.rs)
- [`request_id.rs`](src/bin/request_id.rs)

More info:
- [Hyper web server based on tokio](https://hyper.rs)
- [Tokio asynchronous framework](https://tokio.rs)
- [Crate slog](https://crates.io/crates/slog)
- [Crate slog-scope-futures](https://crates.io/crates/slog-scope-futures)
- [IntelliJ - Code completion not working for some tokio modules](https://github.com/intellij-rust/intellij-rust/issues/4706#issuecomment-608987405)
- [Async/Await - The challenges besides syntax - Cancellation](https://gist.github.com/Matthias247/ffc0f189742abf6aa41a226fe07398a8)
