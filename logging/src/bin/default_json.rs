use std::sync::Mutex;

use slog::{debug, error, info, trace, warn};

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
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

    logging::apple::error();
    logging::apple::warn();
    logging::apple::info();
    logging::apple::debug();
    logging::apple::trace();
    logging::apple::info_in_thread();

    // $ cargo run --bin default_json
    // {"msg":"main 1","level":"ERRO","ts":"2020-04-04T23:57:11.725122-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"WARN","ts":"2020-04-04T23:57:11.725705-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"INFO","ts":"2020-04-04T23:57:11.725733-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"DEBG","ts":"2020-04-04T23:57:11.725759-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"TRCE","ts":"2020-04-04T23:57:11.725785-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"using_log 1","level":"ERRO","ts":"2020-04-04T23:57:11.725810-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"WARN","ts":"2020-04-04T23:57:11.725834-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"INFO","ts":"2020-04-04T23:57:11.725860-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"DEBG","ts":"2020-04-04T23:57:11.725884-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"TRCE","ts":"2020-04-04T23:57:11.725908-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log in thread 1","level":"INFO","ts":"2020-04-04T23:57:11.725933-07:00","key_set_in_root_logger":1}
    // {"msg":"apple 1","level":"ERRO","ts":"2020-04-04T23:57:11.725957-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"WARN","ts":"2020-04-04T23:57:11.725981-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"INFO","ts":"2020-04-04T23:57:11.726006-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"DEBG","ts":"2020-04-04T23:57:11.726031-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"TRCE","ts":"2020-04-04T23:57:11.726056-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple in thread 1","level":"INFO","ts":"2020-04-04T23:57:11.726081-07:00","thread":"apple","key_set_in_root_logger":1,"x":2}
}
