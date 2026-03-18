import { describe, expect, it } from 'vitest'
import { createClaudeProfileSections, getClaudeProfileProviderKey } from '@/utils/claudeProfiles'
import type { ClaudeProfile } from '@/types'

const sampleProfiles: ClaudeProfile[] = [
  {
    name: 'zeta-current',
    provider: 'Zeta Relay',
    provider_type: 'official',
    enabled: true,
    is_current: true,
  },
  {
    name: 'anthropic-a',
    provider: 'Anthropic',
    provider_type: 'api',
    enabled: true,
    is_current: false,
  },
  {
    name: 'anthropic-b',
    provider: 'anthropic',
    provider_type: 'api',
    enabled: false,
    is_current: false,
  },
  {
    name: 'missing-provider',
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
})
