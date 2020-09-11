// This program shows how to pass connection handler functions.

use std::future::Future;
use std::pin::Pin;
use std::println;
use tokio::net::{TcpListener, TcpStream};

fn handle_conn(mut tcp_stream: TcpStream) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        println!("INFO handler writing 'response'");
        use tokio::io::AsyncWriteExt;
        if let Err(e) = tcp_stream.write_all(b"response").await {
            println!("WARN handler write error: {:?}", e);
            return;
        }
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
                println!("INFO accept_loop accepted connection {}", addr);
                tokio::spawn(handler(tcp_stream));
            }
            Err(e) => {
                println!("WARN accept_loop error: {:?}", e);
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn call_server(addr: &str) {
    let mut tcp_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    println!(
        "INFO client connected to {}",
        tcp_stream.peer_addr().unwrap()
    );
    use tokio::io::AsyncReadExt;
    let mut buf = String::new();
    if let Err(e) = tcp_stream.read_to_string(&mut buf).await {
        println!("WARN client read error: {:?}", e);
        return;
    }
    println!("INFO client read {:?}", buf);
}

async fn async_main() -> () {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("INFO listening on {}", listener.local_addr().unwrap());
    tokio::spawn(accept_loop(listener, handle_conn));

    call_server("127.0.0.1:1690").await;
}

pub fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async_main());
    runtime.shutdown_background();
}

// $ cargo run --bin handle_conn_fns
// INFO listening on 127.0.0.1:1690
// INFO accept_loop accepted connection 127.0.0.1:56172
// INFO client connected to 127.0.0.1:1690
// INFO handler writing 'response'
// INFO client read "response"
