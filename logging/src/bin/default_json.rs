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

    logging::apple::error();
    logging::apple::warn();
    logging::apple::info();
    logging::apple::debug();
    logging::apple::trace();
    logging::apple::info_in_thread();

    // $ cargo run --bin default_json
    // {"msg":"main 1","level":"ERRO","ts":"2020-04-02T09:27:02.887279-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"WARN","ts":"2020-04-02T09:27:02.888015-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"INFO","ts":"2020-04-02T09:27:02.888043-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"DEBG","ts":"2020-04-02T09:27:02.888068-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"using_log 1","level":"ERRO","ts":"2020-04-02T09:27:02.888092-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"WARN","ts":"2020-04-02T09:27:02.888123-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"INFO","ts":"2020-04-02T09:27:02.888147-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"DEBG","ts":"2020-04-02T09:27:02.888170-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"TRCE","ts":"2020-04-02T09:27:02.888193-07:00","key_set_in_root_logger":1}
    // {"msg":"using_log in thread 1","level":"INFO","ts":"2020-04-02T09:27:02.888216-07:00","key_set_in_root_logger":1}
    // {"msg":"apple 1","level":"ERRO","ts":"2020-04-02T09:27:02.888239-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"WARN","ts":"2020-04-02T09:27:02.888263-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"INFO","ts":"2020-04-02T09:27:02.888286-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"DEBG","ts":"2020-04-02T09:27:02.888310-07:00","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple in thread 1","level":"INFO","ts":"2020-04-02T09:27:02.888334-07:00","thread_scope_apple_origin":"USA","key_set_in_root_logger":1,"x":2}
}
