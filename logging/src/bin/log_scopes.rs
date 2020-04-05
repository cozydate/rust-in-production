use std::sync::Mutex;

use slog::{debug, error, info, trace, warn};

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!("process" => "log_scopes"));
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();

    slog_scope::scope(&slog_scope::logger().new(slog::o!("thread" => "main")), || {
        error!(slog_scope::logger(), "main {}", 1; "x" => 2);
        warn!(slog_scope::logger(), "main {}", 1; "x" => 2);
        info!(slog_scope::logger(), "main {}", 1; "x" => 2);
        debug!(slog_scope::logger(), "main {}", 1; "x" => 2);
        trace!(slog_scope::logger(), "main {}", 1; "x" => 2);

        logging::using_log::error();
        logging::using_log::warn();
        logging::using_log::info();
        logging::using_log::debug();
        logging::using_log::trace();
        logging::using_log::info_in_thread();

        logging::apple::error();
        logging::apple::warn();
        logging::apple::info();
        logging::apple::debug();
        logging::apple::trace();
        logging::apple::info_in_thread();
    });

    // $ cargo run --bin log_scopes
    // {"msg":"main 1","level":"ERRO","ts":"2020-04-05T00:01:41.260671-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"main 1","level":"WARN","ts":"2020-04-05T00:01:41.261486-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"main 1","level":"INFO","ts":"2020-04-05T00:01:41.261536-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"main 1","level":"DEBG","ts":"2020-04-05T00:01:41.261576-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"main 1","level":"TRCE","ts":"2020-04-05T00:01:41.261615-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_log 1","level":"ERRO","ts":"2020-04-05T00:01:41.261654-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log 1","level":"WARN","ts":"2020-04-05T00:01:41.261691-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log 1","level":"INFO","ts":"2020-04-05T00:01:41.261728-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log 1","level":"DEBG","ts":"2020-04-05T00:01:41.261765-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log 1","level":"TRCE","ts":"2020-04-05T00:01:41.261801-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log in thread 1","level":"INFO","ts":"2020-04-05T00:01:41.261852-07:00","process":"log_scopes"}
    // {"msg":"apple 1","level":"ERRO","ts":"2020-04-05T00:01:41.261892-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"apple 1","level":"WARN","ts":"2020-04-05T00:01:41.261932-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"apple 1","level":"INFO","ts":"2020-04-05T00:01:41.261971-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"apple 1","level":"DEBG","ts":"2020-04-05T00:01:41.262010-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"apple 1","level":"TRCE","ts":"2020-04-05T00:01:41.262048-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"apple in thread 1","level":"INFO","ts":"2020-04-05T00:01:41.262088-07:00","thread":"apple","process":"log_scopes","x":2}
}
