import { describe, expect, it } from 'vitest'
import { toVendorKey } from '@/configs/profileDisplayRecord'
import {
  antigravityProfilePresentation,
  claudeProfilePresentation,
  codexProfilePresentation,
  grokProfilePresentation,
  profilePresentations,
} from '@/configs/profilePresentation'
import {
  claudeProfileFixtures,
  codexProfileFixtures,
  grokProfileFixtures,
} from '../fixtures/profiles'

const PRESENTATIONS = [
  claudeProfilePresentation,
  codexProfilePresentation,
  grokProfilePresentation,
  antigravityProfilePresentation,
] as const

describe('ProfilePresentation 结构', () => {
  it('四份实例 glyph / configFile / configPathKey / fieldSlots 齐备', () => {
    expect(Object.keys(profilePresentations)).toEqual([
      'claude',
      'codex',
      'grok',
      'antigravity',
    ])

    for (const presentation of PRESENTATIONS) {
      expect(presentation.glyph).toHaveLength(1)
      expect(presentation.configFile.length).toBeGreaterThan(0)
      expect(presentation.configPathKey.length).toBeGreaterThan(0)
      expect(presentation.fieldSlots).toHaveLength(4)
      expect(presentation.fieldSlots[0].kind).toBe('url')
      expect(presentation.fieldSlots[1].kind).toBe('text')
      expect(presentation.fieldSlots[2].kind).toBe('chip')
      for (const slot of presentation.fieldSlots) {
        if (slot.kind === 'chip') expect(slot.chip).toBe(true)
        else expect(slot.chip).toBeFalsy()
      }
    }
    expect(claudeProfilePresentation.fieldSlots[3].kind).toBe('chip')
    expect(claudeProfilePresentation.fieldSlots[3].chip).toBe(true)
  })
})

describe('project() typed 投影', () => {
  it('Claude slots 来自 typed DTO 的 provider，而非 ProfileRecord 七字段', () => {
    const record = claudeProfileFixtures[0]
    const projected = claudeProfilePresentation.project(record, { current: record.name })

    expect(projected.slots).toEqual([
      record.base_url,
      record.model,
      record.auth_mode,
      record.provider,
    ])
    expect(projected.vendorKey).toBe(toVendorKey(record.base_url))
    expect(projected.authKey).toBe('api_key')
    expect(projected.badges).toEqual([])
    expect(projected.searchText).toContain('claude-current')
    expect(projected.searchText).toContain('api.anthropic.com')
    expect(projected.searchText).not.toContain('claude-secret-current')
    expect(projected.current).toBe(true)
    expect(record.provider).toBe('Anthropic')
  })

  it('Codex slots 来自 typed DTO 的 wire_api', () => {
    const record = codexProfileFixtures[0]
    const projected = codexProfilePresentation.project(record, { current: record.name })

    expect(projected.slots[3]).toBe(record.wire_api)
    expect(projected.vendorKey).toBe(toVendorKey(record.base_url))
    expect(projected.authKey).toBe('openai_api_key')
    expect(projected.badges.map((badge) => badge.labelKey)).toEqual([
      'profilePresentation.badges.auth_source',
      'profilePresentation.badges.openai_login_api',
    ])
    expect(projected.searchText).not.toContain('codex-secret-current')
  })

  it('Grok slots 来自 typed DTO 的 reasoning_effort，并输出 profile_kind badge', () => {
    const record = grokProfileFixtures[0]
    const projected = grokProfilePresentation.project(record, { current: record.name })

    expect(projected.slots[3]).toBe(record.reasoning_effort)
    expect(projected.badges).toEqual([
      { labelKey: 'profilePresentation.badges.official', tone: 'accent' },
    ])
    expect(projected.authKey).toBe('official')
    expect(projected.searchText).toContain('grok-current')
    expect(projected.searchText).not.toMatch(/https?:\/\//)
    expect(projected.slots.join('|')).not.toMatch(/https?:\/\//)
  })

  it('Grok display URL 只出现在 slots[0] 与 searchText，不进入写入形状', () => {
    const record = grokProfileFixtures[5]
    const projected = grokProfilePresentation.project(record, { current: null })
    const display = record.base_url_display ?? ''

    expect(projected.slots[0]).toBe(display)
    expect(projected.searchText).toContain(display.toLowerCase())
    expect(projected).not.toHaveProperty('base_url')
    expect(JSON.stringify(projected)).not.toContain('base_url_display')
  })
})
