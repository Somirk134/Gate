import type { Router } from "vue-router"
import type { AppContext } from "@/core/AppContext"
import type { Command } from "./types"

export function registerBuiltinCommands(context: AppContext, router: Router) {
  const commands: Command[] = [
    {
      id: "app.commandPalette.open",
      title: "Open Command Palette",
      category: "application",
      icon: "search",
      keywords: ["command", "palette", "search"],
      handler: async ({ context: app }) => {
        await app.events.publish("command-palette:open", undefined)
      },
    },
    {
      id: "app.commandPalette.toggle",
      title: "Toggle Command Palette",
      category: "application",
      icon: "search",
      shortcut: "Ctrl+K",
      keywords: ["command", "palette", "search"],
      handler: async ({ context: app }) => {
        await app.events.publish("command-palette:toggle", undefined)
      },
    },
    {
      id: "settings.open",
      title: "Open Settings",
      category: "settings",
      icon: "settings",
      shortcut: "Ctrl+,",
      keywords: ["settings", "preferences", "configuration"],
      handler: async () => {
        await router.push("/settings")
      },
    },
    {
      id: "app.sidebar.toggle",
      title: "Toggle Sidebar",
      category: "application",
      icon: "sidebar",
      shortcut: "Ctrl+\\",
      keywords: ["sidebar", "navigation"],
      handler: async ({ context: app }) => {
        await app.events.publish("sidebar:toggle", undefined)
      },
    },
    {
      id: "app.inspector.toggle",
      title: "Toggle Inspector",
      category: "application",
      icon: "inspector",
      shortcut: "Ctrl+Shift+I",
      keywords: ["inspector", "panel"],
      handler: async ({ context: app }) => {
        await app.events.publish("inspector:toggle", undefined)
      },
    },
    {
      id: "app.globalSearch.toggle",
      title: "Toggle Global Search",
      category: "application",
      icon: "search",
      shortcut: "Ctrl+Shift+K",
      keywords: ["global", "search"],
      handler: async ({ context: app }) => {
        await app.events.publish("global-search:toggle", undefined)
      },
    },
    {
      id: "project.quickOpen",
      title: "Quick Open Project",
      category: "project",
      icon: "projects",
      shortcut: "Ctrl+P",
      keywords: ["project", "open", "quick"],
      handler: async ({ context: app }) => {
        await app.events.publish("command:reserved", {
          id: "project.quickOpen",
        })
      },
    },
    reserved("project.create", "Create Project", "project", "projects", "Ctrl+N"),
    reserved("project.delete", "Delete Project", "project", "trash"),
    reserved("tunnel.start", "Start Tunnel", "tunnel", "play"),
    reserved("tunnel.stop", "Stop Tunnel", "tunnel", "stop"),
    reserved("server.connect", "Connect Server", "server", "servers"),
  ]

  for (const command of commands) {
    if (!context.commands.has(command.id)) {
      context.commands.register(command)
    }
  }
}

function reserved(
  id: string,
  title: string,
  category: string,
  icon: string,
  shortcut?: string,
): Command {
  return {
    id,
    title,
    category,
    icon,
    shortcut,
    handler: async ({ context, args }) => {
      await context.events.publish("command:reserved", { id, args })
    },
  }
}
