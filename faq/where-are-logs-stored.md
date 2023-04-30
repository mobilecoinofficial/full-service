# Where are logs stored?

Log files for full-service are located in /tmp/ using the format:

`full-service-2023-04-26-15:47:37.log`\

_Note_: logs may contain private information regarding transactions times, wallet addresses, etc.

In Full-Service, the `mc-common` crate contains "logger" implementations based on the
rust [slog](https://github.com/slog-rs/slog) crate. An `slog::logger` is a type-erased logger interface, and it is
enclave-friendly as well.

Most servers use the `mc_common::logger::create_app_logger` function at their entrypoint to create a logger for the
server. This logger uses an in-memory buffer and a background thread to eventually write the log messages to where they
are supposed to go.

The `create_root_logger` function sets up several possible log destinations:

* STDOUT (or STDERR if an environment variable is set)
* UDP-JSON logging (if enabled)
* Sentry logging (forwarding error and critical messages directly to Sentry servers)

The background thread is enabling the main thread to make progress asynchronously while these network calls are in
progress.

\
\








