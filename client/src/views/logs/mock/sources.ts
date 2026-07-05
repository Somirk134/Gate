import type { LogSource, LogSourceNode } from "../types"

export const LOG_SOURCES: LogSourceNode[] = [
  {
    id: "ALL",
    label: "All Logs",
    icon: "logs",
    children: [
      { id: "SYSTEM", label: "System", icon: "monitor" },
      { id: "CLIENT", label: "Client", icon: "terminal" },
      { id: "SERVER", label: "Server", icon: "servers" },
      { id: "PROJECT", label: "Project", icon: "projects" },
      { id: "TUNNEL", label: "Tunnel", icon: "router" },
      { id: "STATISTICS", label: "Statistics", icon: "chart-bar" },
      { id: "UPDATE", label: "Update", icon: "refresh" },
      { id: "PLUGIN", label: "Plugin", icon: "plug", reserved: true },
    ],
  },
]

export const LOG_SOURCE_LIST: LogSource[] = [
  "SYSTEM",
  "CLIENT",
  "SERVER",
  "PROJECT",
  "TUNNEL",
  "STATISTICS",
  "UPDATE",
  "PLUGIN",
]

export const LOG_SOURCE_LABELS: Record<LogSource, string> = {
  SYSTEM: "System",
  CLIENT: "Client",
  SERVER: "Server",
  PROJECT: "Project",
  TUNNEL: "Tunnel",
  STATISTICS: "Statistics",
  UPDATE: "Update",
  PLUGIN: "Plugin",
}
