use std::sync::Mutex;
use std::time::SystemTime;

use slog::{debug, error, FnValue, info, Level, trace, warn};

fn main() {
    // https://github.com/slog-rs/bunyan/blob/master/lib.rs
    let host = hostname::get().unwrap_or_default().into_string().unwrap_or_default();
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
            "process" => "custom_json",
            "host" => host,
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
    // {"host":"mbp","process":"custom_json","time_ns":1585851354242507000,"time":"2020-04-02T18:15:54.242521000Z","module":"custom_json","level":"ERROR","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"custom_json","time_ns":1585851354242732000,"time":"2020-04-02T18:15:54.242734000Z","module":"custom_json","level":"WARN","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"custom_json","time_ns":1585851354242789000,"time":"2020-04-02T18:15:54.242791000Z","module":"custom_json","level":"INFO","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"custom_json","time_ns":1585851354242824000,"time":"2020-04-02T18:15:54.242825000Z","module":"custom_json","level":"DEBUG","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"custom_json","time_ns":1585851354242854000,"time":"2020-04-02T18:15:54.242855000Z","module":"custom_json","level":"TRACE","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"custom_json","time_ns":1585851354242925000,"time":"2020-04-02T18:15:54.242926000Z","module":"logging::using_log","level":"INFO","message":"using_log 1","thread":"main"}
    // {"host":"mbp","process":"custom_json","time_ns":1585851354242953000,"time":"2020-04-02T18:15:54.242954000Z","module":"logging::using_log","level":"INFO","message":"using_log in thread 1"}
    // {"host":"mbp","process":"custom_json","time_ns":1585851354242980000,"time":"2020-04-02T18:15:54.242981000Z","module":"logging::apple","level":"INFO","message":"apple 1","thread":"main","x":2}
    // {"host":"mbp","process":"custom_json","time_ns":1585851354243009000,"time":"2020-04-02T18:15:54.243010000Z","module":"logging::apple","level":"INFO","message":"apple in thread 1","thread":"apple","x":2}
}
