# Dashboard

Dashboard 读取统一 `DashboardData`，当前由 Mock 自动刷新。

## Provided Views

- Overview Card
- Traffic Card
- Realtime Chart
- Connection Chart
- Health Panel
- Tunnel List
- Runtime Status
- Recent Activity

## Vue Components

- `StatisticsCard`
- `TrafficChart`
- `RealtimeChart`
- `HealthIndicator`
- `RuntimePanel`
- `TunnelStatistics`
- `ConnectionStatistics`
- `SystemStatistics`
- `OverviewPanel`
- `DashboardWidget`
- `MonitoringDashboard`

## Dashboard Data Flow

```mermaid
flowchart LR
  MockStatistics --> DashboardService["DashboardService.getDashboard"]
  MockTraffic --> DashboardService
  MockHealth --> DashboardService
  DashboardService --> Composable["useMonitoringDashboard"]
  Composable --> OverviewPanel
  Composable --> RealtimeChart
  Composable --> TrafficChart
  Composable --> HealthPanel["Health Panel"]
  Composable --> RuntimePanel
  Composable --> RecentActivity["Recent Activity"]
```

## Refresh Strategy

`MockDashboard` supports subscription and refreshes once per second. Future real APIs can implement the same Promise service interfaces:

- `StatisticsService`
- `MetricsService`
- `HealthService`
- `DashboardService`
- `ExportService`
