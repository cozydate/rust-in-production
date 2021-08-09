// use std::net::SocketAddr;
// use std::sync::Arc;
// use std::time::Duration;

// use tokio::net::TcpStream;

// use async_trait::async_trait;
// use logging::info;
// use logging::warn;

// // async fn http_get(url: &str) {
// //     let url: hyper::Uri = url.parse().unwrap();
// //     logging::info!("GET {}", url);
// //     let mut response = Client::new().get(url).await.unwrap();
// //     let _body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
// //     logging::info!("{} {:?}", response.status(), _body);
// // }

// #[async_trait]
// pub trait HttpSessionHandler {
//     async fn handle(&self, session: &mut HttpSession);
// }

// #[derive(Debug)]
// pub struct HttpSession {
//     tcp_stream: tokio::net::TcpStream,
//     addr: std::net::SocketAddr,
// }

// impl HttpSession {
// // pub fn shutdown(self) {
// //     if let Err(e) = self.tcp_stream.shutdown(std::net::Shutdown::Both) {
// //         warn!("Failed calling shutdown for tcp socket: {:?}", e);
// //     };
// // }
// }

// pub fn parse_env_var<T>(name: &str, default: T) -> T
//     where T: std::str::FromStr, <T as std::str::FromStr>::Err: std::fmt::Debug
// {
//     match std::env::var(name) {
//         Ok(s) =>
//             s.parse().expect(&format!("Failed parsing env var {}={:?}", name, s)),
//         Err(std::env::VarError::NotUnicode(oss)) =>
//             panic!("Failed parsing {}={:?} value as UTF-8", name, oss),
//         Err(std::env::VarError::NotPresent) =>
//             default,
//     }
// }

// pub struct HttpServer {}

// pub async fn wait_for_sigterm() {
//     // Handle TERM signal for running in Docker, Kubernetes, supervisord, etc.
//     // Also handle INT signal from CTRL-C in dev terminal.
//     use tokio::signal::unix::{signal, SignalKind};
//     let term_signal = signal(SignalKind::terminate())
//         .expect("Failed installing TERM signal handler");
//     let int_signal = signal(SignalKind::interrupt())
//         .expect("Failed installing INT signal handler");
//     // https://docs.rs/tokio/0.2.16/tokio/stream/trait.StreamExt.html#method.merge
//     use tokio::stream::StreamExt;
//     term_signal.merge(int_signal).next().await;
// }

// pub fn random_id(len: usize) -> String {
//     // Alphabet has 27 characters. Each randomly-selected character adds 4.75 bits of entropy.
//     // Selecting 8 with replacement yields a random string with 38 bits of entropy.
//     // At one request-per-second, duplicate request ids will occur once every 74 days, on average.
//     // https://en.wikipedia.org/wiki/Birthday_problem
//     // http://davidjohnstone.net/pages/hash-collision-probability
//     use rand::seq::IteratorRandom;
//     let mut rng = rand::thread_rng();
//     std::iter::repeat(())
//         .take(len)
//         .map(|_| "123456789CDFGHJKLMNPQRTVWXZ".chars().choose(&mut rng).unwrap())
//         .collect()
// }

// async fn handle_tcp_stream(tcp_stream: TcpStream, addr: SocketAddr, handler: Arc<dyn HttpSessionHandler + Send + Sync>) {
//     let logger = slog_scope::logger().new(
//         slog::o!("request_id" => random_id(8), "ip" => addr.ip().to_string()));
//     slog_scope_futures::SlogScope::new(&logger, async move {
//         logging::info!("Handling request");
//         if let Err(e) = tcp_stream.set_keepalive(Some(Duration::from_secs(60))) {
//             warn!("Failed setting keepalive on tcp socket: {:?}", e);
//         }
//         let mut http_session = HttpSession { tcp_stream, addr };
//         handler.handle(&mut http_session).await;
//     }).await;
// }

// async fn accept_loop(mut listener: tokio::net::TcpListener, handler: Arc<dyn HttpSessionHandler + Send + Sync>) {
//     info!("Starting accept loop");
//     loop {
//         match listener.accept().await {
//             Ok((tcp_stream, addr)) => {
//                 let handler_clone = handler.clone();
//                 tokio::spawn(async move {
//                     handle_tcp_stream(tcp_stream, addr, handler_clone).await;
//                 });
//             }
//             Err(e) => {
//                 warn!("Failed accepting connection from socket: {:?}", e);
//                 match e.kind() {
//                     // Do not sleep on connection error.
//                     std::io::ErrorKind::ConnectionAborted
//                     | std::io::ErrorKind::ConnectionRefused
//                     | std::io::ErrorKind::ConnectionReset => {}
//                     // Sleep on accept error.
//                     _ => {
//                         tokio::time::delay_for(Duration::from_secs(1)).await;
//                     }
//                 }
//             }
//         }
//     }
// }

