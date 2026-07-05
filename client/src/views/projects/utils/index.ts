/* ==================================================================
   Project 模块工具函数
   ------------------------------------------------------------------
   颜色预设、图标预设、标签预设、状态配置、格式化函数。
   统一从此处引入，禁止在组件中硬编码。
   ================================================================== */

import type {
  ColorPreset,
  IconPreset,
  ProjectColor,
  ProjectStatus,
  ProjectTag,
  TagPreset,
} from "../types"

/* ── 颜色预设 ── */
export const PROJECT_COLORS: ColorPreset[] = [
  { key: "blue", label: "蓝色", value: "#5B8DEF" },
  { key: "green", label: "绿色", value: "#22C55E" },
  { key: "purple", label: "紫色", value: "#7C6FF2" },
  { key: "orange", label: "橙色", value: "#F59E0B" },
  { key: "red", label: "红色", value: "#EF4444" },
  { key: "cyan", label: "青色", value: "#06B6D4" },
  { key: "pink", label: "粉色", value: "#EC4899" },
  { key: "indigo", label: "靛蓝", value: "#6366F1" },
  { key: "teal", label: "蓝绿", value: "#14B8A6" },
  { key: "amber", label: "琥珀", value: "#D97706" },
  { key: "slate", label: "石板灰", value: "#64748B" },
]

/* 颜色键 → hex 映射，便于 O(1) 查找 */
export const COLOR_MAP: Record<ProjectColor, string> = PROJECT_COLORS.reduce(
  (acc, c) => {
    acc[c.key] = c.value
    return acc
  },
  {} as Record<ProjectColor, string>,
)

/* ── 图标预设（均为 registry 中已注册的 Lucide 图标） ── */
export const PROJECT_ICONS: IconPreset[] = [
  { key: "package", label: "Package" },
  { key: "globe", label: "Globe" },
  { key: "database", label: "Database" },
  { key: "servers", label: "Server" },
  { key: "cloud", label: "Cloud" },
  { key: "code", label: "Code" },
  { key: "box", label: "Box" },
  { key: "terminal", label: "Terminal" },
  { key: "activity", label: "Activity" },
  { key: "cpu", label: "CPU" },
  { key: "router", label: "Router" },
  { key: "layers", label: "Layers" },
  { key: "boxes", label: "Boxes" },
  { key: "network", label: "Network" },
  { key: "shield", label: "Shield" },
  { key: "zap", label: "Zap" },
  { key: "rocket", label: "Rocket" },
  { key: "hard-drive", label: "Storage" },
  { key: "link", label: "Link" },
  { key: "plug", label: "Plug" },
]

/* ── 预置标签 ── */
export const PROJECT_TAGS: TagPreset[] = [
  { name: "Work", color: "#5B8DEF" },
  { name: "Home", color: "#22C55E" },
  { name: "Demo", color: "#F59E0B" },
  { name: "Client", color: "#EC4899" },
  { name: "Open Source", color: "#14B8A6" },
  { name: "Personal", color: "#7C6FF2" },
  { name: "Production", color: "#EF4444" },
  { name: "Staging", color: "#06B6D4" },
]

/* ── 状态配置：label + GBadge variant ── */
export const STATUS_CONFIG: Record<
  ProjectStatus,
  { label: string; variant: "success" | "warning" | "error" | "neutral" | "info"; pulse: boolean }
> = {
  running: { label: "运行中", variant: "success", pulse: false },
  partial: { label: "部分运行", variant: "warning", pulse: false },
  stopped: { label: "已停止", variant: "neutral", pulse: false },
  starting: { label: "启动中", variant: "info", pulse: true },
  error: { label: "异常", variant: "error", pulse: false },
}

/* ── hex → rgba ── */
function hexToRgba(hex: string, alpha: number): string {
  const h = hex.replace("#", "")
  const r = parseInt(h.substring(0, 2), 16)
  const g = parseInt(h.substring(2, 4), 16)
  const b = parseInt(h.substring(4, 6), 16)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

/**
 * 生成项目颜色 CSS 变量对象，用于在元素上注入：
 *   --project-color        实色
 *   --project-color-muted  15% 透明度底色
 *   --project-color-soft   8% 透明度底色
 */
export function projectColorVars(color: ProjectColor): Record<string, string> {
  const hex = COLOR_MAP[color] ?? COLOR_MAP.blue
  return {
    "--project-color": hex,
    "--project-color-muted": hexToRgba(hex, 0.15),
    "--project-color-soft": hexToRgba(hex, 0.08),
  }
}

/* ── 格式化：字节 ── */
export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B"
  const units = ["B", "KB", "MB", "GB", "TB"]
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  const v = bytes / Math.pow(1024, i)
  return `${v.toFixed(v < 10 && i > 0 ? 1 : 0)} ${units[i]}`
}

/* ── 格式化：时长（秒） ── */
export function formatDuration(seconds: number): string {
  if (seconds <= 0) return "0s"
  const d = Math.floor(seconds / 86400)
  const h = Math.floor((seconds % 86400) / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  if (d > 0) return `${d}d ${h}h`
  if (h > 0) return `${h}h ${m}m`
  if (m > 0) return `${m}m ${s}s`
  return `${s}s`
}

/* ── 生成唯一 ID ── */
export function genId(prefix = "p"): string {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`
}

/* ── 将字符串数组转为 Tag 对象 ── */
export function toTagObjects(tags: string[]): ProjectTag[] {
  return tags.map((name, i) => ({ id: `tag-${i}-${name}`, name }))
}
