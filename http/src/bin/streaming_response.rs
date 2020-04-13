use std::convert::Infallible;

use http_body::Body as http_body_Body;
use hyper::{Body, Client, Request, Response, Server};
use hyper::body::Bytes;
use hyper::service::{make_service_fn, service_fn};
use tokio::stream::StreamExt as tokio_stream_StreamExt;

async fn http_get(url: &str) {
    let url: hyper::Uri = url.parse().unwrap();
    println!("get {}", url);
    let mut response = Client::new().get(url).await.unwrap();
    println!("result {}", response.status());
    while let Some(chunk) = response.body_mut().data().await {
        match chunk {
            Ok(bytes) => println!("chunk {:?}", bytes),
            Err(e) => {
                println!("Error reading chunk: {}", e);
                return;
            }
        }
    }
    println!("read all chunks");
}

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    let top = futures::stream::once(async { String::from("Start:\n") });
    let interval_numbers = futures::stream::StreamExt::enumerate(
        tokio::time::interval(std::time::Duration::from_secs(1)).take(5))
        .map(|(n, _val)| format!("{}\n", n));
    let bottom = futures::stream::once(async { String::from("Done.\n") });
    let combined = top.chain(interval_numbers.chain(bottom));
    let as_results = combined.map(|s| Result::Ok::<Bytes, String>(Bytes::from(s)))
        //.chain(futures::stream::once(async { Result::Err(String::from("error1")) }))
        ;
    Ok(Response::new(Body::wrap_stream(as_results)))
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

    // $ cargo run --bin streaming_response
    // Listening on 127.0.0.1:1690
    // get http://127.0.0.1:1690/
    // result 200 OK
    // chunk b"Start:\n"
    // chunk b"0\n"
    // chunk b"1\n"
    // chunk b"2\n"
    // chunk b"3\n"
    // chunk b"4\n"
    // chunk b"Done.\n"
    // read all chunks
}
