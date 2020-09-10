use std::sync::Mutex;
use std::time::SystemTime;

use slog::{debug, error, info, trace, warn, FnValue, Level};

fn main() {
    // https://github.com/slog-rs/bunyan/blob/master/lib.rs
    let time_fn =
        |_: &slog::Record| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let time_ns_fn = |_: &slog::Record| {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            // Duration default is 0 seconds.
            .unwrap_or_default()
            // Nanoseconds overflow u64 in the year 2554.
            .as_nanos() as u64
    };
    let module_fn = |record: &slog::Record| record.module();
    let level_fn = |record: &slog::Record| match record.level() {
        slog::Level::Critical => "ERROR",
        Level::Error => "ERROR",
        Level::Warning => "WARN",
        Level::Info => "INFO",
        Level::Debug => "DEBUG",
        Level::Trace => "TRACE",
    };
    let message_fn = |record: &slog::Record| record.msg().to_string();
    let drain = slog_json::Json::new(std::io::stdout())
        .add_key_value(slog::o!(
            // Fields are in reverse order.
            "message" => FnValue(message_fn),
            "level" => FnValue(level_fn),
            "module" => FnValue(module_fn),
            "time" => FnValue(time_fn),
            "time_ns" => FnValue(time_ns_fn),
            // TODONT(mleonhard) Don't include 'process' or 'host'.  Supervisor and collector will
            // add these values and will not trust any values already present.
        ))
        .build();
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();

    slog_scope::scope(
        &slog_scope::logger().new(slog::o!("thread" => "main")),
        || {
            error!(slog_scope::logger(), "main"; "x" => 2);
            warn!(slog_scope::logger(), "main"; "x" => 2);
            info!(slog_scope::logger(), "main"; "x" => 2);
            debug!(slog_scope::logger(), "main"; "x" => 2);
            trace!(slog_scope::logger(), "main"; "x" => 2);

            logging::using_log::info();
            logging::using_log::info_in_thread();

            logging::using_slog::info();
            logging::using_slog::info_in_thread();

            logging::using_slog_scope::info();
            logging::using_slog_scope::info_in_thread();
        },
    );
}

// $ cargo run --bin custom_json
// {"time_ns":1599776885581851000,"time":"2020-09-10T22:28:05.581888000Z","module":"custom_json","level":"ERROR","message":"main","thread":"main","x":2}
// {"time_ns":1599776885582028000,"time":"2020-09-10T22:28:05.582029000Z","module":"custom_json","level":"WARN","message":"main","thread":"main","x":2}
// {"time_ns":1599776885582093000,"time":"2020-09-10T22:28:05.582094000Z","module":"custom_json","level":"INFO","message":"main","thread":"main","x":2}
// {"time_ns":1599776885582160000,"time":"2020-09-10T22:28:05.582180000Z","module":"custom_json","level":"DEBUG","message":"main","thread":"main","x":2}
// {"time_ns":1599776885582241000,"time":"2020-09-10T22:28:05.582243000Z","module":"custom_json","level":"TRACE","message":"main","thread":"main","x":2}
// {"time_ns":1599776885582269000,"time":"2020-09-10T22:28:05.582271000Z","module":"logging::using_log","level":"INFO","message":"using_log 1","thread":"main"}
// {"time_ns":1599776885582308000,"time":"2020-09-10T22:28:05.582326000Z","module":"logging::using_log","level":"INFO","message":"using_log in thread 1"}
// {"time_ns":1599776885582347000,"time":"2020-09-10T22:28:05.582348000Z","module":"logging::using_slog","level":"INFO","message":"using_slog 1","thread":"main","x":2}
// {"time_ns":1599776885582371000,"time":"2020-09-10T22:28:05.582372000Z","module":"logging::using_slog","level":"INFO","message":"using_slog in thread 1","thread":"using_slog","x":2}
// {"time_ns":1599776885582396000,"time":"2020-09-10T22:28:05.582397000Z","module":"logging::using_slog_scope","level":"INFO","message":"using_slog_scope 1","thread":"main","x":2}
// {"time_ns":1599776885582421000,"time":"2020-09-10T22:28:05.582422000Z","module":"logging::using_slog_scope","level":"INFO","message":"using_slog_scope in thread 1","thread":"using_slog_scope","x":2}
