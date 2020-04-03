// Opinionated JSON logging.  You should do logging like this.

/// Configure `log` and `slog` to emit JSON to stdout.
///
/// Applies global and per-module filtering rules from `filters` and overrides them with
/// rules from the `RUST_LOG` environment variable.
/// The `filters` syntax is quite expressive.
/// See the [slog_envlogger docs](https://docs.rs/slog-envlogger/2.2.0/slog_envlogger/)
///
/// Examples:
/// - Set the default log level to `info`.
///   The program will emit log messages with level `info` and higher.
///   ```
///   let _global_logger_guard = configure_logging("info");
///   info!("a message"; "some_data" => 123, "other_data" => "val1");
///   slog::info!(slog_scope::logger(), "a message"; "some_data" => 123, "other_data" => "val1");
///   log::info!("a message; some_data={} other_data={}", 123, "val1");
///   log::debug("some details");  // Not emitted
///   ```
/// - Set the default log level to `info` and set the level for `chatty::module1` to `warn`.
///   ```
///   let _global_logger_guard = configure_logging("info,chatty::module1=warn");
///   ```
/// - Use the environment variable to Override default default log level.
///   `module1` still gets its special log level.
///   ```
///   std::env::set_var("RUST_LOG", "debug");
///   let _global_logger_guard = configure_logging("info,module1=warn")
///   ```
/// - Use the environment variable to set `module1` to `debug`.
///   ```
///   std::env::set_var("RUST_LOG", "module1=debug");
///   let _global_logger_guard = configure_logging("info")
///   ```
///
/// Example output:
/// ```json
/// {"host":"mbp","process":"opinion","time_ns":1585851354242507000, \
/// "time":"2020-04-02T18:15:54.242521000Z","module":"mod1","level":"ERROR","message":"msg1", \
/// "thread":"main","x":2}
/// ```
fn configure_logging(_process_name: &'static str, filters: &str) -> slog_scope::GlobalLoggerGuard {
    let _host = ::hostname::get()
        .unwrap()
        .into_string()
        .expect("Error converting hostname to UTF-8");
    let _time_fn =
        |_: &slog::Record|
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let _time_ns_fn =
        |_: &slog::Record|
            std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)
                // Use default Duration (0 seconds) if system time is before epoch.
                .unwrap_or_default()
                // Nanoseconds overflow u64 in the year 2554.
                .as_nanos() as u64;
    let _module_fn = |record: &slog::Record|
        record.module();
    let _level_fn =
        |record: &slog::Record|
            match record.level() {
                slog::Level::Critical => "ERROR",
                slog::Level::Error => "ERROR",
                slog::Level::Warning => "WARN",
                slog::Level::Info => "INFO",
                slog::Level::Debug => "DEBUG",
                slog::Level::Trace => "TRACE",
            };
    let _message_fn =
        |record: &slog::Record|
            record.msg().to_string();
    let drain = slog_json::Json::new(std::io::stdout())
        .add_key_value(slog::o!(
            // Fields are in reverse order.
            "message" => slog::FnValue(_message_fn),
            "level" => slog::FnValue(_level_fn),
            "module" => slog::FnValue(_module_fn),
            "time" => slog::FnValue(_time_fn),
            "time_ns" => slog::FnValue(_time_ns_fn),
            "process" => _process_name,
            "host" => _host,
        )).build();
    let drain = slog_envlogger::LogBuilder::new(drain)
        .parse(filters)
        // Add any level overrides from environment variable
        .parse(
            &match std::env::var("RUST_LOG") {
                Ok(x) => Ok(x),
                Err(std::env::VarError::NotPresent) => Ok(String::new()),
                Err(x) => Err(x)
            }.unwrap()
        )
        .build();
    let drain = slog::Fuse(std::sync::Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();
    _guard
}

fn thread_logging_scope<SF, R>(_name: &str, f: SF) -> R
    where SF: FnOnce() -> R {
    let logger = slog_scope::logger().new(slog::o!("thread" => String::from(_name)));
    slog_scope::scope(&logger, f)
}

macro_rules! error (
    ($($args:tt)+) => { slog::error!(slog_scope::logger(), $($args)+) };
);
macro_rules! warn (
    ($($args:tt)+) => { slog::warn!(slog_scope::logger(), $($args)+) };
);
macro_rules! info (
    ($($args:tt)+) => { slog::info!(slog_scope::logger(), $($args)+) };
);
macro_rules! debug (
    ($($args:tt)+) => { slog::debug!(slog_scope::logger(), $($args)+) };
);
macro_rules! trace (
    ($($args:tt)+) => { slog::trace!(slog_scope::logger(), $($args)+) };
);

fn main() {
    let _global_logger_guard = configure_logging("opinion", "info");
    thread_logging_scope("main", || {
        error!("main"; "x" => 2);
        warn!("main"; "x" => 2);
        info!("main"; "x" => 2);
        debug!("main"; "x" => 2);
        trace!("main"; "x" => 2);

        logging::using_log::info();
        logging::using_log::info_in_thread();

        logging::apple::info();
        logging::apple::info_in_thread();
    });

    // $ cargo run --bin opinion
    // {"host":"mbp","process":"opinion","time_ns":1585904687327521000,"time":"2020-04-03T09:04:47.327562000Z","module":"opinion","level":"ERROR","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"opinion","time_ns":1585904687327717000,"time":"2020-04-03T09:04:47.327719000Z","module":"opinion","level":"WARN","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"opinion","time_ns":1585904687327751000,"time":"2020-04-03T09:04:47.327753000Z","module":"opinion","level":"INFO","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"opinion","time_ns":1585904687327783000,"time":"2020-04-03T09:04:47.327784000Z","module":"logging::using_log","level":"INFO","message":"using_log 1","thread":"main"}
    // {"host":"mbp","process":"opinion","time_ns":1585904687327812000,"time":"2020-04-03T09:04:47.327813000Z","module":"logging::using_log","level":"INFO","message":"using_log in thread 1"}
    // {"host":"mbp","process":"opinion","time_ns":1585904687327840000,"time":"2020-04-03T09:04:47.327841000Z","module":"logging::apple","level":"INFO","message":"apple 1","thread":"main","x":2}
    // {"host":"mbp","process":"opinion","time_ns":1585904687327870000,"time":"2020-04-03T09:04:47.327872000Z","module":"logging::apple","level":"INFO","message":"apple in thread 1","thread":"apple","x":2}
}
