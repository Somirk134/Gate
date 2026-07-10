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

const REDACTED = '[REDACTED]'
const sensitiveKeyPattern = /(token|password|secret|private[_-]?key|key[_-]?pem)/i

export function redactText(value: string): string {
  return value
    .replace(/-----BEGIN [^-]*PRIVATE KEY-----[\s\S]*?-----END [^-]*PRIVATE KEY-----/gi, REDACTED)
    .replace(/\bBearer\s+[^\s,;]+/gi, `Bearer ${REDACTED}`)
    .replace(
      /\b(token|password|secret|private[_-]?key)\s*([:=])\s*("[^"]*"|'[^']*'|[^\s,;&]+)/gi,
      (_match, key: string, separator: string) => `${key}${separator}${REDACTED}`,
    )
}

function redactLogValue(value: unknown, seen = new WeakSet<object>()): unknown {
  if (typeof value === 'string') return redactText(value)
  if (value === null || typeof value !== 'object') return value
  if (seen.has(value)) return '[CIRCULAR]'
  seen.add(value)

  if (value instanceof Error) {
    return {
      name: value.name,
      message: redactText(value.message),
      stack: value.stack ? redactText(value.stack) : undefined,
    }
  }
  if (Array.isArray(value)) return value.map((item) => redactLogValue(item, seen))

  const result: Record<string, unknown> = {}
  for (const [key, item] of Object.entries(value)) {
    result[key] = sensitiveKeyPattern.test(key) ? REDACTED : redactLogValue(item, seen)
  }
  return result
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
    // 所有 sink 共用同一脱敏入口，避免错误上下文把凭据写入控制台或未来的文件日志。
    const entry: LogEntry = {
      level,
      message: redactText(message),
      data: redactLogValue(data),
      scope: this.scope,
      timestamp: Date.now(),
    }

    for (const sink of this.sinks) {
      void sink.write(entry)
    }
  }
}
