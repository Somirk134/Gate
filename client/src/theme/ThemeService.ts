import type { ConfigurationService } from '@/services/ConfigurationService'
import type { EventBus } from '@/events/EventBus'
import type { AppEventMap } from '@/types/application'
import type { Disposable } from '@/utils/disposable'

export type ThemeMode = 'dark' | 'light' | 'auto'
export type EffectiveTheme = 'dark' | 'light'

export interface ThemeState {
  mode: ThemeMode
  effectiveTheme: EffectiveTheme
  systemPrefersDark: boolean
}

export interface ThemeService extends Disposable {
  getState(): ThemeState
  setTheme(mode: ThemeMode): void
  toggleTheme(): void
  start(): void
}

export class DefaultThemeService implements ThemeService {
  private state: ThemeState
  private mediaQuery?: MediaQueryList
  private readonly handleSystemThemeChange = (event: MediaQueryListEvent) => {
    this.state = {
      ...this.state,
      systemPrefersDark: event.matches,
      effectiveTheme: this.resolveEffectiveTheme(this.state.mode, event.matches),
    }
    this.applyTheme()
  }

  constructor(
    private readonly configuration: ConfigurationService,
    private readonly events: EventBus<AppEventMap>,
  ) {
    const mode = this.configuration.get<ThemeMode>('appearance.theme') ?? 'dark'
    const systemPrefersDark = this.readSystemPreference()

    this.state = {
      mode,
      systemPrefersDark,
      effectiveTheme: this.resolveEffectiveTheme(mode, systemPrefersDark),
    }
  }

  start() {
    this.mediaQuery =
      typeof window !== 'undefined' ? window.matchMedia('(prefers-color-scheme: dark)') : undefined

    this.mediaQuery?.addEventListener?.('change', this.handleSystemThemeChange)
    this.applyTheme()
  }

  getState(): ThemeState {
    return { ...this.state }
  }

  setTheme(mode: ThemeMode) {
    const systemPrefersDark = this.readSystemPreference()
    this.state = {
      mode,
      systemPrefersDark,
      effectiveTheme: this.resolveEffectiveTheme(mode, systemPrefersDark),
    }
    this.configuration.set('appearance.theme', mode)
    this.applyTheme()
  }

  toggleTheme() {
    const next = this.state.effectiveTheme === 'dark' ? 'light' : 'dark'
    this.setTheme(next)
  }

  dispose() {
    this.mediaQuery?.removeEventListener?.('change', this.handleSystemThemeChange)
  }

  private applyTheme() {
    if (typeof document !== 'undefined') {
      document.documentElement.classList.remove('theme-dark', 'theme-light')
      document.documentElement.classList.add(
        this.state.effectiveTheme === 'dark' ? 'theme-dark' : 'theme-light',
      )
      document.documentElement.dataset.theme = this.state.effectiveTheme
    }

    void this.events.publish('theme:changed', {
      mode: this.state.mode,
      effectiveTheme: this.state.effectiveTheme,
    })
  }

  private resolveEffectiveTheme(mode: ThemeMode, systemPrefersDark: boolean): EffectiveTheme {
    if (mode === 'auto') {
      return systemPrefersDark ? 'dark' : 'light'
    }

    return mode
  }

  private readSystemPreference() {
    if (typeof window === 'undefined') {
      return false
    }

    return window.matchMedia('(prefers-color-scheme: dark)').matches
  }
}
