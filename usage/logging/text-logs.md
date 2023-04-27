# Text Logs

In full-service and fog, the `mc-common` crate contains "logger" implementations based on the rust slog crate.

The `mc-sgx-slog` crate provides an `slog::Logger` appropriate for the enclave. It is recommended to refer
to `mc_common::logger::Logger` in the portable `enclave-impl` crates, and put the calls to `mc-sgx-slog` in
the `enclave-trusted` crates.

Most servers use the `mc_common::logger::create_app_logger` function at their entrypoint to create a logger for the
server. This logger uses an in-memory buffer and a background thread to eventually write the log messages to where they
are supposed to go.

The `create_root_logger` function sets up several possible log destinations:

* STDOUT (or STDERR if an environment variable is set)
* UDP-JSON logging (if enabled)
* Sentry logging (forwarding error and critical messages directly to Sentry servers)

The background thread is enabling the main thread to make progress asynchronously while these network calls are in
progress.

(Note that this is very different from the "12 factor app" guidance: https://12factor.net/logs.)

