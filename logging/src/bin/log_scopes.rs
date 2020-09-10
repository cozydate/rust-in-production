use std::sync::Mutex;

use slog::{debug, error, info, trace, warn};

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!("process" => "log_scopes"));
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();

    slog_scope::scope(
        &slog_scope::logger().new(slog::o!("thread" => "main")),
        || {
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
        },
    );

    // $ cargo run --bin log_scopes
    // {"msg":"main 1","level":"ERRO","ts":"2020-09-10T15:05:22.395346-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"main 1","level":"WARN","ts":"2020-09-10T15:05:22.395968-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"main 1","level":"INFO","ts":"2020-09-10T15:05:22.395996-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"main 1","level":"DEBG","ts":"2020-09-10T15:05:22.396021-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"main 1","level":"TRCE","ts":"2020-09-10T15:05:22.396046-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_log 1","level":"ERRO","ts":"2020-09-10T15:05:22.396070-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log 1","level":"WARN","ts":"2020-09-10T15:05:22.396093-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log 1","level":"INFO","ts":"2020-09-10T15:05:22.396117-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log 1","level":"DEBG","ts":"2020-09-10T15:05:22.396140-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log 1","level":"TRCE","ts":"2020-09-10T15:05:22.396163-07:00","thread":"main","process":"log_scopes"}
    // {"msg":"using_log in thread 1","level":"INFO","ts":"2020-09-10T15:05:22.396186-07:00","process":"log_scopes"}
    // {"msg":"using_slog 1","level":"ERRO","ts":"2020-09-10T15:05:22.396208-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog 1","level":"WARN","ts":"2020-09-10T15:05:22.396281-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog 1","level":"INFO","ts":"2020-09-10T15:05:22.396390-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog 1","level":"DEBG","ts":"2020-09-10T15:05:22.396462-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog 1","level":"TRCE","ts":"2020-09-10T15:05:22.396505-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog in thread 1","level":"INFO","ts":"2020-09-10T15:05:22.396530-07:00","thread":"using_slog","process":"log_scopes","x":2}
    // {"msg":"using_slog_scope 1","level":"ERRO","ts":"2020-09-10T15:05:22.396556-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog_scope 1","level":"WARN","ts":"2020-09-10T15:05:22.396581-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog_scope 1","level":"INFO","ts":"2020-09-10T15:05:22.396606-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog_scope 1","level":"DEBG","ts":"2020-09-10T15:05:22.396630-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog_scope 1","level":"TRCE","ts":"2020-09-10T15:05:22.396655-07:00","thread":"main","process":"log_scopes","x":2}
    // {"msg":"using_slog_scope in thread 1","level":"INFO","ts":"2020-09-10T15:05:22.396680-07:00","thread":"using_slog_scope","process":"log_scopes","x":2}
}
