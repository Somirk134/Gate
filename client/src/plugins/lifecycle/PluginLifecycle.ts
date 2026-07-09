import type { PluginAPI } from '../PluginAPI'

export type PluginLifecycleState =
  'registered' | 'activating' | 'active' | 'deactivating' | 'inactive' | 'failed'

export interface PluginLifecycle {
  activate(api: PluginAPI): void | Promise<void>
  deactivate?(api: PluginAPI): void | Promise<void>
}
