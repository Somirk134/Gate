export type FontSizePreference = 'compact' | 'comfortable' | 'large' | 'extra-large'

const STORAGE_KEY = 'gate.appearance.fontSize'

const fontSizeTokens: Record<FontSizePreference, Record<string, string>> = {
  compact: {
    '--text-xs': '11px',
    '--text-sm': '12px',
    '--text-base': '13px',
    '--text-md': '14px',
    '--text-lg': '15px',
    '--text-xl': '20px',
    '--text-2xl': '24px',
    '--text-3xl': '30px',
  },
  comfortable: {
    '--text-xs': '12px',
    '--text-sm': '13px',
    '--text-base': '14px',
    '--text-md': '15px',
    '--text-lg': '16px',
    '--text-xl': '21px',
    '--text-2xl': '25px',
    '--text-3xl': '31px',
  },
  large: {
    '--text-xs': '13px',
    '--text-sm': '14px',
    '--text-base': '15px',
    '--text-md': '16px',
    '--text-lg': '17px',
    '--text-xl': '22px',
    '--text-2xl': '26px',
    '--text-3xl': '32px',
  },
  'extra-large': {
    '--text-xs': '14px',
    '--text-sm': '15px',
    '--text-base': '16px',
    '--text-md': '17px',
    '--text-lg': '18px',
    '--text-xl': '23px',
    '--text-2xl': '28px',
    '--text-3xl': '34px',
  },
}

export function normalizeFontSizePreference(value: unknown): FontSizePreference {
  if (
    value === 'compact' ||
    value === 'comfortable' ||
    value === 'large' ||
    value === 'extra-large'
  )
    return value
  return 'comfortable'
}

export function getFontSizePreference(): FontSizePreference {
  if (typeof localStorage === 'undefined') return 'comfortable'
  return normalizeFontSizePreference(localStorage.getItem(STORAGE_KEY))
}

export function applyFontSizePreference(value: unknown) {
  const preference = normalizeFontSizePreference(value)
  const root = document.documentElement
  root.dataset.fontSize = preference

  for (const [token, tokenValue] of Object.entries(fontSizeTokens[preference])) {
    root.style.setProperty(token, tokenValue)
  }

  root.style.setProperty('--font-size-title', 'var(--text-xl)')
  root.style.setProperty('--font-size-subtitle', 'var(--text-md)')
  root.style.setProperty('--font-size-body', 'var(--text-base)')
  root.style.setProperty('--font-size-caption', 'var(--text-xs)')
  root.style.setProperty('--font-size-code', 'var(--text-sm)')
  root.style.setProperty('--font-size-button', 'var(--text-base)')
  root.style.setProperty('--font-size-input', 'var(--text-base)')

  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, preference)
  }

  return preference
}

export function initAppearancePreferences() {
  applyFontSizePreference(getFontSizePreference())
}
