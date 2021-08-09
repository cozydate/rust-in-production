// use std::convert::Infallible;

// use hyper::{Body, Client, Request, Response, Server};
// use hyper::service::{make_service_fn, service_fn};

// async fn http_get(url: &str) {
//     let url: hyper::Uri = url.parse().unwrap();
//     logging::info!("GET {}", url);
//     let mut response = match Client::new().get(url).await {
//         Ok(response) => response,
//         Err(e) => {
//             logging::error!("{}", e);
//             return;
//         }
//     };
//     match hyper::body::to_bytes(response.body_mut()).await {
//         Ok(bytes) => logging::info!("{} {:?}", response.status(), bytes),
//         Err(e) => logging::error!("{}", e),
//     };
// }

// pub fn random_id(len: usize) -> String {
//     // Alphabet has 27 characters. Each randomly-selected character adds 4.75 bits of entropy.
//     // Selecting 8 with replacement yields a random string with 38 bits of entropy.
//     // At one request-per-second, duplicate request ids will occur once every 74 days, on average.
//     // https://en.wikipedia.org/wiki/Birthday_problem
//     // http://davidjohnstone.net/pages/hash-collision-probability
//     use rand::seq::IteratorRandom;
//     let mut rng = rand::thread_rng();
//     std::iter::repeat(())
//         .take(len)
//         .map(|_| "123456789CDFGHJKLMNPQRTVWXZ".chars().choose(&mut rng).unwrap())
//         .collect()
// }

// async fn handle_request(_: Request<Body>) -> Result<Response<Body>, Infallible> {
//     let logger = slog_scope::logger().new(slog::o!("request_id" => random_id(8)));
//     slog_scope::scope(&logger, || {
//         logging::info!("Handling request");
//         Ok(Response::new(Body::from("Hello World!\n")))
//     })
// }

// #[tokio::main]
// pub async fn main() -> () {
//     // Leak the global logger guard so it will never get dropped.
//     //
//     // Without this, main() drops the guard when it returns, removing the global logger.  After
//     // that, any task or thread that tries to log anything will panic.  The panic logger will try
//     // to log the panic and will itself panic.  This causes a strange error on shutdown:
//     //    thread panicked while processing panic. aborting.
//     //    Illegal instruction: 4
//     //
//     // A cleaner workaround is to ensure that all tasks stop before returning from main().  That is
//     // often not achievable with timely shutdown.
//     Box::leak(Box::new(logging::configure("info").unwrap()));
//     //let _global_logger_guard = logging::configure("info").unwrap();

//     let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
//     let make_svc = make_service_fn(|_conn| {
//         async { Ok::<_, Infallible>(service_fn(handle_request)) }
//     });
//     let server = Server::bind(&addr).serve(make_svc);
//     logging::info!("Listening on {}", addr);
//     tokio::spawn(server);

//     logging::task_scope("client-1", async {
//         http_get("http://127.0.0.1:1690").await;
//     }).await;

//     tokio::task::spawn_blocking(|| {
//         logging::thread_scope("background", || {
//             std::thread::sleep(std::time::Duration::from_secs(1));
//             logging::info!("background work");
//             std::thread::sleep(std::time::Duration::from_secs(1));
//         });
//     });

//     logging::info!("Exiting");

//     // $ DEV_LOG_FORMAT=plain cargo run --bin request_id
//     // 2020-04-13T17:25:22.207-07:00 INFO Listening on 127.0.0.1:1690
//     // 2020-04-13T17:25:22.208-07:00 INFO GET http://127.0.0.1:1690/, task: client-1
//     // 2020-04-13T17:25:22.210-07:00 INFO Handling request, request_id: JCJGR2F7
//     // 2020-04-13T17:25:22.210-07:00 INFO 200 OK b"Hello World!\n", task: client-1
//     // 2020-04-13T17:25:22.211-07:00 INFO Exiting
//     // 2020-04-13T17:25:23.212-07:00 INFO background work, thread: background
// }
