const MAX_DEPTH = 4
const MAX_JSON_BYTES = 8192
const MAX_ARRAY_LEN = 32

const SENSITIVE_KEYS = new Set([
  'token',
  'apikey',
  'authorization',
  'cookie',
  'cookies',
  'password',
  'secret',
  'bearer',
  'accesstoken',
  'refreshtoken',
  'sessiontoken',
  'privatekey',
  'clientsecret',
  'authjson',
  'cookiesjson',
])

export const maskSensitive = (value: string): string => {
  const chars = Array.from(value)
  if (chars.length <= 10) {
    return '*'.repeat(chars.length)
  }
  return `${chars.slice(0, 4).join('')}...${chars.slice(-4).join('')}`
}

export const normalizeLogKey = (key: string): string => {
  return Array.from(key)
    .filter((char) => /[A-Za-z0-9]/.test(char))
    .join('')
    .toLowerCase()
}

export const isSensitiveLogKey = (key: string): boolean => {
  return SENSITIVE_KEYS.has(normalizeLogKey(key))
}

export const redactLogText = (input: string): string => {
  const trimmed = input.trim()
  try {
    const parsed: unknown = JSON.parse(trimmed)
    if (parsed && typeof parsed === 'object') {
      return JSON.stringify(redactLogValue(parsed))
    }
  } catch {
    // 整段不是 JSON，走自由文本识别。
  }
  return redactFreeText(input)
}

export const redactLogValue = (value: unknown): unknown => {
  return redactValueInner(value, 0, 0, undefined)[0]
}

const truncatedObject = (): Record<string, boolean> => ({ truncated: true })

const estimateValueBytes = (value: unknown): number => {
  try {
    return new TextEncoder().encode(JSON.stringify(value)).length
  } catch {
    return 0
  }
}

const redactValueInner = (
  value: unknown,
  depth: number,
  usedBytes: number,
  parentKey: string | undefined,
): [unknown, number] => {
  if (depth > MAX_DEPTH || usedBytes > MAX_JSON_BYTES) {
    return [truncatedObject(), usedBytes]
  }

  if (Array.isArray(value)) {
    const parentSensitive = parentKey !== undefined && isSensitiveLogKey(parentKey)
    const out: unknown[] = []
    let bytes = usedBytes
    for (const [index, item] of value.entries()) {
      if (index >= MAX_ARRAY_LEN) {
        break
      }
      if (parentSensitive) {
        const masked = typeof item === 'string' ? maskSensitive(item) : maskSensitive(JSON.stringify(item))
        bytes += estimateValueBytes(masked)
        out.push(masked)
      } else {
        const [redacted, nextBytes] = redactValueInner(item, depth + 1, bytes, undefined)
        bytes = nextBytes
        out.push(redacted)
      }
    }
    return [out, bytes]
  }

  if (value && typeof value === 'object') {
    const out: Record<string, unknown> = {}
    let bytes = usedBytes
    for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
      bytes += key.length
      if (bytes > MAX_JSON_BYTES) {
        return [truncatedObject(), bytes]
      }
      if (isSensitiveLogKey(key)) {
        const masked = typeof child === 'string' ? maskSensitive(child) : maskSensitive(JSON.stringify(child))
        bytes += estimateValueBytes(masked)
        out[key] = masked
      } else {
        const [redacted, nextBytes] = redactValueInner(child, depth + 1, bytes, key)
        bytes = nextBytes
        out[key] = redacted
      }
    }
    return [out, bytes]
  }

  if (typeof value === 'string') {
    const redacted = redactLogText(value)
    return [redacted, usedBytes + redacted.length]
  }

  return [value, usedBytes + estimateValueBytes(value)]
}

const isTokenChar = (char: string): boolean => /[A-Za-z0-9._\-+/=]/.test(char)
const isSkChar = (char: string): boolean => /[A-Za-z0-9_-]/.test(char)
const isJwtChar = (char: string): boolean => /[A-Za-z0-9_-]/.test(char)

