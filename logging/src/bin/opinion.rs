// Opinionated JSON logging.  You should do logging like this.
use logging::{debug, error, info, trace, warn};

fn main() {
    let _global_logger_guard = logging::configure("opinion", "info").unwrap();
    logging::thread_scope("main", || {
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
    // {"host":"mbp","process":"opinion","time_ns":1586068677170857000,"time":"2020-04-05T06:37:57.170862000Z","module":"log_panics","level":"ERROR","message":"thread 'main' panicked at 'uhoh': src/bin/opinion.rs:19\n   0: backtrace::backtrace::trace_unsynchronized\n   1: backtrace::backtrace::trace\n   2: backtrace::capture::Backtrace::create\n   3: backtrace::capture::Backtrace::new\n   4: log_panics::init::{{closure}}\n   5: std::panicking::rust_panic_with_hook\n   6: std::panicking::begin_panic\n   7: opinion::main::{{closure}}\n   8: slog_scope::scope\n   9: logging::thread_scope\n  10: opinion::main\n  11: std::rt::lang_start::{{closure}}\n  12: std::panicking::try::do_call\n  13: __rust_maybe_catch_panic\n  14: std::rt::lang_start_internal\n  15: std::rt::lang_start\n  16: main\n","thread":"main"}

    // $ DEV_LOG_FORMAT=plain cargo run --bin opinion
    // 2020-04-05T00:37:48.675-07:00 ERRO main, x: 2, thread: main
    // 2020-04-05T00:37:48.676-07:00 WARN main, x: 2, thread: main
    // 2020-04-05T00:37:48.676-07:00 INFO main, x: 2, thread: main
    // 2020-04-05T00:37:48.676-07:00 INFO using_log 1, thread: main
    // 2020-04-05T00:37:48.676-07:00 INFO using_log in thread 1
    // 2020-04-05T00:37:48.676-07:00 INFO apple 1, x: 2, thread: main
    // 2020-04-05T00:37:48.676-07:00 INFO apple in thread 1, x: 2, thread: apple
    // 2020-04-05T00:37:48.701-07:00 ERRO thread 'main' panicked at 'uhoh': src/bin/opinion.rs:19
    //    0: backtrace::backtrace::trace_unsynchronized
    //    1: backtrace::backtrace::trace
    //    2: backtrace::capture::Backtrace::create
    //    3: backtrace::capture::Backtrace::new
    //    4: log_panics::init::{{closure}}
    //    5: std::panicking::rust_panic_with_hook
    //    6: std::panicking::begin_panic
    //    7: opinion::main::{{closure}}
    //    8: slog_scope::scope
    //    9: logging::thread_scope
    //   10: opinion::main
    //   11: std::rt::lang_start::{{closure}}
    //   12: std::panicking::try::do_call
    //   13: __rust_maybe_catch_panic
    //   14: std::rt::lang_start_internal
    //   15: std::rt::lang_start
    //   16: main
    // , thread: main
}
