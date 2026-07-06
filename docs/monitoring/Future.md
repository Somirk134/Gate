# Future

## Real Collectors

- Tunnel collector
- Server collector
- Runtime collector
- Heartbeat collector
- Authentication collector
- Project collector
- System collector
- Network collector
- Client collector

## Exporters

- JSON exporter
- CSV exporter
- Prometheus exporter
- OpenTelemetry exporter

Prometheus 和 OpenTelemetry 当前只保留接口，不实现协议、注册表、push/pull gateway 或 collector SDK。

## Storage

未来可以增加：

- in-memory ring buffer
- embedded local store
- remote statistics API
- retention policy
- downsampling

当前阶段不接数据库。

## Dashboard

未来 Dashboard 可以增加：

- zoomable chart
- custom widgets
- alert timeline
- module drill-down
- collector plugin status
- historical comparison
