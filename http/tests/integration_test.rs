use std::borrow::BorrowMut;
use std::sync::Arc;
use std::time::Duration;

use ::function_name::named;

use async_trait::async_trait;
use http::{HttpServerBuilder, HttpSession, HttpSessionHandler};
use logging::info;


// https://stackoverflow.com/a/63190858
// #[test]
// fn test_cli() {
//     let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
//     path.push("target/debug/passman");
//     let output = Command::new(path)
//         .arg("--an-arg")
//         .output()
//         .expect("Failed to execute command");
//
//     assert_eq!(&output.stdout[..], b"Hello, world!\n");
// }

// use std::net::SocketAddr;
// use tokio::net::TcpStream;
// use logging::warn;

struct Handler {}

#[async_trait]
impl HttpSessionHandler for Handler {
    async fn handle(&self, session: &mut HttpSession) {
        info!("Handler::handle() {:?}", session);
        use tokio::io::AsyncWriteExt;
        tokio::io::AsyncWriteExt::write_all(&mut session.tcp_stream, b"hello").await.unwrap();
    }
}

#[test]
#[named]
fn test_port() {
    logging::configure_for_test("info").unwrap();
    // logging::configure_for_test("info").unwrap();
    tokio_test::block_on(logging::task_scope(function_name!(), async {
        let _http_server = HttpServerBuilder::new()
            .localhost()
            .port(24854)
            .run(Arc::new(Handler {})).await.unwrap();
        use tokio::io::AsyncReadExt;
        let mut response = String::new();
        tokio::io::AsyncReadExt::read_to_string(
            tokio::net::TcpStream::connect("127.0.0.1:24854").await.unwrap().borrow_mut(),
            response.borrow_mut()).await.unwrap();
        assert_eq!("hello", response);
    }));
}

// #[test]
// #[named]
// fn test_ipv4_and_ipv6() {
//     logging::configure_for_test("info").unwrap();
//     tokio_test::block_on(logging::task_scope(function_name!(), async {
//         let builder = HttpServerBuilder::new();
//         info!("{:?}", builder);
//         let builder = builder.any_port();
//         info!("{:?}", builder);
//         let http_server = builder.run(Arc::new(Handler {})).await.unwrap();
//         tokio::net::TcpStream::connect(("127.0.0.1", http_server.socket_addr.port())).await.unwrap();
//         tokio::net::TcpStream::connect(("::1", http_server.socket_addr.port())).await.unwrap();
//         tokio::time::delay_for(Duration::from_millis(100)).await;
//     }));
// }
