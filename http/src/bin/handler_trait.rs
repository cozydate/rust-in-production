use std::sync::Arc;

use async_trait::async_trait;
use logging::info;

#[async_trait]
pub trait HttpSessionHandler {
    async fn handle(&self, session: HttpSession);
}

#[derive(Debug)]
pub struct HttpSession {
    tcp_stream: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
}

pub struct HttpServer {}

pub struct HttpServerBuilder {
    all_interfaces: bool,
    port: u16,
}

impl HttpServerBuilder {
    pub fn new(port: u16) -> HttpServerBuilder {
        HttpServerBuilder {
            all_interfaces: false,
            port,
        }
    }

    pub fn all_interfaces(self) -> HttpServerBuilder {
        HttpServerBuilder {
            all_interfaces: true,
            port: self.port,
        }
    }

    pub async fn run(self, handler_fn: Arc<dyn HttpSessionHandler + Send + Sync>) -> std::io::Result<HttpServer> {
        let interface =
            if self.all_interfaces {
                std::net::IpAddr::from(std::net::Ipv6Addr::UNSPECIFIED /* includes ipv4 */)
            } else {
                std::net::IpAddr::from(std::net::Ipv4Addr::LOCALHOST)
            };
        let addr = std::net::SocketAddr::from((interface, self.port));
        info!("Listening for TCP connections on {}", addr);
        let mut listener = tokio::net::TcpListener::bind(&addr).await?;
        tokio::spawn(async move {
            loop {
                let (tcp_stream, addr) = listener.accept().await.unwrap();
                let session = HttpSession { tcp_stream, addr };
                let handler_fn_clone = handler_fn.clone();
                tokio::spawn(async move {
                    handler_fn_clone.handle(session).await;
                });
            }
        });
        Ok(HttpServer {})
    }
}

struct Handler {}

#[async_trait]
impl HttpSessionHandler for Handler {
    async fn handle(&self, session: HttpSession) {
        info!("Got {:?}", session)
    }
}

pub async fn async_main() -> () {
    let _http_server = HttpServerBuilder::new(1690)
        .all_interfaces()
        // TODO(mleonhard) Make this take an async closure once they are stable, https://github.com/rust-lang/rust/issues/62290
        .run(Arc::new(Handler {}))
        .await.unwrap();
    tokio::net::TcpStream::connect("127.0.0.1:1690").await.unwrap();
    tokio::net::TcpStream::connect("::1:1690").await.unwrap();

    // $ DEV_LOG_FORMAT=compact cargo run --bin handler_trait
    // 2020-05-08T02:29:18.461-07:00 INFO Listening for TCP connections on [::]:1690
    // 2020-05-08T02:29:18.461-07:00 INFO Got HttpSession { tcp_stream: TcpStream { addr: V6([::ffff:127.0.0.1]:1690), peer: V6([::ffff:127.0.0.1]:58762), fd: 9 }, addr: V6([::ffff:127.0.0.1]:58762) }
    // 2020-05-08T02:29:18.462-07:00 INFO Got HttpSession { tcp_stream: TcpStream { addr: V6([::1]:1690), peer: V6([::1]:58763), fd: 8 }, addr: V6([::1]:58763) }
}

pub fn main() {
    let _global_logger_guard = logging::configure("info").unwrap();
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async_main());
    // Drops waiting tasks.  Waits for all busy tasks to await and drops them.  Gives up after timeout.
    runtime.shutdown_timeout(std::time::Duration::from_secs(3));
}
