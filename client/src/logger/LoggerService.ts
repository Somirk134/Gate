export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error'

export interface LogEntry {
  level: LogLevel
  message: string
  scope?: string
  data?: unknown
  timestamp: number
}

export interface LogSink {
  write(entry: LogEntry): void | Promise<void>
}

export interface LoggerService {
  trace(message: string, data?: unknown): void
  debug(message: string, data?: unknown): void
  info(message: string, data?: unknown): void
  warn(message: string, data?: unknown): void
  error(message: string, data?: unknown): void
  child(scope: string): LoggerService
}

export class ConsoleLogSink implements LogSink {
  write(entry: LogEntry) {
    const prefix = `[${entry.level.toUpperCase()}]${entry.scope ? ` [${entry.scope}]` : ''}`

    if (entry.level === 'error') {
      console.error(prefix, entry.message, entry.data ?? '')
      return
    }

    if (entry.level === 'warn') {
      console.warn(prefix, entry.message, entry.data ?? '')
      return
    }

    if (entry.level === 'debug' || entry.level === 'trace') {
      return
    }

    console.info(prefix, entry.message, entry.data ?? '')
  }
}

export class DefaultLoggerService implements LoggerService {
  constructor(
    private readonly sinks: LogSink[] = [new ConsoleLogSink()],
    private readonly scope?: string,
  ) {}

  trace(message: string, data?: unknown) {
    this.write('trace', message, data)
  }

  debug(message: string, data?: unknown) {
    this.write('debug', message, data)
  }

  info(message: string, data?: unknown) {
    this.write('info', message, data)
  }

  warn(message: string, data?: unknown) {
    this.write('warn', message, data)
  }

  error(message: string, data?: unknown) {
    this.write('error', message, data)
  }

  child(scope: string): LoggerService {
    const nextScope = this.scope ? `${this.scope}:${scope}` : scope
    return new DefaultLoggerService(this.sinks, nextScope)
  }

  private write(level: LogLevel, message: string, data?: unknown) {
    const entry: LogEntry = {
      level,
      message,
      data,
      scope: this.scope,
      timestamp: Date.now(),
    }

    for (const sink of this.sinks) {
      void sink.write(entry)
    }
  }
}
