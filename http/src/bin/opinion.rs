use std::convert::Infallible;

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn http_get(url: &str) {
    let url: hyper::Uri = url.parse().unwrap();
    logging::info!("GET {}", url);
    let mut response = Client::new().get(url).await.unwrap();
    let _body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
    logging::info!("{} {:?}", response.status(), _body);
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

pub fn random_id(len: usize) -> String {
    // Alphabet has 27 characters. Each randomly-selected character adds 4.75 bits of entropy.
    // Selecting 8 with replacement yields a random string with 38 bits of entropy.
    // At one request-per-second, duplicate request ids will occur once every 74 days, on average.
    // https://en.wikipedia.org/wiki/Birthday_problem
    // http://davidjohnstone.net/pages/hash-collision-probability
    use rand::seq::IteratorRandom;
    let mut rng = rand::thread_rng();
    std::iter::repeat(())
        .take(len)
        .map(|_| "123456789CDFGHJKLMNPQRTVWXZ".chars().choose(&mut rng).unwrap())
        .collect()
}

async fn handle_request(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    let logger = slog_scope::logger().new(slog::o!("request_id" => random_id(8)));
    slog_scope::scope(&logger, || {
        logging::info!("Handling request");
        Ok(Response::new(Body::from("Hello World!\n")))
    })
}

#[tokio::main]
pub async fn main() -> () {
    let _guard = logging::configure("info").unwrap();
    let port: u16 = parse_env_var("PORT", 1690);
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(handle_request)) }
    });
    let addr = std::net::SocketAddr::from(
        (std::net::Ipv6Addr::UNSPECIFIED /* includes ipv4 */, port));
    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(wait_for_sigterm());
    logging::info!("Listening on http://{}", addr);
    server.await.unwrap();
    logging::info!("Exiting.");
}
