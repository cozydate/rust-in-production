use std::sync::Mutex;
use std::time::SystemTime;

use slog::{debug, error, FnValue, info, Level, trace, warn};

fn main() {
    // https://github.com/slog-rs/bunyan/blob/master/lib.rs
    let time_fn =
        |_: &slog::Record|
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let time_ns_fn =
        |_: &slog::Record|
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                // Duration default is 0 seconds.
                .unwrap_or_default()
                // Nanoseconds overflow u64 in the year 2554.
                .as_nanos() as u64;
    let module_fn = |record: &slog::Record|
        record.module();
    let level_fn =
        |record: &slog::Record|
            match record.level() {
                slog::Level::Critical => "ERROR",
                Level::Error => "ERROR",
                Level::Warning => "WARN",
                Level::Info => "INFO",
                Level::Debug => "DEBUG",
                Level::Trace => "TRACE",
            };
    let message_fn =
        |record: &slog::Record|
            record.msg().to_string();
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
        )).build();
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();

    slog_scope::scope(&slog_scope::logger().new(slog::o!("thread" => "main")), || {
        error!(slog_scope::logger(), "main"; "x" => 2);
        warn!(slog_scope::logger(), "main"; "x" => 2);
        info!(slog_scope::logger(), "main"; "x" => 2);
        debug!(slog_scope::logger(), "main"; "x" => 2);
        trace!(slog_scope::logger(), "main"; "x" => 2);

        logging::using_log::info();
        logging::using_log::info_in_thread();

        logging::apple::info();
        logging::apple::info_in_thread();
    });

    // $ cargo run --bin custom_json
    // {"time_ns":1586384276407166000,"time":"2020-04-08T22:17:56.407195000Z","module":"custom_json","level":"ERROR","message":"main","thread":"main","x":2}
    // {"time_ns":1586384276407298000,"time":"2020-04-08T22:17:56.407300000Z","module":"custom_json","level":"WARN","message":"main","thread":"main","x":2}
    // {"time_ns":1586384276407329000,"time":"2020-04-08T22:17:56.407331000Z","module":"custom_json","level":"INFO","message":"main","thread":"main","x":2}
    // {"time_ns":1586384276407362000,"time":"2020-04-08T22:17:56.407363000Z","module":"custom_json","level":"DEBUG","message":"main","thread":"main","x":2}
    // {"time_ns":1586384276407389000,"time":"2020-04-08T22:17:56.407391000Z","module":"custom_json","level":"TRACE","message":"main","thread":"main","x":2}
    // {"time_ns":1586384276407417000,"time":"2020-04-08T22:17:56.407418000Z","module":"logging::using_log","level":"INFO","message":"using_log 1","thread":"main"}
    // {"time_ns":1586384276407443000,"time":"2020-04-08T22:17:56.407444000Z","module":"logging::using_log","level":"INFO","message":"using_log in thread 1"}
    // {"time_ns":1586384276407468000,"time":"2020-04-08T22:17:56.407469000Z","module":"logging::apple","level":"INFO","message":"apple 1","thread":"main","x":2}
    // {"time_ns":1586384276407495000,"time":"2020-04-08T22:17:56.407496000Z","module":"logging::apple","level":"INFO","message":"apple in thread 1","thread":"apple","x":2}
}
