let fallbackSequence = 0

export function createId(prefix: string): string {
  // 优先使用平台 UUID；旧 WebView 不支持时，用时间戳和进程内序号保证本次运行不重复。
  const value =
    globalThis.crypto?.randomUUID?.() ??
    `${Date.now().toString(36)}-${(++fallbackSequence).toString(36)}`
  return `${prefix}-${value}`
}
