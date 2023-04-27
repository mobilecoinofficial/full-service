# Logging

This page is meant to be an overview of:

* How do our rust servers create logs
* What different technologies do we use to capture and view those logs

### Types of Logs

There are two types of logs right now:

* [Text logs (syslog style)](./text-logs.md). With a timestamp, a log level, and text description of an event.
* [Crash reports](./crash-reports.md). These are supposed to include text logs leading up to the crash, and a stack trace.
