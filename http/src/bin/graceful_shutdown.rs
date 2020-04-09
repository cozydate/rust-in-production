use std::convert::Infallible;

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!")))
}

pub async fn wait_for_sigterm() {
    // Handle TERM signal for running in Docker, Kubernetes, supervisord, etc.
    // Also handle INT signal from CTRL-C in dev terminal.
    use tokio::signal::unix::{signal, SignalKind};
    let term_signal = signal(SignalKind::terminate())
        .expect("Failed installing TERM signal handler");
    let int_signal = signal(SignalKind::interrupt())
        .expect("Failed installing INT signal handler");
    // https://docs.rs/tokio/0.2.16/tokio/stream/trait.StreamExt.html#method.merge
    use tokio::stream::StreamExt;
    term_signal.merge(int_signal).next().await;
}

#[tokio::main]
pub async fn main() -> () {
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(hello)) }
    });
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(wait_for_sigterm());
    println!("Listening on http://{}", addr);
    server.await.unwrap();
    println!("Exiting.");

    // $ cargo run --bin graceful_shutdown
    // Listening on http://127.0.0.1:1690
    // ^CExiting.


    // $ cargo run --bin graceful_shutdown
    // Listening on http://127.0.0.1:1690
    // Exiting.

    // $ pkill graceful_shutdown
}
