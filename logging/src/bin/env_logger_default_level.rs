fn main() {
    // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
    // https://github.com/sebasmagri/env_logger/releases/tag/v0.5.8
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));
    log::error!("main");
    log::warn!("main");
    log::info!("main");
    log::debug!("main");
    log::trace!("main");
    logging::using_log::error();
    logging::using_log::warn();
    logging::using_log::info();
    logging::using_log::debug();
    logging::using_log::trace();
    logging::using_log::info_in_thread();

    // $ cargo run --bin 0_default_level
    // [2020-04-01T19:38:24Z ERROR 0_default_level] main
    // [2020-04-01T19:38:24Z WARN  0_default_level] main
    // [2020-04-01T19:38:24Z INFO  0_default_level] main
    // [2020-04-01T19:38:24Z ERROR logging::using_log] using_log 1
    // [2020-04-01T19:38:24Z WARN  logging::using_log] using_log 1
    // [2020-04-01T19:38:24Z INFO  logging::using_log] using_log 1

    // You can change the default log level:
    // $ RUST_LOG="debug" cargo run --bin 0_default_level
    // [2020-04-01T19:38:50Z ERROR 0_default_level] main
    // [2020-04-01T19:38:50Z WARN  0_default_level] main
    // [2020-04-01T19:38:50Z INFO  0_default_level] main
    // [2020-04-01T19:38:50Z DEBUG 0_default_level] main
    // [2020-04-01T19:38:50Z ERROR logging::using_log] using_log 1
    // [2020-04-01T19:38:50Z WARN  logging::using_log] using_log 1
    // [2020-04-01T19:38:50Z INFO  logging::using_log] using_log 1
    // [2020-04-01T19:38:50Z DEBUG logging::using_log] using_log 1

    // You can set log level for specific modules:
    // $ RUST_LOG="info,logging::using_log=debug" cargo run --bin 0_default_level
    // [2020-04-01T19:39:32Z ERROR 0_default_level] main
    // [2020-04-01T19:39:32Z WARN  0_default_level] main
    // [2020-04-01T19:39:32Z INFO  0_default_level] main
    // [2020-04-01T19:39:32Z ERROR logging::using_log] using_log 1
    // [2020-04-01T19:39:32Z WARN  logging::using_log] using_log 1
    // [2020-04-01T19:39:32Z INFO  logging::using_log] using_log 1
    // [2020-04-01T19:39:32Z DEBUG logging::using_log] using_log 1

    // Be sure to include the default level.  If you omit it, only modules with explicitly set
    // levels will produce log messages.
    // $ RUST_LOG="logging::using_log=error" cargo run --bin 0_default_level
    // [2020-04-01T19:40:24Z ERROR logging::using_log] using_log 1
}
