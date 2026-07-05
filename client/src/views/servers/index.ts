/* ==================================================================
   Server 模块统一导出（barrel）
   ------------------------------------------------------------------
   对外暴露类型、Store、Composables、工具与 Mock 数据。
   组件层按需引入，避免深层路径。
   ================================================================== */

// ── Types ──
export * from "./types"

// ── Utils ──
export * from "./utils"

// ── Store ──
export { useServerStore } from "./store/server"

// ── Composables ──
export { useServer } from "./composables/useServer"
export { useServerSearch } from "./composables/useServerSearch"
export { useServerFilter } from "./composables/useServerFilter"
export { useServerSort } from "./composables/useServerSort"
export { useServerStatistics } from "./composables/useServerStatistics"
export { useServerHealth } from "./composables/useServerHealth"
export { useServerMonitor } from "./composables/useServerMonitor"

// ── Mock ──
export {
  mockServers,
  defaultServerForm,
  connectionMethods,
} from "./mock"
