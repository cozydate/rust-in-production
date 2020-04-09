use std::convert::Infallible;

use hyper::{Body, Client, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn http_get(url: &str) {
    let url: hyper::Uri = url.parse().unwrap();
    println!("GET {}", url);
    let mut response = Client::new().get(url).await.unwrap();
    let _body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
    println!("{} {:?}", response.status(), _body);
}

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!\n")))
}

#[tokio::main]
pub async fn main() -> () {
    let addr = std::net::SocketAddr::from(
        (std::net::Ipv6Addr::UNSPECIFIED /* includes ipv4 */, 1690));
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(hello)) }
    });
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on {}", &addr);
    tokio::spawn(async move {
        server.await.unwrap();
    });

    http_get("http://127.0.0.1:1690").await;
    http_get("http://[::1]:1690").await;

    // $ cargo run --bin ipv4_and_ipv6
    // Listening on [::]:1690
    // GET http://127.0.0.1:1690/
    // 200 OK b"Hello World!\n"
    // GET http://[::1]:1690/
    // 200 OK b"Hello World!\n"
}
