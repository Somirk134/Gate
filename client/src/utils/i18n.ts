import { nextTick, type Ref } from 'vue'
import type { Composer } from 'vue-i18n'
import type { Path, PathValue } from '@intlify/core-base'

/**
 * Escape bare `@` in vue-i18n message templates.
 * Linked messages (`@:key`, `@.modifier:key`) are left untouched.
 */
export function escapeUnlinkedAtSign(message: string): string {
  if (message.includes("{'@'}")) {
    return message
  }
  return message.replace(/@(?![:.])/g, "{'@'}")
}

export function resolveNestedMessage(obj: unknown, path: string): unknown {
  if (!obj || typeof obj !== 'object') {
    return null
  }

  let current: unknown = obj
  for (const segment of path.split('.')) {
    if (!current || typeof current !== 'object' || !(segment in current)) {
      return null
    }
    current = (current as Record<string, unknown>)[segment]
  }

  return current ?? null
}

export function createAtSafeMessageResolver(): (obj: unknown, path: Path) => PathValue {
  return (obj, path) => {
    const resolved = resolveNestedMessage(obj, String(path))
    if (typeof resolved === 'string') {
      return escapeUnlinkedAtSign(resolved)
    }
    return resolved as PathValue
  }
}

export async function reopenOverlay(visible: Ref<boolean>) {
  if (visible.value) {
    visible.value = false
    await nextTick()
  }
  visible.value = true
}

export function translateIfExists(
  t: Composer['t'],
  te: Composer['te'],
  key: string,
  fallback: string,
): string {
  if (!te(key)) {
    return fallback
  }

  try {
    return String(t(key))
  } catch {
    return fallback
  }
}
