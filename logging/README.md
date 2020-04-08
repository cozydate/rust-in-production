# logging

Opinion: Do logging like [`opinion.rs`](src/bin/opinion.rs) which uses [`lib.rs`](src/lib.rs).

That code does a lot of things.  Here are the things split into separate binaries:
- [`default_log_levels.rs`](src/bin/default_log_levels.rs)
- [`log_levels_from_env_var.rs`](src/bin/log_levels_from_env_var.rs)
- [`default_json.rs`](src/bin/default_json.rs)
- [`dev_log_format.rs`](src/bin/dev_log_format.rs)
- [`log_scopes.rs`](src/bin/log_scopes.rs)
- [`log_panics.rs`](src/bin/log_panics.rs)
- [`custom_json.rs`](src/bin/custom_json.rs)

More info:
- [Crate slog](https://docs.rs/slog/)
- [Crate slog_envlogger](https://docs.rs/slog-envlogger/)
- [Crate log](https://docs.rs/log/)
- [Ask HN: How do you handle logging?](https://news.ycombinator.com/item?id=20818106)
