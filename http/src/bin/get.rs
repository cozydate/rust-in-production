// This program shows how to handle HTTP 1.1 requests.
use std::println;

use beatrice_http::{Http11Method, Http11Request, Http11ResponseWriter, Http11Status, read_http11_request};
use beatrice_http::buffer::Buffer;

async fn handle_get<'a, T>(_req: &Http11Request, resp: &mut Http11ResponseWriter<'a, T>)
                           -> std::io::Result<()>
    where T: tokio::io::AsyncWrite + std::marker::Unpin
{
    resp.send_text(Http11Status::Ok200, "body1").await
}

async fn handle_connection<'a>(tcp_stream: &'a mut tokio::net::TcpStream) -> std::io::Result<()> {
    let (mut tcp_reader, mut tcp_writer) = tcp_stream.split();
    let mut mem: [u8; 4 * 1024] = [0; 4 * 1024];
    let mut buffer = Buffer::new(&mut mem[..]);
    let req = read_http11_request(&mut tcp_reader, &mut buffer).await?;
    let mut resp = Http11ResponseWriter::new(&mut tcp_writer);
    let result = match req.method {
        Http11Method::GET => {
            handle_get(&req, &mut resp).await
        }
        _ => {
            resp.send_without_body(Http11Status::MethodNotAllowed405).await
        }
    };
    match &result {
        Ok(_) => {
            println!("INFO server {:?} {:?}", req, resp);
        }
        Err(e) => {
            println!("INFO server {:?} err={}", req, e);
            let _ = resp.send_without_body(Http11Status::InternalServerError500).await;
        }
    };
    result
}

async fn async_main() -> () {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let mut listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!(
        "INFO server listening on {}",
        listener.local_addr().unwrap()
    );
    tokio::spawn(async move {
        loop {
            let (mut tcp_stream, _addr) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(&mut tcp_stream).await {
                    println!("ERROR server error: {:?}", e);
                }
            });
        }
    });

    let response = reqwest::Client::new()
        .get("http://127.0.0.1:1690/path1")
        .send()
        .await
        .unwrap();
    println!("INFO client response {:?}", response);
    assert_eq!(200, response.status().as_u16());
    let body = response.bytes().await.unwrap();
    println!("INFO client response body {:?}", body);
    assert_eq!(bytes::Bytes::from_static(b"body1"), body);
}

pub fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async_main());
    runtime.shutdown_background();
}

// $ cargo run --bin get
// INFO server listening on 127.0.0.1:1690
// INFO server Http11Request { method: GET, path: "/path1", expect_100_continue: false, content_length: 0, chunked: false } Http11ResponseWriter{Some(Ok200), bytes_written=84}
// INFO client response Response { url: "http://127.0.0.1:1690/path1", status: 200, headers: {"content-type": "text/plain; charset=UTF-8", "content-length": "5"} }
// INFO client response body b"body1"
