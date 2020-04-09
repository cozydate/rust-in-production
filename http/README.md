# http

Opinion: HTTP servers and clients like [`opinion.rs`](src/bin/opinion.rs) which uses [`lib.rs`](src/lib.rs).

That code does a lot of things.  Here are the things split into separate binaries:
- [`get.rs`](src/bin/get.rs)
- [`ipv4_and_ipv6.rs`](src/bin/ipv4_and_ipv6.rs)
- [`port.rs`](src/bin/port.rs)
- [`graceful_shutdown.rs`](src/bin/graceful_shutdown.rs)
- [`request_id.rs`](src/bin/request_id.rs)

More info:
- [Hyper web server based on tokio](https://hyper.rs)
- [Tokio asynchronous framework](https://tokio.rs)
- [Crate slog](https://crates.io/crates/slog-scope-futures)
- [Crate slog-scope-futures](https://crates.io/crates/slog-scope-futures)
