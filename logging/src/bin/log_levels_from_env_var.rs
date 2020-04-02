use std::sync::Mutex;

use slog::{debug, info};

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
    let filters = std::env::var("RUST_LOG").unwrap_or(String::from("info"));
    let drain = slog_envlogger::LogBuilder::new(drain).parse(&filters).build();
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);

    info!(slog_scope::logger(), "main");
    debug!(slog_scope::logger(), "main");
    logging::using_slog::debug();

    // $ cargo run --bin log_levels_from_env_var
    // {"msg":"main","level":"INFO","ts":"2020-04-02T00:05:36.083135-07:00"}

    // You can change the default log level:
    // $ RUST_LOG=debug cargo run --bin log_levels_from_env_var
    // {"msg":"main","level":"INFO","ts":"2020-04-02T00:05:54.017148-07:00"}
    // {"msg":"main","level":"DEBG","ts":"2020-04-02T00:05:54.017735-07:00"}
    // {"msg":"using_slog 1","level":"DEBG","ts":"2020-04-02T00:05:54.017766-07:00","x":2}

    // You can set log level for specific modules.
    // See https://docs.rs/slog-envlogger/2.2.0/slog_envlogger/
    // $ RUST_LOG="info,logging::using_slog=debug" cargo run --bin log_levels_from_env_var
    // {"msg":"main","level":"INFO","ts":"2020-04-02T00:06:10.670963-07:00"}
    // {"msg":"using_slog 1","level":"DEBG","ts":"2020-04-02T00:06:10.671500-07:00","x":2}

    // Be sure to include the default level.  If you omit it, only modules with explicitly set
    // levels will produce log messages.
    // $ RUST_LOG="logging::using_slog=debug" cargo run --bin log_levels_from_env_var
    // {"msg":"using_slog 1","level":"DEBG","ts":"2020-04-02T00:07:43.752226-07:00","x":2}
}