// pub struct HttpServerBuilder {
//     all_interfaces: bool,
//     port: u16,
// }

// impl HttpServerBuilder {
//     pub fn new(port: u16) -> HttpServerBuilder {
//         HttpServerBuilder {
//             all_interfaces: false,
//             port,
//         }
//     }

//     pub fn all_interfaces(self) -> HttpServerBuilder {
//         HttpServerBuilder {
//             all_interfaces: true,
//             port: self.port,
//         }
//     }

//     pub async fn run(self, handler: Arc<dyn HttpSessionHandler + Send + Sync>) -> std::io::Result<HttpServer> {
//         let interface =
//             if self.all_interfaces {
//                 std::net::IpAddr::from(std::net::Ipv6Addr::UNSPECIFIED /* includes ipv4 */)
//             } else {
//                 std::net::IpAddr::from(std::net::Ipv4Addr::LOCALHOST)
//             };
//         let addr = std::net::SocketAddr::from((interface, self.port));
//         info!("Listening for TCP connections on {}", addr);
//         let listener = tokio::net::TcpListener::bind(&addr).await?;
//         tokio::spawn(async move { accept_loop(listener, handler).await; });
//         Ok(HttpServer {})
//     }
// }

// struct Handler {}

// #[async_trait]
// impl HttpSessionHandler for Handler {
//     async fn handle(&self, session: &mut HttpSession) {
//         info!("Got {:?}", session);
//         tokio::time::delay_for(Duration::from_secs(1)).await;
//     }
// }

// pub async fn async_main() -> () {
//     let port: u16 = parse_env_var("PORT", 1690);
//     let _http_server = HttpServerBuilder::new(port)
//         .all_interfaces()
//         // TODO(mleonhard) Make this take an async closure once they are stable, https://github.com/rust-lang/rust/issues/62290
//         .run(Arc::new(Handler {}))
//         .await.unwrap();
//     tokio::net::TcpStream::connect("127.0.0.1:1690").await.unwrap();
//     tokio::net::TcpStream::connect("::1:1690").await.unwrap();
//     // // Test accept error handling.
//     // // $ (cargo build --bin opinion && ulimit -n 26 && DEV_LOG_FORMAT=plain target/debug/opinion)
//     // // ...
//     // // 2020-05-26T23:17:40.179-07:00 WARN Failed accepting connection from socket: Os { code: 24, kind: Other, message: "Too many open files" }
//     // let mut tcp_streams: Vec<TcpStream> = Vec::new();
//     // for _n in 1..25 {
//     //     match tokio::net::TcpStream::connect("127.0.0.1:1690").await {
//     //         Ok(tcp_stream) => { tcp_streams.push(tcp_stream); }
//     //         Err(e) => { warn!("{:?}", e); }
//     //     }
//     // }

//     // $ DEV_LOG_FORMAT=compact cargo run --bin handler_trait
//     // 2020-05-08T02:29:18.461-07:00 INFO Listening for TCP connections on [::]:1690
//     // 2020-05-08T02:29:18.461-07:00 INFO Got HttpSession { tcp_stream: TcpStream { addr: V6([::ffff:127.0.0.1]:1690), peer: V6([::ffff:127.0.0.1]:58762), fd: 9 }, addr: V6([::ffff:127.0.0.1]:58762) }
//     // 2020-05-08T02:29:18.462-07:00 INFO Got HttpSession { tcp_stream: TcpStream { addr: V6([::1]:1690), peer: V6([::1]:58763), fd: 8 }, addr: V6([::1]:58763) }
// }

// pub fn main() {
//     let _global_logger_guard = logging::configure("info").unwrap();
//     let mut runtime = tokio::runtime::Builder::new()
//         .threaded_scheduler()
//         .enable_all()
//         .build()
//         .unwrap();
//     runtime.block_on(async_main());
//     // Drops waiting tasks.  Waits for all busy tasks to await and drops them.  Gives up after timeout.
//     runtime.shutdown_timeout(Duration::from_secs(3));
// }

// use tokio::io::AsyncWriteExt;
// if let Err(e) = tcp_stream.write_all(b"greeting").await {
//     println!("WARN client write error: {:?}", e);
//     return;
// }
