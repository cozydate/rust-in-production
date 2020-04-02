use std::sync::Mutex;

use slog::{debug, error, info, trace, warn};

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!("key_set_in_root_logger" => 1));
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

    // $ cargo run --bin default_json
    // {"msg":"main 1","level":"ERRO","ts":"2020-04-02T11:05:34.769807-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"WARN","ts":"2020-04-02T11:05:34.770483-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"INFO","ts":"2020-04-02T11:05:34.770516-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"DEBG","ts":"2020-04-02T11:05:34.770546-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"main 1","level":"TRCE","ts":"2020-04-02T11:05:34.770575-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"using_log 1","level":"ERRO","ts":"2020-04-02T11:05:34.770604-07:00","thread":"main","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"WARN","ts":"2020-04-02T11:05:34.770633-07:00","thread":"main","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"INFO","ts":"2020-04-02T11:05:34.770661-07:00","thread":"main","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"DEBG","ts":"2020-04-02T11:05:34.770689-07:00","thread":"main","key_set_in_root_logger":1}
    // {"msg":"using_log 1","level":"TRCE","ts":"2020-04-02T11:05:34.770717-07:00","thread":"main","key_set_in_root_logger":1}
    // {"msg":"using_log in thread 1","level":"INFO","ts":"2020-04-02T11:05:34.770746-07:00","key_set_in_root_logger":1}
    // {"msg":"apple 1","level":"ERRO","ts":"2020-04-02T11:05:34.770773-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"WARN","ts":"2020-04-02T11:05:34.770865-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"INFO","ts":"2020-04-02T11:05:34.770903-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"DEBG","ts":"2020-04-02T11:05:34.770934-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple 1","level":"TRCE","ts":"2020-04-02T11:05:34.770971-07:00","thread":"main","key_set_in_root_logger":1,"x":2}
    // {"msg":"apple in thread 1","level":"INFO","ts":"2020-04-02T11:05:34.771003-07:00","thread":"apple","key_set_in_root_logger":1,"x":2}
}
