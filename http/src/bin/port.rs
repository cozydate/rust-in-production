// This program shows how to read the PORT environment variable.

use std::println;

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

async fn async_main() -> () {
    let port: u16 = parse_env_var("PORT", 1690); // <--------------
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
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

    let mut tcp_stream = tokio::net::TcpStream::connect(&format!("127.0.0.1:{}", port))
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

// $ cargo run --bin port
// INFO server listening on 127.0.0.1:1690
// INFO client read "response"

// $ PORT=1700 cargo run --bin port
// INFO server listening on 127.0.0.1:1700
// INFO client read "response"

// $ PORT= cargo run --bin port
// thread 'main' panicked at 'Failed parsing env var PORT="": ParseIntError { kind: Empty }', src/bin/port.rs:11:14

// $ PORT=abc cargo run --bin port
// thread 'main' panicked at 'Failed parsing env var PORT="abc": ParseIntError { kind: InvalidDigit }', src/bin/port.rs:11:14
