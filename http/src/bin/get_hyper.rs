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

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!\n")))
}

#[tokio::main]
pub async fn main() -> () {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(hello)) }
    });
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on {}", &addr);
    tokio::spawn(async move {
        server.await.unwrap();
    });

    http_get("http://127.0.0.1:1690").await;

    // $ cargo run --bin get
    // Listening on 127.0.0.1:1690
    // GET http://127.0.0.1:1690/
    // 200 OK b"Hello World!\n"
}
