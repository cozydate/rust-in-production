use std::future::Future;
use std::pin::Pin;
use std::println;
use tokio::net::{TcpListener, TcpStream};

fn handle_conn(mut tcp_stream: TcpStream) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        use tokio::io::AsyncReadExt;
        let mut buf = String::new();
        match tcp_stream.read_to_string(&mut buf).await {
            Ok(_) => {
                println!("INFO server read {:?}", buf);
            }
            Err(e) => {
                println!("WARN server read error: {:?}", e);
                return;
            }
        };
        println!("INFO server writing 'response'");
        use tokio::io::AsyncWriteExt;
        match tcp_stream.write_all(b"response").await {
            Ok(_) => {}
            Err(e) => {
                println!("WARN server write error: {:?}", e);
            }
        };
        match tcp_stream.shutdown(std::net::Shutdown::Write) {
            Ok(_) => {}
            Err(e) => {
                println!("WARN server stream shutdown error: {:?}", e);
            }
        };
    })
}

// Note: We pass a fn that returns a Future which performs the actual handling.
//       We can simplify this once Rust supports async closures:
//       "Tracking issue for `#!feature(async_closure)]` (RFC 2394)"
//       https://github.com/rust-lang/rust/issues/62290
type HandlerFn = fn(tokio::net::TcpStream) -> Pin<Box<dyn Future<Output = ()> + Send>>;
async fn accept_loop(mut listener: TcpListener, handler: HandlerFn) {
    loop {
        match listener.accept().await {
            Ok((tcp_stream, addr)) => {
                println!("INFO accepted {}", addr);
                tokio::spawn(handler(tcp_stream));
            }
            Err(e) => {
                println!("WARN Failed accepting connection from socket: {:?}", e);
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn listen_on_all_interfaces(port: u16) -> tokio::io::Result<TcpListener> {
    let interface =
        std::net::IpAddr::from(std::net::Ipv6Addr::UNSPECIFIED /* includes ipv4 */);
    let addr = std::net::SocketAddr::from((interface, port));
    tokio::net::TcpListener::bind(&addr).await
}

async fn call_server(addr: &str) {
    println!("INFO client connecting to {}", addr);
    let mut tcp_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    println!(
        "INFO client connected to {}s",
        tcp_stream.peer_addr().unwrap()
    );
    println!("INFO client writing 'greeting'");
    {
        use tokio::io::AsyncWriteExt;
        match tcp_stream.write_all(b"greeting").await {
            Ok(_) => {}
            Err(e) => {
                println!("WARN client write error: {:?}", e);
            }
        };
    }
    match tcp_stream.shutdown(std::net::Shutdown::Write) {
        Ok(_) => {}
        Err(e) => {
            println!("WARN client stream shutdown-write error: {:?}", e);
        }
    };
    use tokio::io::AsyncReadExt;
    let mut buf = String::new();
    match tcp_stream.read_to_string(&mut buf).await {
        Ok(_) => {
            println!("INFO client read {:?}", buf);
        }
        Err(e) => {
            println!("WARN client read error: {:?}", e);
            return;
        }
    };
}

async fn async_main() -> () {
    let listener = listen_on_all_interfaces(1690).await.unwrap();
    println!(
        "INFO server listening on {}",
        listener.local_addr().unwrap()
    );
    tokio::spawn(accept_loop(listener, handle_conn));

    call_server("127.0.0.1:1690").await;
    call_server("[::1]:1690").await;
}

pub fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async_main());
    // Drops waiting tasks.  Waits for all busy tasks to await and drops them.  Gives up after timeout.
    runtime.shutdown_timeout(std::time::Duration::from_secs(3));
}

// $ cargo run --bin tcp_stream_handler
// INFO server listening on [::]:1690
// INFO client connecting to 127.0.0.1:1690
// INFO client connected to 127.0.0.1:1690s
// INFO client writing 'greeting'
// INFO accepted [::ffff:127.0.0.1]:51436
// INFO server read "greeting"
// INFO server writing 'response'
// INFO client read "response"
// INFO client connecting to [::1]:1690
// INFO client connected to [::1]:1690s
// INFO client writing 'greeting'
// INFO accepted [::1]:51437
// INFO server read "greeting"
// INFO server writing 'response'
// INFO client read "response"
