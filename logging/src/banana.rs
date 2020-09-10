use slog::{debug, info};

pub fn info() {
    info!(slog_scope::logger(), "banana {}", 1; "x" => 2);
}

pub fn debug() {
    debug!(slog_scope::logger(), "banana {}", 1; "x" => 2);
}
