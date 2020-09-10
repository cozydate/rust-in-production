use slog::*;
use std::error::Error;
use std::future::Future;
use std::result::Result;

pub mod apple;
pub mod banana;
pub mod using_log;
pub mod using_slog;
pub mod using_slog_scope;

pub enum OutputFormat {
    JSON,
    Compact,
    Full,
    Plain,
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
///   let _global_logger_guard = logging::configure("info");
///   logging::info!("a message"; "some_data" => 123, "other_data" => "val1");
///   slog::info!(slog_scope::logger(), "a message"; "some_data" => 123, "other_data" => "val1");
///   log::info!("a message; some_data={} other_data={}", 123, "val1");
///   log::debug("some details");  // Not emitted
///   ```
/// - Set the default log level to `info` and set the level for `chatty::module1` to `warn`.
///   ```
///   let _global_logger_guard = logging::configure("info,chatty::module1=warn");
///   ```
/// - Use the environment variable to override default log level.
///   `module1` still gets its special log level.
///   ```
///   std::env::set_var("RUST_LOG", "debug");
///   let _global_logger_guard = logging::configure("info,module1=warn");
///   ```
/// - Use the environment variable to set `module1` to `debug`.
///   ```
///   std::env::set_var("RUST_LOG", "module1=debug");
///   let _global_logger_guard = logging::configure("info");
///   ```
///
/// Example output:
/// ```json
/// {"time_ns":1585851354242507000, "time":"2020-04-02T18:15:54.242521000Z", \
/// "module":"mod1","level":"ERROR","message":"msg1", "thread":"main","x":2}
/// ```
pub fn configure(filters: &str) -> Result<slog_scope::GlobalLoggerGuard, Box<dyn Error>> {
    let format = OutputFormat::JSON;
    #[cfg(debug_assertions)] // Include the following statement in debug binaries, not release.
    let dev_log_format: String = std::env::var("DEV_LOG_FORMAT").unwrap_or(String::new());
    let format = match dev_log_format.as_ref() {
        "" | "json" => format,
        "compact" => OutputFormat::Compact,
        "full" => OutputFormat::Full,
        "plain" => OutputFormat::Plain,
        s => panic!("Invalid DEV_LOG_FORMAT env var value {:?}", s),
    };
    configure_inner(filters, format)
}

/// Configures `log` and `slog` to emit to stdout with "plain" format.
/// Can be called multiple times from different threads.
/// Each call leaks a `GlobalLoggerGuard` which contains only a `bool`.
pub fn configure_for_test(filters: &str) -> Result<(), Box<dyn Error>> {
    let global_logger_guard = configure_inner(filters, OutputFormat::Plain)?;
    Box::leak(Box::new(global_logger_guard));
    Ok(())
}

fn configure_inner(
    filters: &str,
    output_format: OutputFormat,
) -> Result<slog_scope::GlobalLoggerGuard, Box<dyn Error>> {
    let _time_fn =
        |_: &slog::Record| chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let _time_ns_fn = |_: &slog::Record| {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            // Use default Duration (0 seconds) if system time is before epoch.
            .unwrap_or_default()
            // Nanoseconds overflow u64 in the year 2554.
            .as_nanos() as u64
    };
    let _module_fn = |record: &slog::Record| record.module();
    let _level_fn = |record: &slog::Record| match record.level() {
        slog::Level::Critical => "ERROR",
        slog::Level::Error => "ERROR",
        slog::Level::Warning => "WARN",
        slog::Level::Info => "INFO",
        slog::Level::Debug => "DEBUG",
        slog::Level::Trace => "TRACE",
    };
    let _message_fn = |record: &slog::Record| record.msg().to_string();
    let _time_fn = |_record: &slog::Record| {
        chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    };
    let _write_timestamp_fn = |w: &mut dyn std::io::Write| {
        w.write(
            chrono::Local::now()
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
                .as_bytes(),
        )
        .map(|_| ())
    };
    let drain: Box<dyn slog::Drain<Ok = (), Err = std::io::Error> + Send> = match output_format {
        OutputFormat::JSON => {
            Box::new(
                slog_json::Json::new(std::io::stdout())
                    .add_key_value(slog::o!(
                        // Fields are in reverse order.
                        "message" => slog::FnValue(_message_fn),
                        "level" => slog::FnValue(_level_fn),
                        "module" => slog::FnValue(_module_fn),
                        "time" => slog::FnValue(_time_fn),
                        "time_ns" => slog::FnValue(_time_ns_fn),
                        // TODONT(mleonhard) Don't include 'process' or 'host'.
                        // Supervisor and collector will add these values and
                        // will not trust any values already present.
                    ))
                    .build(),
            )
        }
        OutputFormat::Compact => Box::new(
            slog_term::CompactFormat::new(slog_term::TermDecorator::new().build())
                .use_custom_timestamp(_write_timestamp_fn)
                .build(),
        ),
        OutputFormat::Full => Box::new(
            slog_term::FullFormat::new(slog_term::TermDecorator::new().build())
                .use_custom_timestamp(_write_timestamp_fn)
                .build(),
        ),
        OutputFormat::Plain => Box::new(
            slog_term::FullFormat::new(slog_term::PlainDecorator::new(std::io::stdout()))
                .use_custom_timestamp(_write_timestamp_fn)
                .build(),
        ),
    };
    let drain = slog_envlogger::LogBuilder::new(drain)
        .parse(filters)
        // Add any level overrides from environment variable
        .parse(&match std::env::var("RUST_LOG") {
            Ok(x) => Ok(x),
            Err(std::env::VarError::NotPresent) => Ok(String::new()),
            Err(x) => Err(x),
        }?)
        .build();
    let drain = slog::Fuse(std::sync::Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init()?;
    log_panics::init();
    Ok(_guard)
}

pub fn thread_scope<SF, R>(name: &str, f: SF) -> R
where
    SF: FnOnce() -> R,
{
    let _name = name;
    let logger = slog_scope::logger().new(slog::o!("thread" => String::from(_name)));
    slog_scope::scope(&logger, f)
}

pub fn task_scope<F>(name: &'static str, f: F) -> slog_scope_futures::SlogScope<Logger, F>
where
    F: Future,
{
    let _name = name;
    slog_scope_futures::SlogScope::new(slog_scope::logger().new(slog::o!("task" => _name)), f)
}

#[macro_export]
macro_rules! error (
    ($($args:tt)+) => { slog::error!(slog_scope::logger(), $($args)+) };
);
#[macro_export]
macro_rules! warn (
    ($($args:tt)+) => { slog::warn!(slog_scope::logger(), $($args)+) };
);
#[macro_export]
macro_rules! info (
    ($($args:tt)+) => { slog::info!(slog_scope::logger(), $($args)+) };
);
#[macro_export]
macro_rules! debug (
    ($($args:tt)+) => { slog::debug!(slog_scope::logger(), $($args)+) };
);
#[macro_export]
macro_rules! trace (
    ($($args:tt)+) => { slog::trace!(slog_scope::logger(), $($args)+) };
);
