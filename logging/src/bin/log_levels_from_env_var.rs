use std::sync::Mutex;

use slog::{debug, info};

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
    let drain = slog_envlogger::LogBuilder::new(drain)
        // Default levels
        .parse("debug,logging::apple=debug,logging::banana=info")
        // Add any level overrides from environment variable
        .parse(&std::env::var("RUST_LOG").unwrap_or(String::new()))
        .build();
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);

    info!(slog_scope::logger(), "main");
    debug!(slog_scope::logger(), "main");
    logging::apple::debug();
    logging::banana::info();
    logging::banana::debug();

    // $ cargo run --bin log_levels_from_env_var
    // {"msg":"main","level":"INFO","ts":"2020-04-02T09:36:10.116799-07:00"}
    // {"msg":"main","level":"DEBG","ts":"2020-04-02T09:36:10.117368-07:00"}
    // {"msg":"apple 1","level":"DEBG","ts":"2020-04-02T09:36:10.117394-07:00","x":2}
    // {"msg":"banana 1","level":"INFO","ts":"2020-04-02T09:36:10.117426-07:00","x":2}

    // You can change the default log level:
    // $ RUST_LOG=debug cargo run --bin log_levels_from_env_var
    // {"msg":"main","level":"INFO","ts":"2020-04-02T09:36:25.783190-07:00"}
    // {"msg":"main","level":"DEBG","ts":"2020-04-02T09:36:25.783800-07:00"}   <-- Note
    // {"msg":"apple 1","level":"DEBG","ts":"2020-04-02T09:36:25.783830-07:00","x":2}
    // {"msg":"banana 1","level":"INFO","ts":"2020-04-02T09:36:25.783866-07:00","x":2}

    // You can set log level for specific modules.
    // See https://docs.rs/slog-envlogger/2.2.0/slog_envlogger/
    // $ RUST_LOG="logging::banana=debug" cargo run --bin log_levels_from_env_var
    // {"msg":"main","level":"INFO","ts":"2020-04-02T09:36:59.037244-07:00"}
    // {"msg":"main","level":"DEBG","ts":"2020-04-02T09:36:59.037870-07:00"}
    // {"msg":"apple 1","level":"DEBG","ts":"2020-04-02T09:36:59.037901-07:00","x":2}
    // {"msg":"banana 1","level":"INFO","ts":"2020-04-02T09:36:59.037953-07:00","x":2}
    // {"msg":"banana 1","level":"DEBG","ts":"2020-04-02T09:36:59.037981-07:00","x":2} <-- Note
}
