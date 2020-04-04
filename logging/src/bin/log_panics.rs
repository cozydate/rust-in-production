use std::sync::Mutex;

fn main() {
    let drain = slog_json::Json::default(std::io::stdout());
    let drain = slog::Fuse(Mutex::new(drain));
    let logger = slog::Logger::root(drain, slog::o!());
    let _guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();
    log_panics::init();

    panic!("uhoh");

    // $ cargo run --bin log_panics
    // {"msg":"thread 'main' panicked at 'uhoh': src/bin/log_panics.rs:11\n   0: backtrace::backtrace::trace_unsynchronized\n   1: backtrace::backtrace::trace\n   2: backtrace::capture::Backtrace::create\n   3: backtrace::capture::Backtrace::new\n   4: log_panics::init::{{closure}}\n   5: std::panicking::rust_panic_with_hook\n   6: std::panicking::begin_panic\n   7: log_panics::main\n   8: std::rt::lang_start::{{closure}}\n   9: std::panicking::try::do_call\n  10: __rust_maybe_catch_panic\n  11: std::rt::lang_start_internal\n  12: std::rt::lang_start\n  13: main\n","level":"ERRO","ts":"2020-04-03T19:57:12.365129-07:00"}
}
