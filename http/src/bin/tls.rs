// This program shows how to use TLS.

use std::println;
use std::sync::Arc;

/// AcceptSpecificCertsVerifier implements certificate pinning.
/// 
/// The rustls library has an open issue to add something like this:
/// "Implement support for certificate pinning" https://github.com/ctz/rustls/issues/227
struct AcceptSpecificCertsVerifier {
    certs: Vec<rustls::Certificate>,
}

impl rustls::ServerCertVerifier for AcceptSpecificCertsVerifier {
    fn verify_server_cert(
        &self,
        _roots: &rustls::RootCertStore,
        presented_certs: &[rustls::Certificate],
        _dns_name: webpki::DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        let presented_cert = &presented_certs[0];
        for cert in &self.certs {
            if presented_cert == cert {
                return Ok(rustls::ServerCertVerified::assertion());
            }
        }
        return Err(rustls::TLSError::WebPKIError(webpki::Error::UnknownIssuer));
    }
}

fn arbitrary_dns_name() -> webpki::DNSName {
    webpki::DNSNameRef::try_from_ascii_str("arbitrary1")
        .unwrap()
        .to_owned()
}

async fn async_main() -> () {
    let cert_pem_bytes = tokio::fs::read("localhost.cert").await.unwrap();
    let cert_pem = pem::parse(cert_pem_bytes).unwrap();
    assert_eq!(cert_pem.tag, "CERTIFICATE");
    let cert = rustls::Certificate(cert_pem.contents);

    let key_pem_bytes = tokio::fs::read("localhost.key").await.unwrap();
    let key_pem = pem::parse(key_pem_bytes).unwrap();
    let key = rustls::PrivateKey(key_pem.contents);
    assert_eq!(key_pem.tag, "PRIVATE KEY");

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let mut listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!(
        "INFO server listening on {}",
        listener.local_addr().unwrap()
    );

    let mut server_config = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    server_config
        .set_single_cert(vec![cert.clone()], key)
        .unwrap();
    let server_config_arc = Arc::new(server_config);

    tokio::spawn(async move {
        let tls_acceptor = tokio_rustls::TlsAcceptor::from(server_config_arc);
        loop {
            let (tcp_stream, _addr) = listener.accept().await.unwrap();
            let mut tls_stream = tls_acceptor.accept(tcp_stream).await.unwrap();
            use tokio::io::AsyncWriteExt;
            if let Err(e) = tls_stream.write_all(b"response").await {
                println!("WARN server write error: {:?}", e);
                return;
            }
        }
    });

    let mut client_config = rustls::ClientConfig::new();
    client_config
        .dangerous()
        .set_certificate_verifier(Arc::new(AcceptSpecificCertsVerifier { certs: vec![cert] }));
    let tls_connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));
    let tcp_stream = tokio::net::TcpStream::connect("127.0.0.1:1690")
        .await
        .unwrap();
    let mut tls_stream = tls_connector
        .connect(arbitrary_dns_name().as_ref(), tcp_stream)
        .await
        .unwrap();
    use tokio::io::AsyncReadExt;
    let mut buf = String::new();
    if let Err(e) = tls_stream.read_to_string(&mut buf).await {
        println!("WARN client read error: {:?}", e);
        return;
    }
    println!("INFO client read {:?}", buf);
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

// $ cargo run --bin tls
// INFO server listening on 127.0.0.1:1690
// INFO client read "response"
