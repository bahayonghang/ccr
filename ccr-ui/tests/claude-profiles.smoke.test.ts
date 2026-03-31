import { describe, expect, it } from 'vitest'
import { createClaudeProfileSections, filterClaudeProfiles, getClaudeProfileProviderKey } from '@/utils/claudeProfiles'
import type { ClaudeProfile } from '@/types'

const sampleProfiles: ClaudeProfile[] = [
  {
    name: 'zeta-current',
    provider: 'Zeta Relay',
    provider_type: 'official',
    description: 'Primary API relay for production traffic',
    base_url: 'https://relay.zeta.ai',
    model: 'claude-sonnet-4-5',
    small_fast_model: 'claude-3-5-haiku',
    account: 'github_5962',
    tags: ['prod', 'backup'],
    enabled: true,
    is_current: true,
  },
  {
    name: 'anthropic-a',
    provider: 'Anthropic',
    provider_type: 'api',
    description: 'Direct production account',
    base_url: 'https://api.anthropic.com',
    model: 'claude-opus-4-1',
    account: 'work-account',
    tags: ['production'],
    enabled: true,
    is_current: false,
  },
  {
    name: 'anthropic-b',
    provider: 'anthropic',
    provider_type: 'api',
    description: 'Fast fallback route',
    small_fast_model: 'CLAUDE-3-5-HAIKU',
    tags: ['staging'],
    enabled: false,
    is_current: false,
  },
  {
    name: 'missing-provider',
    description: 'Temporary local sandbox',
    base_url: 'https://sandbox.internal',
    tags: ['local'],
    enabled: true,
    is_current: false,
  },
]

describe('claude profiles utils', () => {
  it('sorts the current provider section first and keeps other sections alphabetical', () => {
    const sections = createClaudeProfileSections(sampleProfiles, 'Unspecified Provider')

    expect(sections.map(section => section.title)).toEqual([
      'Zeta Relay',
      'Anthropic',
      'Unspecified Provider',
    ])
    expect(sections[0]?.profiles[0]?.name).toBe('zeta-current')
  })

  it('uses the fallback label when provider is missing', () => {
    const sections = createClaudeProfileSections(sampleProfiles, 'Unspecified Provider')
    const fallbackSection = sections.find(section => section.providerKey === '__unset_provider__')

    expect(fallbackSection?.title).toBe('Unspecified Provider')
    expect(fallbackSection?.count).toBe(1)
  })

  it('normalizes provider keys case-insensitively', () => {
    expect(getClaudeProfileProviderKey('Anthropic')).toBe(getClaudeProfileProviderKey('anthropic'))
    expect(getClaudeProfileProviderKey()).toBe('__unset_provider__')
  })

  it('filters profiles case-insensitively across searchable fields', () => {
    expect(filterClaudeProfiles(sampleProfiles, 'github_5962').map(profile => profile.name)).toEqual(['zeta-current'])
    expect(filterClaudeProfiles(sampleProfiles, 'haiku').map(profile => profile.name)).toEqual(['zeta-current', 'anthropic-b'])
    expect(filterClaudeProfiles(sampleProfiles, 'backup').map(profile => profile.name)).toEqual(['zeta-current'])
    expect(filterClaudeProfiles(sampleProfiles, 'API').map(profile => profile.name)).toEqual(['zeta-current', 'anthropic-a', 'anthropic-b'])
  })

  it('keeps current-provider-first ordering after filtering', () => {
    const filteredSections = createClaudeProfileSections(
      filterClaudeProfiles(sampleProfiles, 'api'),
      'Unspecified Provider'
    )

    expect(filteredSections.map(section => section.title)).toEqual([
      'Zeta Relay',
      'Anthropic',
    ])
  })

  it('returns an empty result when the query does not match any searchable field', () => {
    expect(filterClaudeProfiles(sampleProfiles, 'no-such-profile')).toEqual([])
  })
})
