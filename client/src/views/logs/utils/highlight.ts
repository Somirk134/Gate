function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;")
}

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
}

export function highlightText(text: string, keyword: string): string {
  const safe = escapeHtml(text)
  const query = keyword.trim()
  if (!query) return safe
  return safe.replace(new RegExp(`(${escapeRegex(query)})`, "gi"), "<mark>$1</mark>")
}
