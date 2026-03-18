import type { ClaudeProfile } from '@/types'

export const CLAUDE_PROFILE_UNSET_PROVIDER_KEY = '__unset_provider__'

export interface ClaudeProfileSection {
  id: string
  providerKey: string
  title: string
  count: number
  enabledCount: number
  isCurrentProvider: boolean
  profiles: ClaudeProfile[]
}

const normalizeProvider = (provider?: string | null): string => provider?.trim() ?? ''

const compareProfiles = (left: ClaudeProfile, right: ClaudeProfile): number => {
  if (left.is_current !== right.is_current) {
    return left.is_current ? -1 : 1
  }

  return left.name.localeCompare(right.name, undefined, { sensitivity: 'base' })
}

const buildSectionId = (label: string): string => {
  const slug = label
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')

  return `claude-provider-${slug || 'unset'}`
}

export const getClaudeProfileProviderKey = (provider?: string | null): string => {
  const normalized = normalizeProvider(provider)
  return normalized ? normalized.toLowerCase() : CLAUDE_PROFILE_UNSET_PROVIDER_KEY
}

export const getClaudeProfileProviderLabel = (
  provider?: string | null,
  unsetLabel = 'Unspecified Provider'
): string => {
  const normalized = normalizeProvider(provider)
  return normalized || unsetLabel
}

export const createClaudeProfileSections = (
  profiles: ClaudeProfile[],
  unsetLabel: string
): ClaudeProfileSection[] => {
  const sectionMap = new Map<string, ClaudeProfileSection>()

  for (const profile of profiles) {
    const providerKey = getClaudeProfileProviderKey(profile.provider)
    const providerLabel = getClaudeProfileProviderLabel(profile.provider, unsetLabel)
    const existingSection = sectionMap.get(providerKey)

    if (!existingSection) {
      sectionMap.set(providerKey, {
        id: buildSectionId(providerLabel),
        providerKey,
        title: providerLabel,
        count: 0,
        enabledCount: 0,
        isCurrentProvider: !!profile.is_current,
        profiles: [profile],
      })
      continue
    }

    existingSection.profiles.push(profile)
    existingSection.isCurrentProvider = existingSection.isCurrentProvider || !!profile.is_current

    if (profile.is_current && providerKey !== CLAUDE_PROFILE_UNSET_PROVIDER_KEY) {
      existingSection.title = providerLabel
      existingSection.id = buildSectionId(providerLabel)
    }
  }

  return Array.from(sectionMap.values())
    .map(section => ({
      ...section,
      count: section.profiles.length,
      enabledCount: section.profiles.filter(profile => profile.enabled !== false).length,
      profiles: [...section.profiles].sort(compareProfiles),
    }))
    .sort((left, right) => {
      if (left.isCurrentProvider !== right.isCurrentProvider) {
        return left.isCurrentProvider ? -1 : 1
      }

      return left.title.localeCompare(right.title, undefined, { sensitivity: 'base' })
    })
}
