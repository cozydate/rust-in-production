// This program shows how to handle HTTP 1.1 requests.
use std::println;

use beatrice_http::{
    HttpError,
    HttpMethod,
    HttpRequest,
    HttpResponseWriter,
    HttpStatus,
    read_http_request,
};
use beatrice_http::buffer::Buffer;

async fn handle_get<'a, T>(_req: &HttpRequest, resp: &mut HttpResponseWriter<'a, T>)
                           -> Result<(), HttpError>
    where T: tokio::io::AsyncWrite + std::marker::Unpin
{
    resp.send_text(HttpStatus::Ok200, "body1").await
}

// async fn handle_put<'a, T1, T2>(
//     req: &HttpRequest, body: &mut T1, resp: &mut HttpResponseWriter<'a, T2>)
//     -> Option<HttpResult>
//     where T1: tokio::io::AsyncRead + std::marker::Unpin,
//           T2: tokio::io::AsyncWrite + std::marker::Unpin
// {
//     if req.content_length < 1 {
//         resp.send_without_body(HttpStatus::LengthRequired411).await;
//         return None;
//     }
//     if req.expect_100_continue {
//         resp.send_without_body(HttpStatus::Continue100).await?;
//     }
//     let mut body_bytes: [u8; 4 * 1024] = [0; 4 * 1024];
//     tokio::io::AsyncReadExt::read_exact(body, &mut body_bytes).await?;
//
//     resp.send_without_body(HttpStatus::Created201).await
// }

async fn handle_request<'a, T1, T2>(
    _input: &mut T1, req: HttpRequest, resp: &mut HttpResponseWriter<'a, T2>)
    -> Result<HttpRequest, HttpError>
    where T1: tokio::io::AsyncRead + std::marker::Unpin,
          T2: tokio::io::AsyncWrite + std::marker::Unpin {
    if req.method != HttpMethod::GET {
        return Err(HttpError::ProcessingError(req, HttpStatus::MethodNotAllowed405));
    }
    handle_get(&req, resp)
        .await
        .map(|_| req)
    //let mut body = tokio::io::AsyncReadExt::chain(buffer, tcp_reader);
    // handle_put(&req, &mut input, &mut resp).await
}

async fn parse_and_handle_request<'a, T1, T2>(mut buffer: &mut Buffer<'a>,
                                              mut input: &mut T1,
                                              resp: &mut HttpResponseWriter<'a, T2>)
                                              -> Result<HttpRequest, HttpError>
    where T1: tokio::io::AsyncRead + std::marker::Unpin,
          T2: tokio::io::AsyncWrite + std::marker::Unpin {
    let req = read_http_request(&mut input, &mut buffer).await?;
    handle_request(&mut input, req, resp).await
}

async fn handle_connection(tcp_stream: &mut tokio::net::TcpStream) {
    let (mut tcp_reader, mut tcp_writer) = tcp_stream.split();
    let mut mem: [u8; 4 * 1024] = [0; 4 * 1024];
    let mut buffer = Buffer::new(&mut mem[..]);
    let mut resp = HttpResponseWriter::new(&mut tcp_writer);
    loop {
        buffer.shift();
        resp.reset();
        match parse_and_handle_request(&mut buffer, &mut tcp_reader, &mut resp).await {
            Err(HttpError::IoError(e)) => {
                println!("INFO server io_error={:?}", e);
                let _ = tcp_stream.shutdown(std::net::Shutdown::Both);
                break;
            }
            Err(HttpError::ParseError(e)) => {
                println!("INFO server parse_error={:?}", e);
                let _ = resp.send_without_body(e.status()).await;
            }
            Err(HttpError::ProcessingError(req, status)) => {
                println!("INFO server {:?} processing_error={:?}", req, status);
                let _ = resp.send_without_body(status).await;
            }
            Ok(req) => {
                println!("INFO server {:?} {:?}", req, resp);
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

    let response = reqwest::Client::new()
        .get("http://127.0.0.1:1690/path1")
        // .put("http://127.0.0.1:1690/path1")
        // .body("request-body1")
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
