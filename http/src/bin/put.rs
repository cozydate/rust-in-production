// This program shows how to handle HTTP 1.1 requests.
use std::pin::Pin;
use std::println;

use beatrice_http::{
    escape_ascii,
    HttpError,
    HttpMethod,
    HttpReaderWriter,
    HttpStatus,
};

async fn handle_get(http_reader_writer: &mut HttpReaderWriter<'_>) -> Result<(), HttpError> {
    http_reader_writer.send_text(HttpStatus::Ok200, "body1").await
}

async fn handle_put(http_reader_writer: &mut HttpReaderWriter<'_>) -> Result<(), HttpError>
{
    let body_len = http_reader_writer.content_length_usize()?;
    if http_reader_writer.content_length < 1 {
        return http_reader_writer.send_without_body(HttpStatus::LengthRequired411).await;
    }
    if http_reader_writer.content_length > 4 * 1024 {
        return http_reader_writer.send_without_body(HttpStatus::PayloadTooLarge413).await;
    }
    let mut body_mem: [u8; 4 * 1024] = [0; 4 * 1024];
    let mut body_bytes = &mut body_mem[..body_len];
    tokio::io::AsyncReadExt::read_exact(http_reader_writer, &mut body_bytes)
        .await
        .map_err(HttpError::from_io_err)?;
    println!("INFO handle_put body {:?}", escape_ascii(body_bytes));
    http_reader_writer.send_without_body(HttpStatus::Created201).await
}

async fn handle_request(http_reader_writer: &mut HttpReaderWriter<'_>) -> Result<(), HttpError> {
    http_reader_writer.read_request(&mut []).await?;
    match http_reader_writer.method {
        Some(HttpMethod::GET) => {
            handle_get(http_reader_writer).await
        }
        Some(HttpMethod::PUT) => {
            handle_put(http_reader_writer).await
        }
        _ => {
            Err(HttpError::ProcessingError(HttpStatus::MethodNotAllowed405))
        }
    }
}

async fn handle_connection(tcp_stream: &mut tokio::net::TcpStream) {
    let (mut tcp_reader, mut tcp_writer) = tcp_stream.split();
    let mut http_reader_writer =
        HttpReaderWriter::new(Pin::new(&mut tcp_reader), Pin::new(&mut tcp_writer));
    loop {
        http_reader_writer.reset();
        match handle_request(&mut http_reader_writer).await {
            Err(HttpError::IoError(e)) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    println!("INFO server client disconnected");
                } else {
                    println!("INFO server io_error={:?}", e);
                }
                let _ = tcp_stream.shutdown(std::net::Shutdown::Both);
                break;
            }
            Err(HttpError::ParseError(e)) => {
                println!("INFO server parse_error={:?}", e);
                let _ = http_reader_writer.send_without_body(e.status()).await;
            }
            Err(HttpError::ProcessingError(status)) => {
                println!("INFO server {:?} processing_error={:?}",
                         http_reader_writer.method, status);
                let _ = http_reader_writer.send_without_body(status).await;
            }
            Ok(req) => {
                println!("INFO server {:?} {:?}", req, http_reader_writer);
            }
        };
    }
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
            tokio::spawn(async move { handle_connection(&mut tcp_stream).await });
        }
    });

    let client = reqwest::Client::new();
    println!("INFO client doing GET");
    let response = client.get("http://127.0.0.1:1690/path1")
        .send()
        .await
        .unwrap();
    println!("INFO client response {:?}", response);
    assert_eq!(200, response.status().as_u16());
    let body = response.bytes().await.unwrap();
    println!("INFO client response body {:?}", body);
    assert_eq!(bytes::Bytes::from_static(b"body1"), body);

    println!("INFO client doing PUT");
    let response = client.put("http://127.0.0.1:1690/path2")
        .body("request-body1")
        .send()
        .await
        .unwrap();
    println!("INFO client response {:?}", response);
    assert_eq!(201, response.status().as_u16());
    let body = response.bytes().await.unwrap();
    println!("INFO client response body {:?}", body);
    assert_eq!(bytes::Bytes::from_static(b""), body);
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
