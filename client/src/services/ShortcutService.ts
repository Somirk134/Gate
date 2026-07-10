import type { CommandRegistry } from '@/commands/CommandRegistry'
import type { EventBus } from '@/events/EventBus'
import { GateAppError } from '@/ipc'
import type { LoggerService } from '@/logger/LoggerService'
import type { AppEventMap } from '@/types/application'
import type { Disposable } from '@/utils/disposable'

export interface ShortcutBinding {
  id: string
  commandId: string
  shortcut: string
  when?: () => boolean
}

export interface ShortcutService extends Disposable {
  register(binding: ShortcutBinding): Disposable
  unregister(id: string): void
  list(): ShortcutBinding[]
  start(): void
  stop(): void
}

export class BrowserShortcutService implements ShortcutService {
  private readonly bindings = new Map<string, ShortcutBinding>()
  private started = false

  private readonly handleKeydown = (event: KeyboardEvent) => {
    const binding = this.findBinding(event)

    if (!binding) {
      return
    }

    if (binding.when && !binding.when()) {
      return
    }

    event.preventDefault()
    void this.events.publish('shortcut:triggered', {
      id: binding.id,
      commandId: binding.commandId,
      shortcut: binding.shortcut,
    })

    void this.commands
      .execute(binding.commandId, {
        source: 'shortcut',
      })
      .catch((error) => {
        this.logger.error(`Shortcut command failed: ${binding.commandId}`, error)
      })
  }

  constructor(
    private readonly commands: CommandRegistry,
    private readonly events: EventBus<AppEventMap>,
    private readonly logger: LoggerService,
  ) {}

  register(binding: ShortcutBinding): Disposable {
    if (this.bindings.has(binding.id)) {
      throw new GateAppError({
        code: 'SHORTCUT_ALREADY_REGISTERED',
        messageKey: 'errors.application.shortcutAlreadyRegistered',
        details: { id: binding.id },
        timestamp: Date.now(),
      })
    }

    this.bindings.set(binding.id, binding)

    return {
      dispose: () => this.unregister(binding.id),
    }
  }

  unregister(id: string) {
    this.bindings.delete(id)
  }

  list() {
    return Array.from(this.bindings.values())
  }

  start() {
    if (this.started || typeof document === 'undefined') {
      return
    }

    document.addEventListener('keydown', this.handleKeydown)
    this.started = true
  }

  stop() {
    if (!this.started || typeof document === 'undefined') {
      return
    }

    document.removeEventListener('keydown', this.handleKeydown)
    this.started = false
  }

  dispose() {
    this.stop()
    this.bindings.clear()
  }

  private findBinding(event: KeyboardEvent) {
    if (this.isTextInput(event.target) && !event.metaKey && !event.ctrlKey) {
      return undefined
    }

    return this.list().find((binding) => this.matches(binding.shortcut, event))
  }

  private matches(shortcut: string, event: KeyboardEvent) {
    const parts = shortcut
      .toLowerCase()
      .split('+')
      .map((part) => part.trim())
    const key = parts[parts.length - 1]
    const wantsCtrl = parts.includes('ctrl')
    const wantsMeta = parts.includes('meta') || parts.includes('cmd')
    const wantsShift = parts.includes('shift')
    const wantsAlt = parts.includes('alt')
    const modPressed = event.ctrlKey || event.metaKey

    if (wantsCtrl && !modPressed) {
      return false
    }

    if (wantsMeta && !event.metaKey) {
      return false
    }

    if (wantsShift !== event.shiftKey) {
      return false
    }

    if (wantsAlt !== event.altKey) {
      return false
    }

    return event.key.toLowerCase() === key
  }

  private isTextInput(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) {
      return false
    }

    const tag = target.tagName.toLowerCase()
    return tag === 'input' || tag === 'textarea' || target.isContentEditable
  }
}
