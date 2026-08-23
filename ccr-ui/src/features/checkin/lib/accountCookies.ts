/** 从 cookies JSON 中提取表单展示值 */
export const extractCookiesFieldValue = (json: string): string => {
  const trimmed = json.trim()
  if (!trimmed) return ''

  const sessionOnly = readSessionOnly(trimmed)
  if (sessionOnly !== null) return sessionOnly
  return trimmed
}

const readSessionOnly = (trimmed: string): string | null => {
  try {
    const parsed: unknown = JSON.parse(trimmed)
    if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) return null
    const record = parsed as Record<string, unknown>
    if (Object.keys(record).length !== 1 || !('session' in record)) return null
    return typeof record.session === 'string' ? record.session : ''
  } catch {
    return null
  }
}

/** 将 session 值转换为 cookies JSON 格式 */
export const sessionToCookiesJson = (session: string): string => {
  const trimmed = session.trim()
  if (!trimmed) return ''

  if (trimmed.startsWith('{')) {
    try {
      JSON.parse(trimmed)
      return trimmed
    } catch {
      // 不是有效 JSON，当作 session 值处理
    }
  }

  return JSON.stringify({ session: trimmed })
}
