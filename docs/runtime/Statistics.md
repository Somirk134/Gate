# Statistics

Runtime statistics are collected through `TrafficStatistics` and exposed by
`RuntimeMonitor`.

## TrafficStatistics

Counters:

- total upload
- total download
- current upload
- current download
- packet count
- session count
- error count
- reconnect count

## RuntimeMonitor

Metrics:

- active session
- active connection
- upload
- download
- current speed
- peak speed
- average speed
- runtime
- error count

## Accounting Direction

- bytes read from client are upload.
- bytes read from target are download.
- stream read and write errors increment error counters.
- connector retries increment reconnect counters.

## Snapshot Model

`RuntimeMonitor::snapshot()` produces a point-in-time `RuntimeMetrics` value.
It computes current speed from the byte delta since the previous snapshot and
average speed from total bytes over runtime uptime.
