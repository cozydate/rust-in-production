// This program shows how to handle multiple connections at the same time.

use std::println;
use tokio::net::{TcpListener, TcpStream};

async fn handle_conn(mut tcp_stream: TcpStream) {
    let addr = tcp_stream.peer_addr().unwrap();
    use tokio::io::AsyncReadExt;
    let mut buf = String::new();
    if let Err(e) = tcp_stream.read_to_string(&mut buf).await {
        println!("WARN handler {} read error: {:?}", addr, e);
        return;
    }
    println!("INFO handler {} read {:?}", addr, buf);
    println!("INFO handler {} writing 'response'", addr,);
    use tokio::io::AsyncWriteExt;
    if let Err(e) = tcp_stream.write_all(b"response").await {
        println!("WARN handler {} write error: {:?}", addr, e);
        return;
    }
}

async fn accept_loop(mut listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((tcp_stream, addr)) => {
                println!("INFO server accepted connection {}", addr);
                tokio::spawn(handle_conn(tcp_stream));
            }
            Err(e) => {
                println!("WARN server error accepting connection: {:?}", e);
                tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn call_server(addr: &str) {
    println!("INFO client {} connecting", addr);
    let mut tcp_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    println!(
        "INFO client {} connected to {}",
        addr,
        tcp_stream.peer_addr().unwrap()
    );
    println!("INFO client {} writing 'greeting'", addr);
    {
        use tokio::io::AsyncWriteExt;
        if let Err(e) = tcp_stream.write_all(b"greeting").await {
            println!("WARN client {} write error: {:?}", addr, e);
            return;
        }
    }
    if let Err(e) = tcp_stream.shutdown(std::net::Shutdown::Write) {
        println!("WARN client {} stream shutdown-write error: {:?}", addr, e);
        return;
    }
    use tokio::io::AsyncReadExt;
    let mut buf = String::new();
    if let Err(e) = tcp_stream.read_to_string(&mut buf).await {
        println!("WARN client {} read error: {:?}", addr, e);
        return;
    }
    println!("INFO client {} read {:?}", addr, buf);
}

async fn async_main() -> () {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!(
        "INFO server listening on {}",
        listener.local_addr().unwrap()
    );
    tokio::spawn(accept_loop(listener));
    tokio::spawn(call_server("127.0.0.1:1690"));
    tokio::spawn(call_server("127.0.0.1:1690"));
    tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
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

// $ cargo run --bin concurrent_connections
// INFO server listening on 127.0.0.1:1690
// INFO client 127.0.0.1:1690 connecting
// INFO client 127.0.0.1:1690 connecting
// INFO client 127.0.0.1:1690 connected to 127.0.0.1:1690
// INFO client 127.0.0.1:1690 writing 'greeting'
// INFO client 127.0.0.1:1690 connected to 127.0.0.1:1690
// INFO client 127.0.0.1:1690 writing 'greeting'
// INFO server accepted connection 127.0.0.1:56135
// INFO server accepted connection 127.0.0.1:56136
// INFO handler 127.0.0.1:56136 read "greeting"
// INFO handler 127.0.0.1:56136 writing 'response'
// INFO handler 127.0.0.1:56135 read "greeting"
// INFO handler 127.0.0.1:56135 writing 'response'
// INFO client 127.0.0.1:1690 read "response"
// INFO client 127.0.0.1:1690 read "response"
