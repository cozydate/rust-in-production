use slog::debug;

pub fn debug() {
    debug!(slog_scope::logger(), "apple {}", 1; "x" => 2);
}
