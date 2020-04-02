use slog::{debug, error, info, trace, warn};

pub fn error() { error!(slog_scope::logger(), "banana {}", 1; "x" => 2); }

pub fn warn() { warn!(slog_scope::logger(), "banana {}", 1; "x" => 2); }

pub fn info() { info!(slog_scope::logger(), "banana {}", 1; "x" => 2); }

pub fn debug() { debug!(slog_scope::logger(), "banana {}", 1; "x" => 2); }

pub fn trace() { trace!(slog_scope::logger(), "banana {}", 1; "x" => 2); }

pub fn info_in_thread() {
    std::thread::spawn(|| {
        slog_scope::scope(
            &slog_scope::logger().new(slog::o!("thread" => "banana")),
            || {
                info!(slog_scope::logger(), "banana in thread {}", 1; "x" => 2);
            });
    }).join().unwrap();
}
