import type { ConfigurationService } from "@/services/ConfigurationService"

export interface WindowBounds {
  width: number
  height: number
  x?: number
  y?: number
}

export interface WindowState {
  title: string
  focused: boolean
  maximized: boolean
  minimized: boolean
  bounds?: WindowBounds
}

export interface WindowService {
  getState(): WindowState
  setTitle(title: string): void
  minimize(): Promise<void>
  maximize(): Promise<void>
  unmaximize(): Promise<void>
  close(): Promise<void>
  startDragging(): Promise<void>
  focus(): Promise<void>
  setBounds(bounds: WindowBounds): Promise<void>
}

export class BrowserWindowService implements WindowService {
  private state: WindowState

  constructor(private readonly configuration: ConfigurationService) {
    this.state = {
      title: this.configuration.get<string>("window.title") ?? "Gate",
      focused: typeof document !== "undefined" ? document.hasFocus() : true,
      maximized: false,
      minimized: false,
      bounds: this.configuration.get<WindowBounds>("window.bounds"),
    }
  }

  getState(): WindowState {
    return { ...this.state }
  }

  setTitle(title: string) {
    this.state = { ...this.state, title }

    if (typeof document !== "undefined") {
      document.title = title
    }

    this.configuration.set("window.title", title)
  }

  async minimize() {
    this.state = { ...this.state, minimized: true }
  }

  async maximize() {
    this.state = { ...this.state, maximized: true, minimized: false }
  }

  async unmaximize() {
    this.state = { ...this.state, maximized: false }
  }

  async close() {
    this.state = { ...this.state, focused: false }
  }

  async startDragging() {
    return undefined
  }

  async focus() {
    this.state = { ...this.state, focused: true }

    if (typeof window !== "undefined") {
      window.focus()
    }
  }

  async setBounds(bounds: WindowBounds) {
    this.state = { ...this.state, bounds }
    this.configuration.set("window.bounds", bounds)
  }
}
