use std::convert::Infallible;

use hyper::{Body, Client, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn http_get(url: &str) {
    let url: hyper::Uri = url.parse().unwrap();
    println!("GET {}", url);
    let mut response = match Client::new().get(url).await {
        Ok(response) => response,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    match hyper::body::to_bytes(response.body_mut()).await {
        Ok(bytes) => println!("{} {:?}", response.status(), bytes),
        Err(e) => println!("Error: {}", e),
    };
}

pub fn parse_env_var<T>(name: &str, default: T) -> T
    where T: std::str::FromStr, <T as std::str::FromStr>::Err: std::fmt::Debug
{
    match std::env::var(name) {
        Ok(s) =>
            s.parse().expect(&format!("Failed parsing env var {}={:?}", name, s)),
        Err(std::env::VarError::NotUnicode(oss)) =>
            panic!("Failed parsing {}={:?} value as UTF-8", name, oss),
        Err(std::env::VarError::NotPresent) =>
            default,
    }
}

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!\n")))
}

#[tokio::main]
pub async fn main() -> () {
    let port: u16 = parse_env_var("PORT", 1690);

    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(hello)) }
    });
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on {}", &addr);
    tokio::spawn(async move {
        server.await.unwrap();
    });

    http_get(&format!("http://127.0.0.1:{}", port)).await;

    // $ cargo run --bin port
    // Listening on 127.0.0.1:1690
    // GET http://127.0.0.1:1690/
    // 200 OK b"Hello World!\n"

    // $ PORT=1700 cargo run --bin port
    // Listening on 127.0.0.1:1700
    // GET http://127.0.0.1:1700/
    // 200 OK b"Hello World!\n"

    // $ PORT= cargo run --bin port
    // thread 'main' panicked at 'Failed parsing env var PORT="": ParseIntError { kind: Empty }', src/bin/port.rs:11:13

    // $ PORT=abc cargo run --bin port
    // thread 'main' panicked at 'Failed parsing env var PORT="abc": ParseIntError { kind: InvalidDigit }', src/bin/port.rs:11:13
}
