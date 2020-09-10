use std::println;
use tokio::net::{TcpListener, TcpStream};

async fn handle_conn(mut tcp_stream: TcpStream) {
    use tokio::io::AsyncReadExt;
    let mut buf = String::new();
    while let Err(e) = tcp_stream.read_to_string(&mut buf).await {
        println!("WARN server read error: {:?}", e);
        return;
    }
    println!("INFO server read {:?}", buf);
    println!("INFO server writing 'response'");
    use tokio::io::AsyncWriteExt;
    while let Err(e) = tcp_stream.write_all(b"response").await {
        println!("WARN server write error: {:?}", e);
    }
}

async fn accept_loop(mut listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((tcp_stream, addr)) => {
                println!("INFO accepted {}", addr);
                tokio::spawn(handle_conn(tcp_stream));
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
        "INFO client connected to {}",
        tcp_stream.peer_addr().unwrap()
    );
    println!("INFO client writing 'greeting'");
    {
        use tokio::io::AsyncWriteExt;
        while let Err(e) = tcp_stream.write_all(b"greeting").await {
            println!("WARN client write error: {:?}", e);
        }
    }
    while let Err(e) = tcp_stream.shutdown(std::net::Shutdown::Write) {
        println!("WARN client stream shutdown-write error: {:?}", e);
    }
    use tokio::io::AsyncReadExt;
    let mut buf = String::new();
    while let Err(e) = tcp_stream.read_to_string(&mut buf).await {
        println!("WARN client read error: {:?}", e);
        return;
    }
    println!("INFO client read {:?}", buf);
}

async fn async_main() -> () {
    let listener = listen_on_all_interfaces(1690).await.unwrap();
    println!(
        "INFO server listening on {}",
        listener.local_addr().unwrap()
    );
    tokio::spawn(accept_loop(listener));

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

// $ cargo run --bin tcp
// INFO server listening on [::]:1690
// INFO client connecting to 127.0.0.1:1690
// INFO accepted [::ffff:127.0.0.1]:51537
// INFO client connected to 127.0.0.1:1690s
// INFO client writing 'greeting'
// INFO server read "greeting"
// INFO server writing 'response'
// INFO client read "response"
// INFO client connecting to [::1]:1690
// INFO client connected to [::1]:1690s
// INFO client writing 'greeting'
// INFO accepted [::1]:51538
// INFO server read "greeting"
// INFO server writing 'response'
// INFO client read "response"
