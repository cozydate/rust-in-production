// use logano::{set_global_formatter, set_global_writer, error, warn, info, debug, trace, Ldp};

fn main() {
    logano::set_global_timezone(chrono_tz::America::Los_Angeles);
    // let _global_logger_guard = logging::configure("info").unwrap();
    // logging::thread_scope("main", || {
    //     error!("main"; "x" => 2);
    //     warn!("main"; "x" => 2);
    //     info!("main"; "x" => 2);
    //     debug!("main"; "x" => 2);
    //     trace!("main"; "x" => 2);
    //
    //     logging::using_log::info();
    //     logging::using_log::info_in_thread();
    //
    //     logging::using_slog::info();
    //     logging::using_slog::info_in_thread();
    //
    //     panic!("uhoh");
    // });

    // $ cargo run --bin opinion
    // {"time_ns":1586384161913089000,"time":"2020-04-08T22:16:01.913107000Z","module":"opinion","level":"ERROR","message":"main","thread":"main","x":2}
    // {"time_ns":1586384161913227000,"time":"2020-04-08T22:16:01.913230000Z","module":"opinion","level":"WARN","message":"main","thread":"main","x":2}
    // {"time_ns":1586384161913270000,"time":"2020-04-08T22:16:01.913272000Z","module":"opinion","level":"INFO","message":"main","thread":"main","x":2}
    // {"time_ns":1586384161913303000,"time":"2020-04-08T22:16:01.913305000Z","module":"logging::using_log","level":"INFO","message":"using_log 1","thread":"main"}
    // {"time_ns":1586384161913332000,"time":"2020-04-08T22:16:01.913334000Z","module":"logging::using_log","level":"INFO","message":"using_log in thread 1"}
    // {"time_ns":1586384161913359000,"time":"2020-04-08T22:16:01.913361000Z","module":"logging::apple","level":"INFO","message":"apple 1","thread":"main","x":2}
    // {"time_ns":1586384161913388000,"time":"2020-04-08T22:16:01.913390000Z","module":"logging::apple","level":"INFO","message":"apple in thread 1","thread":"apple","x":2}
    // {"time_ns":1586384161939148000,"time":"2020-04-08T22:16:01.939154000Z","module":"log_panics","level":"ERROR","message":"thread 'main' panicked at 'uhoh': src/bin/opinion.rs:19\n   0: backtrace::backtrace::trace_unsynchronized\n   1: backtrace::backtrace::trace\n   2: backtrace::capture::Backtrace::create\n   3: backtrace::capture::Backtrace::new\n   4: log_panics::init::{{closure}}\n   5: std::panicking::rust_panic_with_hook\n   6: std::panicking::begin_panic\n   7: opinion::main::{{closure}}\n   8: slog_scope::scope\n   9: logging::thread_scope\n  10: opinion::main\n  11: std::rt::lang_start::{{closure}}\n  12: std::panicking::try::do_call\n  13: __rust_maybe_catch_panic\n  14: std::rt::lang_start_internal\n  15: std::rt::lang_start\n  16: main\n","thread":"main"}

    // $ DEV_LOG_FORMAT=plain cargo run --bin opinion
    // 2020-04-08T15:16:31.367-07:00 ERRO main, x: 2, thread: main
    // 2020-04-08T15:16:31.368-07:00 WARN main, x: 2, thread: main
    // 2020-04-08T15:16:31.368-07:00 INFO main, x: 2, thread: main
    // 2020-04-08T15:16:31.368-07:00 INFO using_log 1, thread: main
    // 2020-04-08T15:16:31.368-07:00 INFO using_log in thread 1
    // 2020-04-08T15:16:31.368-07:00 INFO apple 1, x: 2, thread: main
    // 2020-04-08T15:16:31.368-07:00 INFO apple in thread 1, x: 2, thread: apple
    // 2020-04-08T15:16:31.393-07:00 ERRO thread 'main' panicked at 'uhoh': src/bin/opinion.rs:19
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
