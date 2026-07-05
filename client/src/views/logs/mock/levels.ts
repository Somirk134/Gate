import type { LogLevel, LogLevelOption } from "../types"

export const LOG_LEVELS: LogLevelOption[] = [
  { level: "TRACE", label: "Trace", color: "#8A9099", muted: "rgba(138, 144, 153, 0.12)", icon: "circle-dot" },
  { level: "DEBUG", label: "Debug", color: "#06B6D4", muted: "rgba(6, 182, 212, 0.12)", icon: "code" },
  { level: "INFO", label: "Info", color: "#3B82F6", muted: "rgba(59, 130, 246, 0.12)", icon: "info-circle" },
  { level: "WARN", label: "Warn", color: "#F59E0B", muted: "rgba(245, 158, 11, 0.14)", icon: "alert-triangle" },
  { level: "ERROR", label: "Error", color: "#EF4444", muted: "rgba(239, 68, 68, 0.13)", icon: "alert-circle" },
  { level: "FATAL", label: "Fatal", color: "#A855F7", muted: "rgba(168, 85, 247, 0.14)", icon: "zap" },
]

export const LOG_LEVEL_ORDER: Record<LogLevel, number> = {
  TRACE: 0,
  DEBUG: 1,
  INFO: 2,
  WARN: 3,
  ERROR: 4,
  FATAL: 5,
}

export function getLevelOption(level: LogLevel): LogLevelOption {
  return LOG_LEVELS.find((item) => item.level === level) ?? LOG_LEVELS[2]
}
