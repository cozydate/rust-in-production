// This program shows how to receive TCP connections.

use std::println;

async fn async_main() -> () {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let mut listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!(
        "INFO server listening on {}",
        listener.local_addr().unwrap()
    );
    tokio::spawn(async move {
        loop {
            let (mut tcp_stream, _addr) = listener.accept().await.unwrap();
            use tokio::io::AsyncWriteExt;
            if let Err(e) = tcp_stream.write_all(b"response").await {
                println!("WARN server write error: {:?}", e);
                return;
            }
        }
    });

    let mut tcp_stream = tokio::net::TcpStream::connect("127.0.0.1:1690")
        .await
        .unwrap();
    use tokio::io::AsyncReadExt;
    let mut buf = String::new();
    if let Err(e) = tcp_stream.read_to_string(&mut buf).await {
        println!("WARN client read error: {:?}", e);
        return;
    }
    println!("INFO client read {:?}", buf);
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

// $ cargo run --bin tcp
// INFO server listening on 127.0.0.1:1690
// INFO client read "response"
