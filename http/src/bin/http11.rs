// This program shows how to handle HTTP 1.1 requests.
use std::println;

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
            let (mut tcp_reader, mut tcp_writer) = tcp_stream.split();
            let mut buffer = beatrice_http::buffer::Buffer::new();
            match beatrice_http::read_http11_request(&mut tcp_reader, &mut buffer).await {
                Err(e) => {
                    println!("WARN server read error: {:?}", e);
                    return;
                }
                Ok(req) => {
                    println!("INFO server got req {:?}", req);
                }
            }
            use tokio::io::AsyncWriteExt;
            if let Err(e) = tcp_writer.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 5\r\n\r\nbody1").await {
                println!("WARN server write error: {:?}", e);
                return;
            }
        }
    });

    let response = reqwest::Client::new()
        .put("http://127.0.0.1:1690/path1")
        .body("req1")
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

// $ cargo run --bin http11
// INFO server listening on 127.0.0.1:1690
// INFO server got req Http11Request { method: PUT, path: "/path1", expecting_100_continue: false, content_length: 4, chunked: false }
// INFO client response Response { url: "http://127.0.0.1:1690/path1", status: 200, headers: {"content-length": "5"} }
// INFO client response body b"body1"
