# Monitoring

Monitoring Center 是 Gate 后续所有模块接入统计、健康、采样、告警、导出的统一入口。

## Current Phase

已完成：

- Rust Trait、Struct、Enum
- TypeScript Interface
- Promise Service API
- Mock 数据源
- Vue Dashboard Components
- Mermaid 架构图和数据流
- 测试目录骨架

未做：

- 数据库连接
- Prometheus 实现
- OpenTelemetry 实现
- 真实业务采集

## Runtime Behavior

`MonitoringCenter.collect_once()` 的逻辑：

1. `CollectorRegistry.collect_all()`
2. `StatisticsAggregator.ingest_metrics()`
3. `StatisticsSampler.sample()`
4. `HealthCenter.update()`
5. `AlertManager.evaluate_statistics()`
6. 发送 `StatisticsEvent`

## Logging

Rust 侧通过 `tracing` 记录：

- Collector Start
- Collector Stop
- Sampling
- Aggregation
- Export
- Alert
