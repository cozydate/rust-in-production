use std::convert::Infallible;

use hyper::{Body, Client, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn http_get(url: &str) {
    let url: hyper::Uri = url.parse().unwrap();
    logging::info!("GET {}", url);
    let mut response = Client::new().get(url).await.unwrap();
    let _body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
    logging::info!("{} {:?}", response.status(), _body);
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
    let _global_logger_guard = logging::configure("info").unwrap();
    let (up_tx, up_rx) = tokio::sync::oneshot::channel::<()>();
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();

    let server_handle = tokio::spawn(
        logging::task_scope("server", async move {
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
            let make_svc = make_service_fn(|_conn| {
                async { Ok::<_, Infallible>(service_fn(handle_request)) }
            });
            let server = Server::bind(&addr)
                .serve(make_svc)
                .with_graceful_shutdown(async { stop_rx.await.unwrap(); });
            logging::info!("Listening on {}", addr);
            up_tx.send(()).unwrap();
            server.await.unwrap();
            logging::info!("Stopped");
        }));

    logging::task_scope("client", async {
        up_rx.await.unwrap();
        http_get("http://127.0.0.1:1690").await;
        http_get("http://127.0.0.1:1690").await;
        logging::info!("Done");
    }).await;

    // We must wait for all tasks to stop before returning.  If we return too soon,
    // `_global_logger_guard` gets dropped which removes the global logger.  After that, any running
    // thread that tries to log anything will panic.  Then the panic logger will try to log the
    // panic, and will itself panic.  This causes a strange "panic inside panic" error on shutdown.
    // The solution is to that ensure all tasks are stop before returning from main().
    stop_tx.send(()).unwrap();
    server_handle.await.unwrap();
    logging::info!("Exiting");

    // $ DEV_LOG_FORMAT=plain cargo run --bin request_id
    // 2020-04-09T13:49:16.230-07:00 INFO Listening on 127.0.0.1:1690, task: server
    // 2020-04-09T13:49:16.230-07:00 INFO GET http://127.0.0.1:1690/, task: client
    // 2020-04-09T13:49:16.231-07:00 INFO Handling request, request_id: 6CZ3NM6M
    // 2020-04-09T13:49:16.232-07:00 INFO 200 OK b"Hello World!\n", task: client
    // 2020-04-09T13:49:16.232-07:00 INFO GET http://127.0.0.1:1690/, task: client
    // 2020-04-09T13:49:16.233-07:00 INFO Handling request, request_id: NHH7X1KF
    // 2020-04-09T13:49:16.233-07:00 INFO 200 OK b"Hello World!\n", task: client
    // 2020-04-09T13:49:16.233-07:00 INFO Done, task: client
    // 2020-04-09T13:49:16.233-07:00 INFO Stopped, task: server
    // 2020-04-09T13:49:16.233-07:00 INFO Exiting
}
