use std::println;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{delay_for, Duration};

pub async fn wait_for_stop_signal() {
    // Handle TERM signal for running in Docker, Kubernetes, supervisord, etc.
    // Also handle INT signal from CTRL-C in dev terminal.
    use tokio::signal::unix::{signal, SignalKind};
    let mut term_signal =
        signal(SignalKind::terminate()).expect("Failed installing TERM signal handler");
    let mut int_signal =
        signal(SignalKind::interrupt()).expect("Failed installing INT signal handler");
    use tokio::stream::StreamExt;
    futures::future::select(term_signal.next(), int_signal.next()).await;
}

pub struct StopperController {
    signal_rx: mpsc::Receiver<()>,
    tracker_rx: mpsc::Receiver<()>,
}

impl StopperController {
    fn signal_stop(&mut self) {
        self.signal_rx.close();
    }

    async fn wait(&mut self) {
        self.tracker_rx.recv().await;
    }
}

#[derive(Clone)]
pub struct Stopper {
    signal_tx: mpsc::Sender<()>,
    tracker_tx: mpsc::Sender<()>,
}

impl Stopper {
    async fn wait(&mut self) {
        // The send() call always waits because new_stopper() fills the channel.
        match self.signal_tx.send(()).await {
            Ok(_) => {}
            Err(_) => {
                return;
            }
        }
    }

    fn is_signalled(&mut self) -> bool {
        match self.signal_tx.try_send(()) {
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => true,
            _ => false,
        }
    }
}

pub fn new_stopper() -> (StopperController, Stopper) {
    let (mut signal_tx, signal_rx) = mpsc::channel(1);
    signal_tx.try_send(()).unwrap(); // Fill the channel so senders wait.
    let (tracker_tx, tracker_rx) = mpsc::channel(1);
    (
        StopperController {
            signal_rx,
            tracker_rx,
        },
        Stopper {
            signal_tx,
            tracker_tx,
        },
    )
}

async fn handle_conn(mut tcp_stream: TcpStream, mut stopper: Stopper) {
    println!("INFO server handling connection, writing response slowly");
    delay_for(Duration::from_secs(1)).await;
    if stopper.is_signalled() {
        println!("INFO server shutting down, handle_conn returning early");
        return;
    }
    println!("INFO server writing 'response1 '");
    use tokio::io::AsyncWriteExt;
    while let Err(e) = tcp_stream.write_all(b"response1 ").await {
        println!("WARN server write error: {:?}", e);
    }
    delay_for(Duration::from_secs(1)).await;
    if stopper.is_signalled() {
        println!("INFO server shutting down, handle_conn returning early");
        return;
    }
    println!("INFO server writing 'response2 '");
    while let Err(e) = tcp_stream.write_all(b"response2 ").await {
        println!("WARN server write error: {:?}", e);
    }
    delay_for(Duration::from_secs(1)).await;
    if stopper.is_signalled() {
        println!("INFO server shutting down, handle_conn returning early");
        return;
    }
    println!("INFO server writing 'response3'");
    while let Err(e) = tcp_stream.write_all(b"response3").await {
        println!("WARN server write error: {:?}", e);
    }
}

async fn accept_loop(mut listener: TcpListener, mut stopper: Stopper) {
    loop {
        tokio::select! {
            listen_result = listener.accept() => {
                match listen_result {
                    Ok((tcp_stream, addr)) => {
                        println!("INFO accepted {}", addr);
                        tokio::spawn(handle_conn(tcp_stream, stopper.clone()));
                    }
                    Err(e) => {
                        println!("WARN Failed accepting connection from socket: {:?}", e);
                        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                    }
                }
            },
            _ = stopper.wait() => {
                println!("INFO exiting accept_loop");
                break
            },
        }
    }
}

async fn listen_on_all_interfaces(port: u16) -> tokio::io::Result<TcpListener> {
    let interface =
        std::net::IpAddr::from(std::net::Ipv6Addr::UNSPECIFIED /* includes ipv4 */);
    let addr = std::net::SocketAddr::from((interface, port));
    tokio::net::TcpListener::bind(&addr).await
}

async fn call_server(addr: &str) {
    println!("INFO client connecting to {}", addr);
    let mut tcp_stream = match tokio::net::TcpStream::connect(addr).await {
        Ok(tcp_stream) => tcp_stream,
        Err(e) => {
            println!("WARN client connect error: {:?}", e);
            return;
        }
    };
    println!(
        "INFO client connected to {}",
        tcp_stream.peer_addr().unwrap()
    );
    while let Err(e) = tcp_stream.shutdown(std::net::Shutdown::Write) {
        println!("WARN client stream shutdown-write error: {:?}", e);
    }
    use tokio::io::AsyncReadExt;
    loop {
        use bytes::BytesMut;
        let mut buffer = BytesMut::new();
        match tcp_stream.read_buf(&mut buffer).await {
            Ok(0) => {
                println!("INFO client connection EOF");
                return;
            }
            Ok(_) => {
                println!(
                    "INFO client read {:?}",
                    std::str::from_utf8(&buffer).unwrap()
                );
            }
            Err(e) => {
                println!("WARN client read error: {:?}", e);
                return;
            }
        }
    }
}

