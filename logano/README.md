# Logano opinionated structured logging library

Logano is a Rust library for structured logging.
It is opinionated.  This means it has good defaults that just work and just enough configurability.

By default, it emits JSON objects to stdout, one per line.
All JSON values are UTF-8 strings or 64-bit unsigned integers.

# Examples
```rust
use logano::info;
use std::fmt::format;
info(format!("Listening on TCP {}", addr));
// Emits:
// {"time_ns":1591847174000000000,"level":"INFO","message":"Listening on TCP *:1291"}\n
```

```rust
use logano::kv;
use std::fmt::format;
let _guard = logano::threadlocal_kv(
    kv("pii_ip",request.pii_addr),
    kv("useragent",request.useragent),
    );
let wallclock = WallClock::new();
let response = process(request);
let processing_ns = wallclock.elapsed_ns();
logano::event(
    wallclock.start_epoch_ns(),
    kv("http_status",response.status),
    kv("pii_clientid",response.pii_clientid),
    kv("pii_userid",response.pii_userid),
    kv("rpc_method",response.rpc_method),
    kv("processing_ns",processing_ns),
    kv("request_bytes",request.num_bytes),
    kv("response_bytes",response.num_bytes),
    );
// Emits:
// {"time_ns":1591847175000000000,
//  "pii_ip":"123.45.67.89",
//  "useragent":"myapp-ios-1472-prod/iOS-13/iPhone8",
//  "http_status":200,
//  "pii_clientid":"ZTQ1PT",
//  "pii_userid":"CX59F4",
//  "rpc_method":"GetProfile",
//  "processing_ns":29000000,
//  "request_bytes":2910,
//  "response_bytes":5719}\n
```

# Fields
- `time_ns`
  - The number of nano-seconds since the Epoch until the program prepared the logging event.
  - When recording a time interval, time_ns should indicate the start of the interval.
    For example, when an RPC, the time_ns should be the time that the request was successfully parsed and ready for processing.
  - Events may be emitted out of order.
  - Always present.
- `pii_*`
  - These fields contain [Personally Identifiable Information (PII)](https://en.wikipedia.org/wiki/Personal_data).
  - The storage system should purge these fields once the event reaches a specific age.
- `level`
  - The level of a log event.
  - The value can be `"ERROR"`, "`WARN`", `"INFO"`, `"DEBUG"`, or `"TRACE"`.
- `message`
  - The text of a log event.
  - Do not include PII in log messages.  Instead, include PII in separate fields, with names starting with `pii_`.
- Other fields are application-specific.

# Graphs and Alarms
Servers emit streams of metrics.
Alarms consume metrics.
Dashboards display metrics, alarm thresholds, and alarm events.

Traditional operations stacks use separate tools for emitting metrics,
collecting metrics, storing metrics, evaluating alarms, and displaying dashboards.
Each of these tools will have its own configuration file, written in a unique
configuration language.  This is a lot of complexity to maintain.

In Logano, we define metrics, alarms, and dashboards in Rust.
The definitions are part of the server that generates the metrics.
This has several advantages over the traditional approach:
- One language for code and config
- Config is written in a safe language that is checked at compile time.
  Errors are discovered at compile-time.
- Alarms and dashboards use metric objects.
  This prevents typos in metric names.
- Metrics, alarms, and dashboards are always tested and deployed together.
- Testing alarms is easy, since they can be integration tests.

When a server starts, Logano emits a special event containing the alarms and
dashboard configs.
The alarm and dashboard servers look for these events and use the latest configs.

A single service may be backed by many servers.
Service alarms and dashboards consume events from all of the servers.
Each server provides its own version of the service alarm & dashboard config.
The alarm & dashboard server uses the config version provided by the most number of servers.
This means that a rolling update of a service alarm takes effect when half of
its servers have been updated.
Dashboard tools may allow canarying updates by using service config from a single server.

```rust
use logano::kv;
use std::fmt::format;
let builder = logano::builder();
let http_status = builder.add_number_field("http_status");
builder.add_server_alarm(
    http_status,
    "400s_high",
    logano::gt(
        logano::div(
            logano::count(logano::in_range_inclusive(400,499)),
            logano::count(logano::any_value())),
        0.05),
    logano::sec(500),
    logano::ticket(),
);
builder.add_server_alarm(
    http_status,
    "503s_high",
    logano::gt(
        logano::div(
            logano::count(logano::is(503)),
            logano::count(logano::any_value())),
        0.05),
    logano::sec(500),
    logano::ticket(),
);
builder.add_server_alarm(
    http_status,
    "500s_sans503_high",
    logano::gt(
        logano::div(
            logano::count(
                logano::or(
                    logano::in_range_inclusive(500,502),
                    logano::in_range_inclusive(504,599)),
            logano::count(logano::any_value())),
        0.05),
);
builder.add_server_alarm(
    http_status,
    "few_requests",
    logano::lt(
        logano::count(logano::any_value()),
        300 + 10, // 1 rps health checks plus two requests per min
    logano::sliding_window_sec(300)
);
let dbconn_waiters = builder.add_number_field("dbconn_waiters");
let db_txn_errors = builder.add_number_field("db_txn_errors");
let pii_clientid = builder.add_pii_string_field("pii_clientid");
let pii_userid = builder.add_pii_string_field("pii_userid");
builder.build().init();

let _guard = logano::threadlocal_kv(
    kv("pii_ip",request.pii_addr),
    kv("useragent",request.useragent),
    );
let wallclock = WallClock::new();
let response = process(request);
let processing_ns = wallclock.elapsed_ns();
logano::event(
    wallclock.start_epoch_ns(),
    kv("http_status",response.status),
    kv("pii_clientid",response.pii_clientid /* is a logano::PiiString */),
    kv("pii_userid",response.pii_userid /* is a logano::PiiString */)),
    kv("rpc_method",response.rpc_method),
    kv("processing_ns",processing_ns),
    kv("request_bytes",request.num_bytes),
    kv("response_bytes",response.num_bytes),
    );
// Emits:
// {"time_ns":1591847175000000000,
//  "pii_ip":"123.45.67.89",
//  "useragent":"myapp-ios-1472-prod/iOS-13/iPhone8",
//  "http_status":200,
//  "pii_clientid":"ZTQ1PT",
//  "pii_userid":"CX59F4",
//  "rpc_method":"GetProfile",
//  "processing_ns":29000000,
//  "request_bytes":2910,
//  "response_bytes":5719}\n
```
