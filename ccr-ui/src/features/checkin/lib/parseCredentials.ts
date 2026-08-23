export type CookieMap = Record<string, string>

export const parseCookieString = (str: string): CookieMap => {
  const cookies: CookieMap = {}
  for (const part of str.split(';')) {
    const eqIdx = part.indexOf('=')
    if (eqIdx > 0) {
      cookies[part.substring(0, eqIdx).trim()] = part.substring(eqIdx + 1).trim()
    }
  }
  return cookies
}

const asCookieRecord = (json: unknown): CookieMap | null => {
  if (!json || typeof json !== 'object' || Array.isArray(json)) return null
  const record = json as Record<string, unknown>
  if (typeof record.cookies === 'string') return parseCookieString(record.cookies)
  const cookies: CookieMap = {}
  for (const [key, value] of Object.entries(record)) {
    if (typeof value === 'string') cookies[key] = value
  }
  return cookies
}

export const parseCookies = (input: string): CookieMap => {
  const trimmed = input.trim()
  try {
    const fromJson = asCookieRecord(JSON.parse(trimmed) as unknown)
    if (fromJson) return fromJson
  } catch {
    // 非 JSON，走 cookie 字符串
  }
  if (trimmed.includes('=')) return parseCookieString(trimmed)
  throw new Error('UNRECOGNIZED_CREDENTIALS')
}

export const extractApiUserFromCredentials = (input: string): string => {
  try {
    const json: unknown = JSON.parse(input.trim())
    if (!json || typeof json !== 'object' || Array.isArray(json)) return ''
    const record = json as Record<string, unknown>
    return record.api_user ? String(record.api_user) : ''
  } catch {
    return ''
  }
}