async fn async_main() -> () {
    let listener = listen_on_all_interfaces(1690).await.unwrap();
    println!(
        "INFO server listening on {}",
        listener.local_addr().unwrap()
    );
    let (mut stopper_controller, stopper) = new_stopper();
    tokio::spawn(accept_loop(listener, stopper));

    tokio::spawn(async {
        call_server("127.0.0.1:1690").await;
        call_server("[::1]:1690").await;
    });

    // Comment this out to test ^C handling.
    tokio::spawn(async {
        delay_for(Duration::from_secs(1)).await;
        println!("INFO sending TERM signal to self");
        nix::sys::signal::kill(nix::unistd::getpid(), nix::sys::signal::SIGTERM).unwrap();
    });

    println!("INFO main task waiting for stop signal");
    wait_for_stop_signal().await;
    println!("INFO main got stop signal, signalling all tasks to stop");
    stopper_controller.signal_stop();
    println!("INFO main waiting for tasks to stop");
    tokio::select! {
        _ = stopper_controller.wait() => {},
        _ = delay_for(Duration::from_secs(1)) => {},
    }
    println!("INFO exiting");
}

pub fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async_main());
    // Drops waiting tasks.  Waits for all busy tasks to await and drops them.  Gives up after timeout.
    runtime.shutdown_timeout(std::time::Duration::from_millis(1));
}

// $ cargo run --bin graceful_shutdown
// INFO server listening on [::]:1690
// INFO main task waiting for stop signal
// INFO client connecting to 127.0.0.1:1690
// INFO accepted [::ffff:127.0.0.1]:51972
// INFO client connected to 127.0.0.1:1690
// INFO server handling connection, writing response slowly
// INFO server writing 'response1 '
// INFO sending TERM signal to self
// INFO client read "response1 "
// INFO main got stop signal, signalling all tasks to stop
// INFO main waiting for tasks to stop
// INFO exiting accept_loop
// INFO server shutting down, handle_conn returning early
// INFO exiting
// INFO client connection EOF
// INFO client connecting to [::1]:1690
// WARN client connect error: Os { code: 61, kind: ConnectionRefused, message: "Connection refused" }
// $

// $ cargo run --bin graceful_shutdown
// INFO server listening on [::]:1690
// INFO main task waiting for stop signal
// INFO client connecting to 127.0.0.1:1690
// INFO accepted [::ffff:127.0.0.1]:51946
// INFO client connected to 127.0.0.1:1690
// INFO server handling connection, writing response slowly
// INFO server writing 'response1 '
// INFO client read "response1 "
// INFO server writing 'response2 '
// INFO client read "response2 "
// INFO server writing 'response3'
// INFO client read "response3"
// INFO client connection EOF
// INFO client connecting to [::1]:1690
// INFO accepted [::1]:51947
// INFO client connected to [::1]:1690
// INFO server handling connection, writing response slowly
// INFO server writing 'response1 '
// INFO client read "response1 "
// INFO server writing 'response2 '
// INFO client read "response2 "
// INFO server writing 'response3'
// INFO client read "response3"
// INFO client connection EOF
// ^CINFO main got stop signal, signalling all tasks to stop
// INFO main waiting for tasks to stop
// INFO exiting accept_loop
// INFO exiting
// $

// $ cargo run --bin graceful_shutdown
// INFO server listening on [::]:1690
// INFO main task waiting for stop signal
// INFO client connecting to 127.0.0.1:1690
// INFO accepted [::ffff:127.0.0.1]:51948
// INFO server handling connection, writing response slowly
// INFO client connected to 127.0.0.1:1690
// INFO server writing 'response1 '
// INFO client read "response1 "
// ^CINFO main got stop signal, signalling all tasks to stop
// INFO main waiting for tasks to stop
// INFO exiting accept_loop
// INFO server shutting down, handle_conn returning early
// INFO exiting
// INFO client connection EOF
// INFO client connecting to [::1]:1690
// WARN client connect error: Os { code: 61, kind: ConnectionRefused, message: "Connection refused" }
// $

// $ cargo run --bin graceful_shutdown
// INFO server listening on [::]:1690
// INFO main task waiting for stop signal
// INFO client connecting to 127.0.0.1:1690
// INFO accepted [::ffff:127.0.0.1]:51944
// INFO server handling connection, writing response slowly
// INFO client connected to 127.0.0.1:1690
// INFO server writing 'response1 '
// INFO client read "response1 "
// INFO server writing 'response2 '
// INFO client read "response2 "
// INFO server writing 'response3'
// INFO client read "response3"
// INFO client connection EOF
// INFO client connecting to [::1]:1690
// INFO client connected to [::1]:1690
// INFO accepted [::1]:51945
// INFO server handling connection, writing response slowly
// ^CINFO main got stop signal, signalling all tasks to stop
// INFO main waiting for tasks to stop
// INFO exiting accept_loop
// INFO server shutting down, handle_conn returning early
// INFO exiting
// INFO client connection EOF
// $
