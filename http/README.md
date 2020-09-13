# http

Opinion: Write HTTP servers and clients like [`opinion.rs`](src/bin/opinion.rs) which uses [`lib.rs`](src/lib.rs).

That code does a lot of things.  Here are the things split into separate binaries:
- [`runtime_shutdown.rs`](src/bin/runtime_shutdown.rs) - Safely shut down a Tokio async task executor
- [`tcp.rs`](src/bin/tcp.rs) - Receive TCP connections
- [`port.rs`](src/bin/port.rs) - Read the PORT environment variable
- [`ipv4_and_ipv6.rs`](src/bin/ipv4_and_ipv6.rs) - Receive IPv4 and IPv6 TCP conenctions at the same time
- [`concurrent_connections.rs`](src/bin/concurrent_connections.rs) - Handle multiple connections at the same time
- [`handle_conn_fns.rs`](src/bin/handle_conn_fns.rs) - Pass connection handler functions
- [`graceful_shutdown.rs`](src/bin/graceful_shutdown.rs) - Shutdown a server that is serving clients
- [`tls.rs`](src/bin/tls.rs) - Use TLS with certificate pinning

- [`get.rs`](src/bin/get.rs)
- [`streaming_response.rs`](src/bin/streaming_response.rs)
- [`request_id.rs`](src/bin/request_id.rs)


https://crates.io/crates/webpki

https://rust-unofficial.github.io/too-many-lists/third.html

# Certificates

Rust's TLS client (`rustls`) accepts only certificates with Subject Alternative Name (SAN) values.
There are two ways to make a certificate or certificate signing request with SAN values:
1. Put the values in an `openssl.cfg` file and pass the `-config openssl.cfg` parameter to the `openssl` command.
   This works on MacOS 10.15.6 which has an `openssl` command from LibreSSL 2.8.3.
   ```
   # openssl.cfg
   [req]
   distinguished_name=dn
   x509_extensions=ext
   [ dn ]
   CN=localhost
   [ ext ]
   subjectAltName = @alt_names
   [alt_names]
   DNS.1 = localhost
   IP.1 = 127.0.0.1
   IP.2 = ::1
   ```
   See:
   - [PKCS#10 certificate request and certificate generating utility](https://www.openssl.org/docs/man1.1.1/man1/req.html)
   - [X509 V3 certificate extension configuration format](https://www.openssl.org/docs/man1.1.1/man5/x509v3_config.html)
2. Use a newer version of OpenSSL which accepts multiple `-addext 'subjectAltName = DNS:localhost'` parameters.
   See:
   - [Provide subjectAltName to openssl directly on the command line](https://security.stackexchange.com/a/183973).

Generate `localhost.key` and self-signed `localhost.cert` with:
```
openssl req -newkey rsa:2048 -new -nodes -x509 -days 3650 -out localhost.cert -keyout localhost.key -subj '/CN=localhost' -config openssl.cfg
```

Print out a certificate:
```
openssl x509 -in server1.cert -noout -text
```


More info:
- [Hyper web server based on tokio](https://hyper.rs)
- [Tokio asynchronous framework](https://tokio.rs)
- [Crate hyper-native-tls](https://crates.io/crates/hyper-native-tls)
- [Crate slog](https://crates.io/crates/slog)
- [Crate slog-scope-futures](https://crates.io/crates/slog-scope-futures)
- [IntelliJ - Code completion not working for some tokio modules](https://github.com/intellij-rust/intellij-rust/issues/4706#issuecomment-608987405)
- [Async/Await - The challenges besides syntax - Cancellation](https://gist.github.com/Matthias247/ffc0f189742abf6aa41a226fe07398a8)
