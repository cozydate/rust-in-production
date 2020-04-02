use std::sync::Mutex;

use slog::{debug, error, info, trace, warn};
use slog::Drain;

fn main() {
    let drain = Mutex::new(slog_json::Json::default(std::io::stdout())).map(slog::Fuse);
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, slog::o!("key_set_in_root_logger" => 1));
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();

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

    logging::using_slog::error();
    logging::using_slog::warn();
    logging::using_slog::info();
    logging::using_slog::debug();
    logging::using_slog::trace();
    logging::using_slog::info_in_thread();

    // $ cargo run --bin default_json
    // {"msg":"main 1","level":"ERRO","ts":"2020-04-01T22:55:28.215560-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"WARN","ts":"2020-04-01T22:55:28.216221-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"INFO","ts":"2020-04-01T22:55:28.216248-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"DEBG","ts":"2020-04-01T22:55:28.216272-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"using_log 1","level":"ERRO","ts":"2020-04-01T22:55:28.216297-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"WARN","ts":"2020-04-01T22:55:28.216321-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"INFO","ts":"2020-04-01T22:55:28.216344-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"DEBG","ts":"2020-04-01T22:55:28.216368-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"TRCE","ts":"2020-04-01T22:55:28.216391-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log in thread 1","level":"INFO","ts":"2020-04-01T22:55:28.216415-07:00","key_set_in_root_logger":1}
    // {"msg":"using_slog 1","level":"ERRO","ts":"2020-04-01T22:55:28.216438-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"using_slog 1","level":"WARN","ts":"2020-04-01T22:55:28.216462-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"using_slog 1","level":"INFO","ts":"2020-04-01T22:55:28.216486-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"using_slog 1","level":"DEBG","ts":"2020-04-01T22:55:28.216510-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"using_slog in thread 1","level":"INFO","ts":"2020-04-01T22:55:28.216534-07:00","key_set_for_thread":1,"key_set_in_root_logger":1,"x":2}
}
