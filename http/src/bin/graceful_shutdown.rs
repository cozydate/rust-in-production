use std::convert::Infallible;

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

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
    server.await.unwrap();
    println!("Server stopped.");

    // $ cargo run --bin graceful_shutdown
    // Listening on http://127.0.0.1:1690
    // ^CServer stopping
    // Server stopped.

    // $ cargo run --bin graceful_shutdown
    // Listening on http://127.0.0.1:1690
    // Server stopping
    // Server stopped.

    // $ curl http://127.0.0.1:1690/
    // Hello World!
    // $ pkill graceful_shutdown
    // $ curl http://127.0.0.1:1690/
    // curl: (7) Failed to connect to 127.0.0.1 port 1690: Connection refused
}
