use std::sync::Mutex;

use slog::{debug, info};

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
    let drain = slog_envlogger::LogBuilder::new(drain)
        // mleonhard found no way to programmatically retrieve a module name.
        // type_name didn't work:
        //   print!("{}", std::any::type_name::<logging::apple>());
        //                                      ^^^^^^^^^^^^^^ not a type
        // https://doc.rust-lang.org/std/any/fn.type_name.html
        // So we must use error-prone strings.
        .parse("info,logging::apple=debug")
        .build();
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);

    info!(slog_scope::logger(), "main");
    debug!(slog_scope::logger(), "main");
    logging::apple::debug();

    // $ cargo run --bin default_log_levels
    // {"msg":"main","level":"INFO","ts":"2020-04-04T23:47:11.147752-07:00"}
    // {"msg":"apple 1","level":"DEBG","ts":"2020-04-04T23:47:11.148345-07:00","x":2}
}
