// Opinionated JSON logging.  You should do logging like this.

use std::error::Error;
use std::ffi::OsString;

fn string_from(oss: OsString) -> Result<String, String> {
    oss.into_string().map_err(
        |oss| std::format!("Error decoding OsString {:?} to String", oss))
}

/// Configure `log` and `slog` to emit JSON to stdout.
///
/// Applies global and per-module filtering rules from `filters` and overrides them with
/// rules from the `RUST_LOG` environment variable.
/// The `filters` syntax is quite expressive.
/// See the [slog_envlogger docs](https://docs.rs/slog-envlogger/2.2.0/slog_envlogger/)
///
/// You can set the `DEV_LOG_FORMAT` environment variable to one of:
/// - `"json"` or `""` to log in JSON format.
/// - `"compact"` to print scope variables on their own line and indent log messages emitted inside
///   the scope.
/// - `"full"` to print scope variables on every line, with color.
/// - `"plain"` to print without color.
///
/// Release binaries always log JSON.  Only debug binaries check the `DEV_LOG_FORMAT` env var.
///
/// Examples:
/// - Set the default log level to `info`.
///   The program will emit log messages with level `info` and higher.
///   ```
///   let _global_logger_guard = configure_logging("prog1", "info");
///   info!("a message"; "some_data" => 123, "other_data" => "val1");
///   slog::info!(slog_scope::logger(), "a message"; "some_data" => 123, "other_data" => "val1");
///   log::info!("a message; some_data={} other_data={}", 123, "val1");
///   log::debug("some details");  // Not emitted
///   ```
/// - Set the default log level to `info` and set the level for `chatty::module1` to `warn`.
///   ```
///   let _global_logger_guard = configure_logging("prog1", "info,chatty::module1=warn");
///   ```
/// - Use the environment variable to Override default default log level.
///   `module1` still gets its special log level.
///   ```
///   std::env::set_var("RUST_LOG", "debug");
///   let _global_logger_guard = configure_logging("prog1", "info,module1=warn")
///   ```
/// - Use the environment variable to set `module1` to `debug`.
///   ```
///   std::env::set_var("RUST_LOG", "module1=debug");
///   let _global_logger_guard = configure_logging("prog1", "info")
///   ```
///
/// Example output:
/// ```json
/// {"host":"mbp","process":"prog1","time_ns":1585851354242507000, \
/// "time":"2020-04-02T18:15:54.242521000Z","module":"mod1","level":"ERROR","message":"msg1", \
/// "thread":"main","x":2}
/// ```
fn configure_logging(_process_name: &'static str, filters: &str)
                     -> Result<slog_scope::GlobalLoggerGuard, Box<dyn Error>>
{
    let _host = string_from(::hostname::get()?)?;
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
    let _time_fn = |w: &mut dyn std::io::Write|
        write!(w, "{}", chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
    #[cfg(debug_assertions)]  // Include the following statement in debug binaries, not release.
        let drain: Box<dyn slog::Drain<Ok=(), Err=std::io::Error> + Send> =
        match std::env::var("DEV_LOG_FORMAT").unwrap_or(String::new()).as_ref() {
            "" | "json" => Box::new(drain),
            "compact" => Box::new(
                slog_term::CompactFormat::new(slog_term::TermDecorator::new().build())
                    .use_custom_timestamp(_time_fn)
                    .build()),
            "full" => Box::new(
                slog_term::FullFormat::new(slog_term::TermDecorator::new().build())
                    .use_custom_timestamp(_time_fn)
                    .build()),
            "plain" => Box::new(
                slog_term::FullFormat::new(slog_term::PlainDecorator::new(std::io::stdout()))
                    .use_custom_timestamp(_time_fn)
                    .build()),
            s => panic!("Invalid DEV_LOG_FORMAT env var value {:?}", s)
        };
    let drain = slog_envlogger::LogBuilder::new(drain)
        .parse(filters)
        // Add any level overrides from environment variable
        .parse(
            &match std::env::var("RUST_LOG") {
                Ok(x) => Ok(x),
                Err(std::env::VarError::NotPresent) => Ok(String::new()),
                Err(x) => Err(x)
            }?
        )
        .build();
    let drain = slog::Fuse(std::sync::Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init()?;
    log_panics::init();
    Ok(_guard)
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
    let _global_logger_guard = configure_logging("opinion", "info").unwrap();
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

        panic!("uhoh");
    });

    // $ cargo run --bin opinion
    // {"host":"mbp","process":"opinion","time_ns":1586068677146422000,"time":"2020-04-05T06:37:57.146433000Z","module":"opinion","level":"ERROR","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"opinion","time_ns":1586068677146638000,"time":"2020-04-05T06:37:57.146640000Z","module":"opinion","level":"WARN","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"opinion","time_ns":1586068677146674000,"time":"2020-04-05T06:37:57.146675000Z","module":"opinion","level":"INFO","message":"main","thread":"main","x":2}
    // {"host":"mbp","process":"opinion","time_ns":1586068677146708000,"time":"2020-04-05T06:37:57.146709000Z","module":"logging::using_log","level":"INFO","message":"using_log 1","thread":"main"}
    // {"host":"mbp","process":"opinion","time_ns":1586068677146757000,"time":"2020-04-05T06:37:57.146759000Z","module":"logging::using_log","level":"INFO","message":"using_log in thread 1"}
    // {"host":"mbp","process":"opinion","time_ns":1586068677146787000,"time":"2020-04-05T06:37:57.146789000Z","module":"logging::apple","level":"INFO","message":"apple 1","thread":"main","x":2}
    // {"host":"mbp","process":"opinion","time_ns":1586068677146833000,"time":"2020-04-05T06:37:57.146834000Z","module":"logging::apple","level":"INFO","message":"apple in thread 1","thread":"apple","x":2}
    // {"host":"mbp","process":"opinion","time_ns":1586068677170857000,"time":"2020-04-05T06:37:57.170862000Z","module":"log_panics","level":"ERROR","message":"thread 'main' panicked at 'uhoh': src/bin/opinion.rs:161\n   0: backtrace::backtrace::trace_unsynchronized\n   1: backtrace::backtrace::trace\n   2: backtrace::capture::Backtrace::create\n   3: backtrace::capture::Backtrace::new\n   4: log_panics::init::{{closure}}\n   5: std::panicking::rust_panic_with_hook\n   6: std::panicking::begin_panic\n   7: opinion::main::{{closure}}\n   8: slog_scope::scope\n   9: opinion::thread_logging_scope\n  10: opinion::main\n  11: std::rt::lang_start::{{closure}}\n  12: std::panicking::try::do_call\n  13: __rust_maybe_catch_panic\n  14: std::rt::lang_start_internal\n  15: std::rt::lang_start\n  16: main\n","thread":"main"}

    // $ DEV_LOG_FORMAT=plain cargo run --bin opinion
    // 2020-04-05T00:37:48.675-07:00 ERRO main, x: 2, thread: main
    // 2020-04-05T00:37:48.676-07:00 WARN main, x: 2, thread: main
    // 2020-04-05T00:37:48.676-07:00 INFO main, x: 2, thread: main
    // 2020-04-05T00:37:48.676-07:00 INFO using_log 1, thread: main
    // 2020-04-05T00:37:48.676-07:00 INFO using_log in thread 1
    // 2020-04-05T00:37:48.676-07:00 INFO apple 1, x: 2, thread: main
    // 2020-04-05T00:37:48.676-07:00 INFO apple in thread 1, x: 2, thread: apple
    // 2020-04-05T00:37:48.701-07:00 ERRO thread 'main' panicked at 'uhoh': src/bin/opinion.rs:167
    //    0: backtrace::backtrace::trace_unsynchronized
    //    1: backtrace::backtrace::trace
    //    2: backtrace::capture::Backtrace::create
    //    3: backtrace::capture::Backtrace::new
    //    4: log_panics::init::{{closure}}
    //    5: std::panicking::rust_panic_with_hook
    //    6: std::panicking::begin_panic
    //    7: opinion::main::{{closure}}
    //    8: slog_scope::scope
    //    9: opinion::thread_logging_scope
    //   10: opinion::main
    //   11: std::rt::lang_start::{{closure}}
    //   12: std::panicking::try::do_call
    //   13: __rust_maybe_catch_panic
    //   14: std::rt::lang_start_internal
    //   15: std::rt::lang_start
    //   16: main
    // , thread: main
}
