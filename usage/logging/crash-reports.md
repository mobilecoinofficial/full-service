# Crash reports

In full-service and fog, we are using [Sentry](https://sentry.io/welcome/) to track crash reports.

There are several ways to use sentry. One is that we could use something like google breakpad to get stacktraces from
the process without instrumenting it. That is not what we do right now -- we instrument the servers to send "Error"
and "Critical" logs to sentry.

We also have a special rust panic handler that formats the stack trace and sends it to the logs as a critical-level
message, so that sentry will get it.

This is currently the only way that sentry gets stack traces. This is a problem because the rust panic handler only
handles rust-language panics, but there are lots of ways the process can die without triggering a rust panic. For
example, if the OS kills the process with OOM, or if the process dies with SIGSEGV in some C code that we have linked
in. In these cases, sentry won't get a stacktrace, and we wouldn't get any stacktrace in production. An alternative
would be to get the stacktrace using google breakpad, which runs outside the code being monitored and captures a
stacktrace directly when it crashes, (and captures its recent logs on STDERR) and ship that to sentry.

Another known problem with this is that because of the way we use a background thread, the most recent logs may not
actually get out of the process before it dies. We try to work around this by installing a rust panic handler that
sleeps for 5 seconds, so that the logs probably escape the process before it dies. This is a hack, and it won't work if
there is a SIGSEGV for instance. A more robust solution would be to use process isolation and not a background thread,
so that log messages get out of the current process' memory (and into a pipe or some other place that will still exist
after SIGSEGV) as quickly as possible. The external logging process would be responsible to actually ship them over the
network to the final destination.

This goes back to 12-factor app guidance, that the app should be writing its logs unbuffered to STDOUT and all routing
and handling of the logs happens in a different process (which survives a crash).

