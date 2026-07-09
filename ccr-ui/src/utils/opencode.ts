export function formatJsonInput(value: unknown, fallback = '{}') {
  if (value == null) return fallback
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return fallback
  }
}

export function parseJsonInput<T>(value: string, fallback: T): T {
  const trimmed = value.trim()
  if (!trimmed) return fallback
  return JSON.parse(trimmed) as T
}

export function splitCommandInput(value: string): string[] {
  return (
    value
      .match(/"[^"]*"|'[^']*'|\S+/g)
      ?.map((part) => part.replace(/^['"]|['"]$/g, '').trim())
      .filter(Boolean) ?? []
  )
}

export function stringifyCommandInput(parts?: string[]) {
  return (parts ?? []).join(' ')
}

export function maskSecret(value?: string) {
  if (!value) return 'not configured'
  if (value.startsWith('{env:')) return value
  if (value.length <= 8) return `${value.slice(0, 2)}••••`
  return `${value.slice(0, 4)}••••${value.slice(-4)}`
}

export function normalizeStringListInput(value: string) {
  return value
    .split('\n')
    .map((item) => item.trim())
    .filter(Boolean)
}
