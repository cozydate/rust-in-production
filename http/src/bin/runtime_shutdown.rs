// This program shows how to safely shut down a Tokio async task executor.

pub async fn async_main() -> () {
    println!("Starting task");
    tokio::spawn(async {
        loop {
            println!("Task");
            tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
        }
    });
    println!("Starting thread");
    tokio::task::spawn_blocking(|| loop {
        println!("Thread");
        std::thread::sleep(std::time::Duration::from_secs(1));
    });
    println!("async_main() returning");
}

pub fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async_main());
    println!("Shutting down Tokio runtime");
    // Drops waiting tasks.  Waits for all busy tasks to await and drops them.  Gives up after timeout.
    runtime.shutdown_timeout(std::time::Duration::from_secs(3));
    println!("main() returning");
}

// $ cargo run --bin runtime_shutdown
// Starting task
// Starting thread
// async_main() returning
// Shutting down Tokio runtime
// Task
// Thread
// Thread
// Task
// Thread
// Task
// main() returning
