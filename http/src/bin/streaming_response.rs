use std::convert::Infallible;

use http_body::Body as http_body_Body;
use hyper::{Body, Client, Request, Response, Server};
use hyper::body::Bytes;
use hyper::service::{make_service_fn, service_fn};
use tokio::stream::StreamExt as tokio_stream_StreamExt;

async fn http_get(url: &str) {
    let url: hyper::Uri = url.parse().unwrap();
    println!("Get {}", url);
    let mut response = match Client::new().get(url).await {
        Ok(response) => response,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    println!("Result {}", response.status());
    while let Some(chunk) = response.body_mut().data().await {
        match chunk {
            Ok(bytes) => println!("chunk {:?}", bytes),
            Err(e) => {
                println!("Error reading chunk: {}", e);
                return;
            }
        }
    }
    println!("Read all chunks");
}

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    let top = futures::stream::once(async { String::from("Start:\n") });
    let middle = {
        let interval = tokio::time::interval(std::time::Duration::from_secs(1)).take(5);
        let numbers = {
            use futures::StreamExt;
            interval.enumerate()
        };
        numbers.map(|(n, _val)| format!("{}\n", n))
    };
    let bottom = futures::stream::once(async { String::from("Done.\n") });
    let combined = top.chain(middle.chain(bottom));
    let as_results = combined.map(|s| Result::Ok::<Bytes, String>(Bytes::from(s)))
        //.chain(futures::stream::once(async { Result::Err(String::from("error1")) }))
        ;
    Ok(Response::new(Body::wrap_stream(as_results)))
}

pub async fn wait_for_stop_signal() {
    // Handle TERM signal for running in Docker, Kubernetes, supervisord, etc.
    // Also handle INT signal from CTRL-C in dev terminal.
    use tokio::signal::unix::{signal, SignalKind};
    let mut term_signal = signal(SignalKind::terminate())
        .expect("Failed installing TERM signal handler");
    let mut int_signal = signal(SignalKind::interrupt())
        .expect("Failed installing INT signal handler");
    futures::future::select(term_signal.next(), int_signal.next()).await;
    println!("Server stopping");
}

#[tokio::main]  // Sets up executor.  Waits for all tasks to finish before exiting the process.
pub async fn main() -> () {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(hello)) }
    });
    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(wait_for_stop_signal());
    println!("Listening on {}", &addr);
    let server_handle = tokio::spawn(async move {
        server.await.unwrap();
        println!("Server stopped.");
    });

    // Client is streaming request when server starts shutdown.  Should finish streaming.
    tokio::spawn(http_get("http://127.0.0.1:1690"));
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("Sending TERM signal");
    nix::sys::signal::kill(nix::unistd::getpid(), nix::sys::signal::SIGTERM).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    http_get("http://127.0.0.1:1690").await;  // Connection refused.
    server_handle.await.unwrap();
    println!("Done.");

    // $ cargo run --bin streaming_response
    // Listening on 127.0.0.1:1690
    // Get http://127.0.0.1:1690/
    // Result 200 OK
    // chunk b"Start:\n"
    // chunk b"0\n"
    // Sending TERM signal
    // Server stopping
    // chunk b"1\n"
    // Get http://127.0.0.1:1690/
    // chunk b"2\n"
    // Error: error trying to connect: tcp connect error: Connection refused (os error 61)
    // chunk b"3\n"
    // Server stopped.
    // chunk b"4\n"
    // Done.
    // chunk b"Done.\n"
    // Read all chunks
}
