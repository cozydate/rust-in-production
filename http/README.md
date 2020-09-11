# http

Opinion: Write HTTP servers and clients like [`opinion.rs`](src/bin/opinion.rs) which uses [`lib.rs`](src/lib.rs).

That code does a lot of things.  Here are the things split into separate binaries:
- [`runtime_shutdown.rs`](src/bin/runtime_shutdown.rs) - Safely shut down a Tokio server
- [`tcp.rs`](src/bin/tcp.rs) - Receive TCP connections
- [`port.rs`](src/bin/port.rs) - Read the PORT environment variable
- [`ipv4_and_ipv6.rs`](src/bin/ipv4_and_ipv6.rs) - Receive IPv4 and IPv6 TCP conenctions at the same time
- [`concurrent_connections.rs`](src/bin/concurrent_connections.rs) - Handle multiple connections at the same time
- [`handle_conn_fns.rs`](src/bin/handle_conn_fns.rs) - Pass connection handler functions
- [`graceful_shutdown.rs`](src/bin/graceful_shutdown.rs) - Shutdown a server that is serving clients

- [`get.rs`](src/bin/get.rs)
- [`streaming_response.rs`](src/bin/streaming_response.rs)
- [`request_id.rs`](src/bin/request_id.rs)


https://crates.io/crates/webpki

https://rust-unofficial.github.io/too-many-lists/third.html


More info:
- [Hyper web server based on tokio](https://hyper.rs)
- [Tokio asynchronous framework](https://tokio.rs)
- [Crate hyper-native-tls](https://crates.io/crates/hyper-native-tls)
- [Crate slog](https://crates.io/crates/slog)
- [Crate slog-scope-futures](https://crates.io/crates/slog-scope-futures)
- [IntelliJ - Code completion not working for some tokio modules](https://github.com/intellij-rust/intellij-rust/issues/4706#issuecomment-608987405)
- [Async/Await - The challenges besides syntax - Cancellation](https://gist.github.com/Matthias247/ffc0f189742abf6aa41a226fe07398a8)
