export const CLAUDE_SECRET_KEYS = ['auth_token'] as const
export const CODEX_SECRET_KEYS = ['auth_token'] as const
export const GROK_SECRET_KEYS = [] as const

const removeSecretKeys = (value: unknown, keys: ReadonlySet<string>): void => {
  if (Array.isArray(value)) {
    for (const item of value) {
      removeSecretKeys(item, keys)
    }
    return
  }
  if (!value || typeof value !== 'object') return

  const record = value as Record<string, unknown>
  for (const key of keys) {
    if (Object.prototype.hasOwnProperty.call(record, key)) {
      delete record[key]
    }
  }
  for (const nested of Object.values(record)) {
    removeSecretKeys(nested, keys)
  }
}

/**
 * 深拷贝后删除指定凭据字段。入参不被修改。
 */
export function stripCredentials<T>(record: T, secretKeys: readonly string[]): T {
  const clone = structuredClone(record)
  removeSecretKeys(clone, new Set(secretKeys))
  return clone
}