const redactFreeText = (input: string): string => {
  const chars = Array.from(input)
  let output = ''
  let index = 0

  while (index < chars.length) {
    const cookie = matchCookieHeader(chars, index)
    const bearer = cookie ?? matchBearer(chars, index)
    const sk = bearer ?? matchSkToken(chars, index)
    const jwt = sk ?? matchJwt(chars, index)
    if (jwt) {
      output += jwt.replacement
      index += jwt.consumed
      continue
    }
    output += chars[index]
    index += 1
  }

  return output
}

const startsWithIgnoreCase = (chars: string[], start: number, needle: string): boolean => {
  const needleChars = Array.from(needle)
  if (start + needleChars.length > chars.length) {
    return false
  }
  return needleChars.every((char, offset) => chars[start + offset].toLowerCase() === char.toLowerCase())
}

const startsWithLiteral = (chars: string[], start: number, needle: string): boolean => {
  const needleChars = Array.from(needle)
  if (start + needleChars.length > chars.length) {
    return false
  }
  return needleChars.every((char, offset) => chars[start + offset] === char)
}

const matchCookieHeader = (
  chars: string[],
  start: number,
): { consumed: number, replacement: string } | null => {
  for (const header of ['cookie:', 'set-cookie:']) {
    if (!startsWithIgnoreCase(chars, start, header)) {
      continue
    }
    let end = start + header.length
    while (end < chars.length && chars[end] !== '\n' && chars[end] !== '\r') {
      end += 1
    }
    const prefix = chars.slice(start, start + header.length).join('')
    const value = chars.slice(start + header.length, end).join('')
    return {
      consumed: end - start,
      replacement: `${prefix}${maskSensitive(value.trim())}`,
    }
  }
  return null
}

const matchBearer = (
  chars: string[],
  start: number,
): { consumed: number, replacement: string } | null => {
  if (!startsWithIgnoreCase(chars, start, 'bearer')) {
    return null
  }
  let cursor = start + 6
  if (cursor >= chars.length || !/\s/.test(chars[cursor])) {
    return null
  }
  while (cursor < chars.length && /\s/.test(chars[cursor])) {
    cursor += 1
  }
  const tokenStart = cursor
  while (cursor < chars.length && isTokenChar(chars[cursor])) {
    cursor += 1
  }
  if (cursor - tokenStart < 8) {
    return null
  }
  const token = chars.slice(tokenStart, cursor).join('')
  return {
    consumed: cursor - start,
    replacement: `Bearer ${maskSensitive(token)}`,
  }
}

const matchSkToken = (
  chars: string[],
  start: number,
): { consumed: number, replacement: string } | null => {
  if (start + 3 >= chars.length) {
    return null
  }
  if (start > 0 && /[A-Za-z0-9]/.test(chars[start - 1])) {
    return null
  }
  if (chars[start] !== 's' || chars[start + 1] !== 'k' || chars[start + 2] !== '-') {
    return null
  }
  let cursor = start + 3
  while (cursor < chars.length && isSkChar(chars[cursor])) {
    cursor += 1
  }
  if (cursor - start < 11) {
    return null
  }
  return {
    consumed: cursor - start,
    replacement: maskSensitive(chars.slice(start, cursor).join('')),
  }
}

const consumeJwtPart = (chars: string[], start: number): number | null => {
  let cursor = start
  while (cursor < chars.length && isJwtChar(chars[cursor])) {
    cursor += 1
  }
  return cursor - start >= 8 ? cursor : null
}

const matchJwt = (
  chars: string[],
  start: number,
): { consumed: number, replacement: string } | null => {
  if (start > 0 && /[A-Za-z0-9]/.test(chars[start - 1])) {
    return null
  }
  if (!startsWithLiteral(chars, start, 'eyJ')) {
    return null
  }
  const first = consumeJwtPart(chars, start)
  if (first === null || first >= chars.length || chars[first] !== '.') {
    return null
  }
  const second = consumeJwtPart(chars, first + 1)
  if (second === null || second >= chars.length || chars[second] !== '.') {
    return null
  }
  const third = consumeJwtPart(chars, second + 1)
  if (third === null || third - start < 20) {
    return null
  }
  return {
    consumed: third - start,
    replacement: maskSensitive(chars.slice(start, third).join('')),
  }
}
