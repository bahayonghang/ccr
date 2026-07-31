import { describe, expect, it } from 'vitest'

import { buildProfileDiff, type ProfileDiffField } from '@/utils/profileDiff'

interface SampleProfile {
  name: string
  base_url?: string | null
  model?: string | null
  auth_mode?: string | null
}

const fields: ProfileDiffField<SampleProfile>[] = [
  { key: 'base_url', label: 'BASE URL', value: p => p.base_url },
  { key: 'model', label: 'MODEL', value: p => p.model },
  { key: 'auth_mode', label: '认证模式', value: p => p.auth_mode },
]

describe('buildProfileDiff smoke', () => {
  it('marks identical rows as unchanged and different rows as changed', () => {
    const current: SampleProfile = {
      name: 'a',
      base_url: 'https://api.anthropic.com',
      model: 'claude-sonnet-4-5',
      auth_mode: 'api_key',
    }
    const target: SampleProfile = {
      name: 'b',
      base_url: 'https://api.anthropic.com',
      model: 'claude-opus-4-1',
      auth_mode: 'api_key',
    }

    const rows = buildProfileDiff(current, target, fields)

    expect(rows).toHaveLength(3)
    expect(rows[0]).toMatchObject({ key: 'base_url', changed: false })
    expect(rows[1]).toMatchObject({
      key: 'model',
      from: 'claude-sonnet-4-5',
      to: 'claude-opus-4-1',
      changed: true,
    })
    expect(rows[2]).toMatchObject({ key: 'auth_mode', changed: false })
  })

  it('treats missing values as null and detects one-sided missing as changed', () => {
    const current: SampleProfile = { name: 'a', model: 'claude-sonnet-4-5' }
    const target: SampleProfile = { name: 'b', base_url: 'https://relay.example.com' }

    const rows = buildProfileDiff(current, target, fields)
    const byKey = Object.fromEntries(rows.map(row => [row.key, row]))

    expect(byKey.base_url).toMatchObject({ from: null, to: 'https://relay.example.com', changed: true })
    expect(byKey.model).toMatchObject({ from: 'claude-sonnet-4-5', to: null, changed: true })
    expect(byKey.auth_mode).toMatchObject({ from: null, to: null, changed: false })
  })

  it('has null from on every row when there is no current profile', () => {
    const target: SampleProfile = { name: 'b', base_url: 'https://relay.example.com' }

    const rows = buildProfileDiff(null, target, fields)
    const byKey = Object.fromEntries(rows.map(row => [row.key, row]))

    expect(rows.every(row => row.from === null)).toBe(true)
    // 目标有值的行算差异；目标同样缺失的行仍为未变化
    expect(byKey.base_url.changed).toBe(true)
    expect(byKey.model.changed).toBe(false)
  })

  it('normalizes whitespace-only and padded values before comparing', () => {
    const current: SampleProfile = { name: 'a', base_url: '  https://api.anthropic.com  ', model: '   ' }
    const target: SampleProfile = { name: 'b', base_url: 'https://api.anthropic.com' }

    const rows = buildProfileDiff(current, target, fields)
    const byKey = Object.fromEntries(rows.map(row => [row.key, row]))

    expect(byKey.base_url.changed).toBe(false)
    expect(byKey.model).toMatchObject({ from: null, to: null, changed: false })
  })
})
