// Opinionated JSON logging.  You should do logging like this.
use logging::{debug, error, info, trace, warn};

fn main() {
    let _global_logger_guard = logging::configure("info").unwrap();
    logging::thread_scope("main", || {
        error!("message1"; "x" => 2);
        warn!("message1"; "x" => 2);
        info!("message1"; "x" => 2);
        debug!("message1"; "x" => 2);
        trace!("message1"; "x" => 2);

        logging::using_log::info();
        logging::using_log::info_in_thread();

        logging::using_slog::info();
        logging::using_slog::info_in_thread();

        logging::using_slog_scope::info();
        logging::using_slog_scope::info_in_thread();

        panic!("uhoh");
    });

    // $ cargo run --bin opinion
    // {"time_ns":1599775310760838000,"time":"2020-09-10T15:01:50.760-07:00","module":"opinion","level":"ERROR","message":"message1","thread":"main","x":2}
    // {"time_ns":1599775310761887000,"time":"2020-09-10T15:01:50.761-07:00","module":"opinion","level":"WARN","message":"message1","thread":"main","x":2}
    // {"time_ns":1599775310761923000,"time":"2020-09-10T15:01:50.761-07:00","module":"opinion","level":"INFO","message":"message1","thread":"main","x":2}
    // {"time_ns":1599775310761955000,"time":"2020-09-10T15:01:50.761-07:00","module":"logging::using_log","level":"INFO","message":"using_log 1","thread":"main"}
    // {"time_ns":1599775310762005000,"time":"2020-09-10T15:01:50.762-07:00","module":"logging::using_log","level":"INFO","message":"using_log in thread 1"}
    // {"time_ns":1599775310762048000,"time":"2020-09-10T15:01:50.762-07:00","module":"logging::using_slog","level":"INFO","message":"using_slog 1","thread":"main","x":2}
    // {"time_ns":1599775310762077000,"time":"2020-09-10T15:01:50.762-07:00","module":"logging::using_slog","level":"INFO","message":"using_slog in thread 1","thread":"using_slog","x":2}
    // {"time_ns":1599775310762109000,"time":"2020-09-10T15:01:50.762-07:00","module":"logging::using_slog_scope","level":"INFO","message":"using_slog_scope 1","thread":"main","x":2}
    // {"time_ns":1599775310762161000,"time":"2020-09-10T15:01:50.762-07:00","module":"logging::using_slog_scope","level":"INFO","message":"using_slog_scope in thread 1","thread":"using_slog_scope","x":2}
    // {"time_ns":1599775311390883000,"time":"2020-09-10T15:01:51.390-07:00","module":"log_panics","level":"ERROR","message":"thread 'main' panicked at 'uhoh': src/bin/opinion.rs:22\n   0: backtrace::backtrace::libunwind::trace\n             at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/backtrace/libunwind.rs:95\n      backtrace::backtrace::trace_unsynchronized\n             at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/backtrace/mod.rs:66\n   1: backtrace::backtrace::trace\n             at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/backtrace/mod.rs:53\n   2: backtrace::capture::Backtrace::create\n             at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/capture.rs:164\n   3: backtrace::capture::Backtrace::new\n             at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/capture.rs:128\n   4: log_panics::init::{{closure}}\n             at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/log-panics-2.0.0/src/lib.rs:52\n   5: std::panicking::rust_panic_with_hook\n             at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/panicking.rs:573\n   6: std::panicking::begin_panic::{{closure}}\n             at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/panicking.rs:498\n   7: std::sys_common::backtrace::__rust_end_short_backtrace\n             at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/sys_common/backtrace.rs:153\n   8: std::panicking::begin_panic\n             at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/panicking.rs:497\n   9: opinion::main::{{closure}}\n             at src/bin/opinion.rs:22\n  10: slog_scope::scope\n             at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/slog-scope-4.3.0/lib.rs:248\n  11: logging::thread_scope\n             at src/lib.rs:181\n  12: opinion::main\n             at src/bin/opinion.rs:6\n  13: core::ops::function::FnOnce::call_once\n             at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/core/src/ops/function.rs:227\n  14: std::sys_common::backtrace::__rust_begin_short_backtrace\n             at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/sys_common/backtrace.rs:137\n  15: std::rt::lang_start::{{closure}}\n             at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/rt.rs:66\n  16: core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once\n             at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/core/src/ops/function.rs:259\n      std::panicking::try::do_call\n             at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/panicking.rs:373\n      std::panicking::try\n             at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/panicking.rs:337\n      std::panic::catch_unwind\n             at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/panic.rs:379\n      std::rt::lang_start_internal\n             at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/rt.rs:51\n  17: std::rt::lang_start\n             at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/rt.rs:65\n  18: _main\n","thread":"main"}

    // $ DEV_LOG_FORMAT=plain cargo run --bin opinion
    // 2020-09-10T15:02:31.344-07:00 ERRO message1, x: 2, thread: main
    // 2020-09-10T15:02:31.345-07:00 WARN message1, x: 2, thread: main
    // 2020-09-10T15:02:31.346-07:00 INFO message1, x: 2, thread: main
    // 2020-09-10T15:02:31.346-07:00 INFO using_log 1, thread: main
    // 2020-09-10T15:02:31.346-07:00 INFO using_log in thread 1
    // 2020-09-10T15:02:31.346-07:00 INFO using_slog 1, x: 2, thread: main
    // 2020-09-10T15:02:31.346-07:00 INFO using_slog in thread 1, x: 2, thread: using_slog
    // 2020-09-10T15:02:31.346-07:00 INFO using_slog_scope 1, x: 2, thread: main
    // 2020-09-10T15:02:31.346-07:00 INFO using_slog_scope in thread 1, x: 2, thread: using_slog_scope
    // 2020-09-10T15:02:31.981-07:00 ERRO thread 'main' panicked at 'uhoh': src/bin/opinion.rs:22
    //    0: backtrace::backtrace::libunwind::trace
    //              at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/backtrace/libunwind.rs:95
    //       backtrace::backtrace::trace_unsynchronized
    //              at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/backtrace/mod.rs:66
    //    1: backtrace::backtrace::trace
    //              at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/backtrace/mod.rs:53
    //    2: backtrace::capture::Backtrace::create
    //              at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/capture.rs:164
    //    3: backtrace::capture::Backtrace::new
    //              at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/backtrace-0.3.50/src/capture.rs:128
    //    4: log_panics::init::{{closure}}
    //              at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/log-panics-2.0.0/src/lib.rs:52
    //    5: std::panicking::rust_panic_with_hook
    //              at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/panicking.rs:573
    //    6: std::panicking::begin_panic::{{closure}}
    //              at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/panicking.rs:498
    //    7: std::sys_common::backtrace::__rust_end_short_backtrace
    //              at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/sys_common/backtrace.rs:153
    //    8: std::panicking::begin_panic
    //              at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/panicking.rs:497
    //    9: opinion::main::{{closure}}
    //              at src/bin/opinion.rs:22
    //   10: slog_scope::scope
    //              at /Users/user/.cargo/registry/src/github.com-1ecc6299db9ec823/slog-scope-4.3.0/lib.rs:248
    //   11: logging::thread_scope
    //              at src/lib.rs:181
    //   12: opinion::main
    //              at src/bin/opinion.rs:6
    //   13: core::ops::function::FnOnce::call_once
    //              at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/core/src/ops/function.rs:227
    //   14: std::sys_common::backtrace::__rust_begin_short_backtrace
    //              at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/sys_common/backtrace.rs:137
    //   15: std::rt::lang_start::{{closure}}
    //              at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/rt.rs:66
    //   16: core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once
    //              at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/core/src/ops/function.rs:259
    //       std::panicking::try::do_call
    //              at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/panicking.rs:373
    //       std::panicking::try
    //              at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/panicking.rs:337
    //       std::panic::catch_unwind
    //              at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/panic.rs:379
    //       std::rt::lang_start_internal
    //              at /rustc/84b047bf64dfcfa12867781e9c23dfa4f2e6082c/library/std/src/rt.rs:51
    //   17: std::rt::lang_start
    //              at /Users/user/.rustup/toolchains/beta-x86_64-apple-darwin/lib/rustlib/src/rust/library/std/src/rt.rs:65
    //   18: _main
    // , thread: main
}
