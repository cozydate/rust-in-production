use std::println;
use tokio::net::{TcpListener, TcpStream};

pub fn parse_env_var<T>(name: &str, default: T) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    match std::env::var(name) {
        Ok(s) => s
            .parse()
            .expect(&format!("Failed parsing env var {}={:?}", name, s)),
        Err(std::env::VarError::NotUnicode(oss)) => {
            panic!("Failed parsing {}={:?} value as UTF-8", name, oss)
        }
        Err(std::env::VarError::NotPresent) => default,
    }
}

async fn handle_conn(mut tcp_stream: TcpStream) {
    use tokio::io::AsyncWriteExt;
    if let Err(e) = tcp_stream.write_all(b"response").await {
        println!("WARN server write error: {:?}", e);
        return;
    }
}

async fn accept_loop(mut listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((tcp_stream, _addr)) => {
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
    let mut tcp_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    use tokio::io::AsyncReadExt;
    let mut buf = String::new();
    if let Err(e) = tcp_stream.read_to_string(&mut buf).await {
        println!("WARN client read error: {:?}", e);
        return;
    }
    println!("INFO client read {:?}", buf);
}

async fn async_main() -> () {
    let port: u16 = parse_env_var("PORT", 1690); // <---------------------
    let listener = listen_on_all_interfaces(port).await.unwrap();
    println!(
        "INFO server listening on {}",
        listener.local_addr().unwrap()
    );
    tokio::spawn(accept_loop(listener));

    call_server(&format!("127.0.0.1:{}", port)).await;
    call_server(&format!("[::1]:{}", port)).await;
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

// $ cargo run --bin port
// INFO server listening on [::]:1690
// INFO client read "response"
// INFO client read "response"

// $ PORT=1700 cargo run --bin port
// INFO server listening on [::]:1700
// INFO client read "response"
// INFO client read "response"

// $ PORT= cargo run --bin port
// thread 'main' panicked at 'Failed parsing env var PORT="": ParseIntError { kind: Empty }', src/bin/port.rs:12:14
// note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

// $ PORT=abc cargo run --bin port
// thread 'main' panicked at 'Failed parsing env var PORT="abc": ParseIntError { kind: InvalidDigit }', src/bin/port.rs:12:14
// note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
