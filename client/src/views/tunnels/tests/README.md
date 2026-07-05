# Tunnel 模块测试（预留）

本目录用于存放 Tunnel 模块的单元测试与集成测试。

## 计划测试范围

- **types**：类型契约不变性测试
- **utils**：格式化函数、端口校验、状态配置映射
- **store**：状态机流转（idle → loading → success/error）、CRUD、生命周期动作、tick 抖动
- **composables**：
  - `useTunnelSearch`：模糊搜索覆盖范围
  - `useTunnelFilter`：各筛选分支与计数
  - `useTunnelSort`：固定优先 + 各排序键 + 方向
  - `useTunnelStatistics`：聚合计算正确性
  - `useTunnelMonitor`：启停与定时驱动
- **components**：关键组件渲染与交互（TunnelCard 选中、TunnelLogs 过滤/暂停、TunnelDialog 校验）

## 运行（待接入）

```bash
pnpm test src/views/tunnels
```

> 当前阶段全部数据来自 Mock，未来替换为真实 Tunnel Engine 后，测试应改为对适配层的契约验证。
