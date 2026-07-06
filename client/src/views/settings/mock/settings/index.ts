import { aboutSettings } from "./about"
import { appearanceSettings } from "./appearance"
import { developerSettings } from "./developer"
import { experimentalSettings } from "./experimental"
import { generalSettings } from "./general"
import { httpsSettings } from "./https"
import { logsSettings } from "./logs"
import { networkSettings } from "./network"
import { notificationSettings } from "./notification"
import { projectSettings } from "./project"
import { securitySettings } from "./security"
import { serverSettings } from "./server"
import { shortcutSettings } from "./shortcut"
import { storageSettings } from "./storage"
import { tunnelSettings } from "./tunnel"
import { updateSettings } from "./update"
import { workspaceSettings } from "./workspace"

export const settingsCategories = [
  generalSettings,
  appearanceSettings,
  workspaceSettings,
  projectSettings,
  tunnelSettings,
  serverSettings,
  logsSettings,
  networkSettings,
  httpsSettings,
  notificationSettings,
  shortcutSettings,
  storageSettings,
  securitySettings,
  updateSettings,
  experimentalSettings,
  developerSettings,
  aboutSettings,
].sort((a, b) => a.order - b.order)

export {
  aboutSettings,
  appearanceSettings,
  developerSettings,
  experimentalSettings,
  generalSettings,
  httpsSettings,
  logsSettings,
  networkSettings,
  notificationSettings,
  projectSettings,
  securitySettings,
  serverSettings,
  shortcutSettings,
  storageSettings,
  tunnelSettings,
  updateSettings,
  workspaceSettings,
}
