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

pub async fn wait_for_stop_signal() {
    // Handle TERM signal for running in Docker, Kubernetes, supervisord, etc.
    // Also handle INT signal from CTRL-C in dev terminal.
    use tokio::signal::unix::{signal, SignalKind};
    let mut term_signal = signal(SignalKind::terminate())
        .expect("Failed installing TERM signal handler");
    let mut int_signal = signal(SignalKind::interrupt())
        .expect("Failed installing INT signal handler");
    use tokio::stream::StreamExt;
    futures::future::select(term_signal.next(), int_signal.next()).await;
    println!("Server stopping");
}

#[tokio::main]
pub async fn main() -> () {
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(hello)) }
    });
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(wait_for_stop_signal());
    println!("Listening on http://{}", addr);
    let server_handle = tokio::spawn(async move {
        server.await.unwrap();
        println!("Server stopped.");
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
    http_get("http://127.0.0.1:1690").await;
    println!("Sending TERM signal to self");
    nix::sys::signal::kill(nix::unistd::getpid(), nix::sys::signal::SIGTERM).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    http_get("http://127.0.0.1:1690").await;  // Connection refused.
    server_handle.await.unwrap();
    println!("Done.");

    // $ cargo run --bin graceful_shutdown
    // Listening on http://127.0.0.1:1690
    // GET http://127.0.0.1:1690/
    // 200 OK b"Hello World!\n"
    // Sending TERM signal to self
    // Server stopping
    // Server stopped.
    // GET http://127.0.0.1:1690/
    // Error: error trying to connect: tcp connect error: Connection refused (os error 61)
    // Done.

    // $ cargo run --bin graceful_shutdown
    // Listening on http://127.0.0.1:1690
    // ^CServer stopping
    // Server stopped.
    // GET http://127.0.0.1:1690/
    // Error: error trying to connect: tcp connect error: Connection refused (os error 61)
    // Sending TERM signal to self
    // GET http://127.0.0.1:1690/
    // Error: error trying to connect: tcp connect error: Connection refused (os error 61)
    // Done.
}
