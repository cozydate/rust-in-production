# rust-in-production
Example code for using Rust programs in production.

TODO(mleonhard) Add example of logging events, like HTTP requests or RPCs.
TODO(mleonhard) Figure out a good way to configure metric alarms and display.

# Streams

Every process in the deployment emits a stream of "observability events" in JSON format on stdout.
Each event has a timestamp and at least one key-value pair.
Values are strings or non-negative integers.
Examples:
- `{"time_ns":1585851354242507000,"level":"INFO","module":"account","message":"Updated email","pii_uid":"U577019","pii_old_email":"user1@abc.com","pii_new_email":"user1@okok.com"}`
- `{"time_ns":1585851354242732000,"rpc_method":"UpdateAccount","user_err":"","server_error":"","duration_ms":12,"conn_ms":7,"txn_ms":3,"pii_ip":"42.102.177.61","pii_uid":"U577019"}`
- `{"time_ns":1585851354242732000,"rpc_method":"UpdateAccount","user_err":"REJECT_EMAIL_ADDRESS","server_error":"","duration_ms":1,"conn_ms":0,"txn_ms":0,"pii_ip":"42.102.177.61","pii_uid":"U577019"}`
- `{"time_ns":1585851354242732000,"rpc_method":"UpdateAccount","user_err":"","server_error":"DB_CONN_UNAVAILABLE","duration_ms":1000,"conn_ms":1000,"txn_ms":0,"pii_ip":"42.102.177.61","pii_uid":"U577019"}`
- `{"time_ns":1585851354242732000,"http_verb":"GET","url":"https://mycorp.com/blobs/bc103b4a84971","status":200,"pii_ip":"42.102.177.61","pii_uid":"U577019"}`
- `{"time_ns":1585851354242789000,"action":"opened_app","app_version":"ios/762","device_type":"iphone8","pii_uid":"U577019"}`
- `{"time_ns":1585851354242789000,"worker":"prod2","work_item":"W223431","type":"IMAGE_INGEST","result":"ok","duration_ms":153,"network_ms":38}`
- `{"time_ns":1585851354242789000,"queue_length":5,"failed_item_count":0}`
- `{"time_ns":1585851354242789000,"heap_size_mb":1024,"heap_used_p":38,"threads":594,"sockets":578,"files":163}`

Each process has a supervisor that collects the events.
The supervisor also collects its own events.
It saves events to disk.
It discards events that are older than the maximum age (7 days).
It discards the oldest events to ensure that total file size is under the maximum number of bytes
(500 MB).

The supervisor groups events logically into an append-only stream.
Events appear in the stream in arbitrary order.

The stream is divided into chunks of less than 1 MB or 10 minutes.
Each chunk has a string id, matching regex `"[123456789CDFGHJKLMNPQRTVWXZ]{1,16}"`.
Example: `"C5FXMD"`.

Supervisor runs an HTTP server providing access to the chunks:
- GET `/chunk/` returns a whitespace-delimited list of chunk ids, in chronological order.  The latest chunk appears last.
- GET `/chunk/C5FXMD` returns the chunk with id `C5FXMD`.
  - Returns 404 if the chunk has been discarded.
  - If this is the latest chunk, immediately returns all data already in the chunk, then streams remaining
data as it becomes available using HTTP chunked transfer-encoding.
  - Client should use socket and read timeouts larger than the max chunk time (10 min).
  - Client should enable SO_KEEPALIVE on the TCP socket to detect server failure.
- Client must provide a TLS client certificate that is on the supervisor's list of allowed certs.

# Event Store

The event store fetches old events from the supervisors and streams new events in real-time.

Event Store keeps event records on disk and provides a query API for retrieving records.

It de-duplicates all records.

It prioritizes recent data over old data.
When a data source comes back online after a multi-hour outage, event store fetches and streasm the
latest data.
It uses a separate thread pool to fetch old data.

xxh3 looks like a good hashing algorithm.

https://crates.io/crates/anyhow

"Dremel: Interactive Analysis of Web-Scale Datasets"
http://static.googleusercontent.com/media/research.google.com/en/us/pubs/archive/36632.pdf

"Capacitor storage engine (successor to ColumnIO)"
https://news.ycombinator.com/item?id=12321515