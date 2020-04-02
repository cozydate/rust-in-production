use std::sync::Mutex;

use slog::{debug, info};

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
    let drain = slog::LevelFilter(drain, slog::Level::Info);
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);

    info!(slog_scope::logger(), "main");
    debug!(slog_scope::logger(), "main");  // Suppressed.
    logging::apple::info();
    logging::apple::debug();  // Suppressed.

    // $ cargo run --bin log_level_in_code
    // {"msg":"main","level":"INFO","ts":"2020-04-02T09:28:01.920450-07:00"}
    // {"msg":"apple 1","level":"INFO","ts":"2020-04-02T09:28:01.921396-07:00","x":2}
}
