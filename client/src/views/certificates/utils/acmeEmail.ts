const RECENT_ACME_EMAILS_KEY = 'gate.acme.recentEmails'
const MAX_RECENT_ACME_EMAILS = 5

function normalizeEmail(email: string): string {
  return email.trim().toLowerCase()
}

export function loadRecentAcmeEmails(): string[] {
  if (typeof localStorage === 'undefined') return []
  try {
    const raw = localStorage.getItem(RECENT_ACME_EMAILS_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed
      .filter((value): value is string => typeof value === 'string')
      .map((value) => value.trim())
      .filter(Boolean)
      .slice(0, MAX_RECENT_ACME_EMAILS)
  } catch {
    return []
  }
}

export function rememberAcmeEmail(email: string) {
  const normalized = normalizeEmail(email)
  if (!normalized || typeof localStorage === 'undefined') return

  const recent = loadRecentAcmeEmails().filter((item) => normalizeEmail(item) !== normalized)
  localStorage.setItem(
    RECENT_ACME_EMAILS_KEY,
    JSON.stringify([email.trim(), ...recent].slice(0, MAX_RECENT_ACME_EMAILS)),
  )
}

export function mergeAcmeEmailOptions(savedEmail = '', recentEmails: string[] = []): string[] {
  const seen = new Set<string>()
  const options: string[] = []

  for (const candidate of [savedEmail, ...recentEmails]) {
    const trimmed = candidate.trim()
    const key = normalizeEmail(trimmed)
    if (!trimmed || seen.has(key)) continue
    seen.add(key)
    options.push(trimmed)
  }

  return options
}
