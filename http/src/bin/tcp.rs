use std::sync::Arc;

use logging::info;

#[derive(Debug)]
pub struct HttpSession {
    tcp_stream: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
}

pub struct HttpServer {
    listener: tokio::net::TcpListener,
}

impl HttpServer {
    pub async fn next(&mut self) -> Option<HttpSession> {
        let (tcp_stream, addr) = self.listener.accept().await.unwrap();
        Some(HttpSession { tcp_stream, addr })
    }
}

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

    pub async fn run(self) -> std::io::Result<HttpServer> {
        let interface =
            if self.all_interfaces {
                std::net::IpAddr::from(std::net::Ipv6Addr::UNSPECIFIED /* includes ipv4 */)
            } else {
                std::net::IpAddr::from(std::net::Ipv4Addr::LOCALHOST)
            };
        let addr = std::net::SocketAddr::from((interface, self.port));
        info!("Listening for TCP connections on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        Ok(HttpServer { listener })
    }
}

struct Handler {}

impl Handler {
    async fn handle(&self, session: HttpSession) {
        info!("Got {:?}", session)
    }
}

pub async fn async_main() -> () {
    let mut http_server = HttpServerBuilder::new(1690)
        .all_interfaces()
        .run()
        .await.unwrap();
    let handler = Arc::new(Handler {});
    tokio::spawn(async move {
        loop {
            let session = http_server.next().await.unwrap();
            let handler_clone = handler.clone();
            tokio::spawn(async move {
                handler_clone.handle(session).await;
            });
        }
    });

    tokio::net::TcpStream::connect("127.0.0.1:1690").await.unwrap();
    tokio::net::TcpStream::connect("::1:1690").await.unwrap();
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
