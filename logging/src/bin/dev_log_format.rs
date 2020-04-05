use std::sync::Mutex;

use slog::info;

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
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
        .parse("info")
        .parse(&std::env::var("RUST_LOG").unwrap_or(String::new()))
        .build();
    let drain = slog::Fuse(Mutex::new(drain));
    let drain = slog::Fuse(slog_async::Async::default(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);

    info!(slog_scope::logger(), "a message");
    slog_scope::scope(
        &slog_scope::logger().new(slog::o!("scope_var" => 123)),
        || {
            info!(slog_scope::logger(), "message 1 inside scope");
            info!(slog_scope::logger(), "message 2 inside scope");
        });
    info!(slog_scope::logger(), "a message with some data"; "x" => 123, "y" => "abc");
    info!(slog_scope::logger(), "line 1\n  line 2"; "x" => 456);

    // JSON is the default format:
    // $ cargo run --bin dev_log_format
    // {"msg":"a message","level":"INFO","ts":"2020-04-04T23:29:47.787546-07:00"}
    // {"msg":"message 1 inside scope","level":"INFO","ts":"2020-04-04T23:29:47.788234-07:00","scope_var":123}
    // {"msg":"message 2 inside scope","level":"INFO","ts":"2020-04-04T23:29:47.788287-07:00","scope_var":123}
    // {"msg":"a message with some data","level":"INFO","ts":"2020-04-04T23:29:47.788329-07:00","y":"abc","x":123}
    // {"msg":"line 1\n  line 2","level":"INFO","ts":"2020-04-04T23:29:47.788376-07:00","x":456}

    // You can specify 'json':
    // $ DEV_LOG_FORMAT=json cargo run --bin dev_log_format
    // {"msg":"a message","level":"INFO","ts":"2020-04-04T23:29:54.733970-07:00"}
    // {"msg":"message 1 inside scope","level":"INFO","ts":"2020-04-04T23:29:54.734620-07:00","scope_var":123}
    // {"msg":"message 2 inside scope","level":"INFO","ts":"2020-04-04T23:29:54.734655-07:00","scope_var":123}
    // {"msg":"a message with some data","level":"INFO","ts":"2020-04-04T23:29:54.734682-07:00","y":"abc","x":123}
    // {"msg":"line 1\n  line 2","level":"INFO","ts":"2020-04-04T23:29:54.734711-07:00","x":456}

    // You can specify 'compact' to print scope variables on their own line and indent log
    // messages emitted inside the scope:
    // $ DEV_LOG_FORMAT=compact cargo run --bin dev_log_format
    // 2020-04-05T00:32:11.140-07:00 INFO a message
    // scope_var: 123
    //  2020-04-05T00:32:11.140-07:00 INFO message 1 inside scope
    //  2020-04-05T00:32:11.140-07:00 INFO message 2 inside scope
    // 2020-04-05T00:32:11.140-07:00 INFO a message with some data, y: abc, x: 123
    // 2020-04-05T00:32:11.141-07:00 INFO line 1
    //   line 2, x: 456

    // You can specify 'full' to print scope variables on every line, with color.
    // $ DEV_LOG_FORMAT=full cargo run --bin dev_log_format
    // 2020-04-05T00:32:26.725-07:00 INFO a message
    // 2020-04-05T00:32:26.726-07:00 INFO message 1 inside scope, scope_var: 123
    // 2020-04-05T00:32:26.726-07:00 INFO message 2 inside scope, scope_var: 123
    // 2020-04-05T00:32:26.726-07:00 INFO a message with some data, y: abc, x: 123
    // 2020-04-05T00:32:26.726-07:00 INFO line 1
    //   line 2, x: 456

    // You can specify 'plain' to print without color:
    // $ DEV_LOG_FORMAT=plain cargo run --bin dev_log_format
    // 2020-04-05T00:32:44.798-07:00 INFO a message
    // 2020-04-05T00:32:44.798-07:00 INFO message 1 inside scope, scope_var: 123
    // 2020-04-05T00:32:44.798-07:00 INFO message 2 inside scope, scope_var: 123
    // 2020-04-05T00:32:44.798-07:00 INFO a message with some data, y: abc, x: 123
    // 2020-04-05T00:32:44.798-07:00 INFO line 1
    //   line 2, x: 456

    // Release binaries ignore the environment variable and always log JSON.
    // $ DEV_LOG_FORMAT=plain cargo run --bin dev_log_format --release
    // {"msg":"a message","level":"INFO","ts":"2020-04-04T23:30:20.708932-07:00"}
    // {"msg":"message 1 inside scope","level":"INFO","ts":"2020-04-04T23:30:20.709440-07:00","scope_var":123}
    // {"msg":"message 2 inside scope","level":"INFO","ts":"2020-04-04T23:30:20.709448-07:00","scope_var":123}
    // {"msg":"a message with some data","level":"INFO","ts":"2020-04-04T23:30:20.709454-07:00","y":"abc","x":123}
    // {"msg":"line 1\n  line 2","level":"INFO","ts":"2020-04-04T23:30:20.709462-07:00","x":456}
}
