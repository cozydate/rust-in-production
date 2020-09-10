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

    logging::using_slog::error();
    logging::using_slog::warn();
    logging::using_slog::info();
    logging::using_slog::debug();
    logging::using_slog::trace();
    logging::using_slog::info_in_thread();

    logging::using_slog_scope::error();
    logging::using_slog_scope::warn();
    logging::using_slog_scope::info();
    logging::using_slog_scope::debug();
    logging::using_slog_scope::trace();
    logging::using_slog_scope::info_in_thread();
}

// $ cargo run --bin default_json
// {"msg":"main 1","level":"ERRO","ts":"2020-09-10T15:29:25.334792-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"main 1","level":"WARN","ts":"2020-09-10T15:29:25.335430-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"main 1","level":"INFO","ts":"2020-09-10T15:29:25.335463-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"main 1","level":"DEBG","ts":"2020-09-10T15:29:25.335492-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"main 1","level":"TRCE","ts":"2020-09-10T15:29:25.335520-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_log 1","level":"ERRO","ts":"2020-09-10T15:29:25.335548-07:00","key_set_in_root_logger":1}
// {"msg":"using_log 1","level":"WARN","ts":"2020-09-10T15:29:25.335575-07:00","key_set_in_root_logger":1}
// {"msg":"using_log 1","level":"INFO","ts":"2020-09-10T15:29:25.335602-07:00","key_set_in_root_logger":1}
// {"msg":"using_log 1","level":"DEBG","ts":"2020-09-10T15:29:25.335629-07:00","key_set_in_root_logger":1}
// {"msg":"using_log 1","level":"TRCE","ts":"2020-09-10T15:29:25.335655-07:00","key_set_in_root_logger":1}
// {"msg":"using_log in thread 1","level":"INFO","ts":"2020-09-10T15:29:25.335682-07:00","key_set_in_root_logger":1}
// {"msg":"using_slog 1","level":"ERRO","ts":"2020-09-10T15:29:25.335709-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog 1","level":"WARN","ts":"2020-09-10T15:29:25.335737-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog 1","level":"INFO","ts":"2020-09-10T15:29:25.335765-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog 1","level":"DEBG","ts":"2020-09-10T15:29:25.335793-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog 1","level":"TRCE","ts":"2020-09-10T15:29:25.335821-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog in thread 1","level":"INFO","ts":"2020-09-10T15:29:25.335850-07:00","thread":"using_slog","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog_scope 1","level":"ERRO","ts":"2020-09-10T15:29:25.335881-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog_scope 1","level":"WARN","ts":"2020-09-10T15:29:25.335910-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog_scope 1","level":"INFO","ts":"2020-09-10T15:29:25.335943-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog_scope 1","level":"DEBG","ts":"2020-09-10T15:29:25.336002-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog_scope 1","level":"TRCE","ts":"2020-09-10T15:29:25.336026-07:00","key_set_in_root_logger":1,"x":2}
// {"msg":"using_slog_scope in thread 1","level":"INFO","ts":"2020-09-10T15:29:25.336051-07:00","thread":"using_slog_scope","key_set_in_root_logger":1,"x":2}
