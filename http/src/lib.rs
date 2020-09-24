pub mod no_borrow_cursor;
pub mod buffer;
pub mod async_write_logger;
pub mod async_readable;
pub mod split_iterate;

pub fn escape_ascii(input: &[u8]) -> String {
    let mut result = String::new();
    for byte in input {
        for ascii_byte in std::ascii::escape_default(*byte) {
            result.push_str(std::str::from_utf8(&[ascii_byte]).unwrap());
        }
    }
    result
}

// use std::net::SocketAddr;
// use std::sync::Arc;
// use std::time::Duration;

// use tokio::net::TcpStream;

// use async_trait::async_trait;
// use logging::info;
// use logging::warn;

// #[async_trait]
// pub trait HttpSessionHandler {
//     async fn handle(&self, session: &mut HttpSession);
// }

// pub struct HttpSession {
//     pub tcp_stream: tokio::net::TcpStream,
//     addr: std::net::SocketAddr,
// }

// impl std::fmt::Debug for HttpSession {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // https://doc.rust-lang.org/std/fmt/struct.Formatter.html
//         f.debug_struct("HttpSession")
//             .field("addr", &self.addr)
//             .finish()
//     }
// }

// impl HttpSession {
// // pub fn shutdown(self) {
// //     if let Err(e) = self.tcp_stream.shutdown(std::net::Shutdown::Both) {
// //         warn!("Failed calling shutdown for tcp socket: {:?}", e);
// //     };
// // }
// }

// pub struct HttpServer {
//     pub socket_addr: std::net::SocketAddr,
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

// struct Handler {}

// #[async_trait]
// impl HttpSessionHandler for Handler {
//     async fn handle(&self, session: &mut HttpSession) {
//         info!("Got {:?}", session);
//         tokio::time::delay_for(Duration::from_secs(1)).await;
//     }
// }

