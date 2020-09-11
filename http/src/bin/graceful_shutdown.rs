// This program shows how to shutdown a server that is serving clients.
//
// You can always just kill a server process.
// - Requests with multiple steps will stop in the middle.
//   - The server may leave data in an inconsistent state.
//   - The server may repeat some operations, like sending an email.
//   - When there are bugs, the server may skip operations or lose data.
// - Clients doing big data uploads will get cut off and have to restart their
//   uploads.  It's the same for large downloads.
// - Log messages and metrics get lost.
//
// To prevent these problems, it's better to shut down the server gracefully:
// 1. First stop accepting new connections.
// 1. Signal all request handlers and background jobs to stop.
//    They can reach a safe stopping place and then stop.
// 1. Wait a bit to give them time to see the signal and respond.
// 1. Finally, kill the server process.

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
    println!("INFO handler writing response slowly");
    for n in 0..3 {
        delay_for(Duration::from_secs(1)).await;
        if stopper.is_signalled() {
            println!("INFO handler returning early because stopper is signalled");
            return;
        }
        use tokio::io::AsyncWriteExt;
        if let Err(e) = tcp_stream.write_all(n.to_string().as_bytes()).await {
            println!("WARN server write error: {:?}", e);
            return;
        }
    }
}

async fn accept_loop(mut listener: TcpListener, mut stopper: Stopper) {
    loop {
        tokio::select! {
            listen_result = listener.accept() => {
                match listen_result {
                    Ok((tcp_stream, addr)) => {
                        println!("INFO server accepted connection {}", addr);
                        tokio::spawn(handle_conn(tcp_stream, stopper.clone()));
                    }
                    Err(e) => {
                        println!("WARN server error accepting connection: {:?}", e);
                        tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                    }
                }
            },
            _ = stopper.wait() => {
                println!("INFO server exiting accept_loop");
                break
            },
        }
    }
}

async fn call_server(addr: &str) {
    let mut tcp_stream = tokio::net::TcpStream::connect(addr).await.unwrap();
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
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 1690));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("INFO listening on {}", listener.local_addr().unwrap());
    let (mut stopper_controller, stopper) = new_stopper();
    tokio::spawn(accept_loop(listener, stopper));
    tokio::spawn(call_server("127.0.0.1:1690"));

    // Comment this out to test ^C handling.
    tokio::spawn(async {
        delay_for(Duration::from_millis(1500)).await;
        println!("INFO sending TERM signal to self");
        nix::sys::signal::kill(nix::unistd::getpid(), nix::sys::signal::SIGTERM).unwrap();
    });

    println!("INFO main waiting for stop signal");
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
    runtime.shutdown_background();
}

// $ cargo run --bin graceful_shutdown
// INFO listening on 127.0.0.1:1690
// INFO main waiting for stop signal
// INFO server accepted connection 127.0.0.1:56383
// INFO handler writing response slowly
// INFO client read "0"
// INFO sending TERM signal to self
// INFO main got stop signal, signalling all tasks to stop
// INFO main waiting for tasks to stop
// INFO server exiting accept_loop
// INFO handler returning early because stopper is signalled
// INFO exiting
// INFO client connection EOF

// $ cargo run --bin graceful_shutdown
// INFO listening on 127.0.0.1:1690
// INFO main waiting for stop signal
// INFO server accepted connection 127.0.0.1:56384
// INFO handler writing response slowly
// INFO client read "0"
// ^CINFO main got stop signal, signalling all tasks to stop
// INFO main waiting for tasks to stop
// INFO server exiting accept_loop
// INFO handler returning early because stopper is signalled
// INFO exiting

// $ cargo run --bin graceful_shutdown
// INFO listening on 127.0.0.1:1690
// INFO main waiting for stop signal
// INFO server accepted connection 127.0.0.1:56385
// INFO handler writing response slowly
// INFO client read "0"
// INFO client read "1"
// INFO client read "2"
// INFO client connection EOF
// ^CINFO main got stop signal, signalling all tasks to stop
// INFO main waiting for tasks to stop
// INFO server exiting accept_loop
// INFO exiting
