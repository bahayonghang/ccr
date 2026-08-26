import { randomUUID } from 'node:crypto'
import { describe, expect, it } from 'vitest'
import {
  CLAUDE_SECRET_KEYS,
  CODEX_SECRET_KEYS,
  GROK_SECRET_KEYS,
  stripCredentials,
} from '@/configs/profileCredentials'
import {
  claudeProfileFixtures,
  codexProfileFixtures,
  grokProfileFixtures,
} from '../fixtures/profiles'

const containsSentinel = (value: unknown, sentinel: string): boolean => {
  if (typeof value === 'string') return value.includes(sentinel)
  if (Array.isArray(value)) return value.some((item) => containsSentinel(item, sentinel))
  if (value && typeof value === 'object') {
    return Object.values(value).some((item) => containsSentinel(item, sentinel))
  }
  return false
}

describe('stripCredentials 凭据剥离', () => {
  it('从 Claude DTO 任意深度移除 sentinel', () => {
    const sentinel = randomUUID()
    const input = {
      ...claudeProfileFixtures[0],
      auth_token: sentinel,
      extra: { nested: { auth_token: sentinel } },
    }
    const stripped = stripCredentials(input, CLAUDE_SECRET_KEYS)

    expect(stripped.auth_token).toBeUndefined()
    expect(containsSentinel(stripped, sentinel)).toBe(false)
    expect(input.auth_token).toBe(sentinel)
  })

  it('从 Codex DTO 任意深度移除 sentinel', () => {
    const sentinel = randomUUID()
    const input = {
      ...codexProfileFixtures[0],
      auth_token: sentinel,
    }
    const stripped = stripCredentials(input, CODEX_SECRET_KEYS)

    expect(stripped.auth_token).toBeUndefined()
    expect(containsSentinel(stripped, sentinel)).toBe(false)
  })

  it('Grok DTO 不含凭据字段，原样返回', () => {
    const input = grokProfileFixtures[0]
    const stripped = stripCredentials(input, GROK_SECRET_KEYS)
    expect(stripped).toEqual(input)
    expect(stripped).not.toBe(input)
  })
})
