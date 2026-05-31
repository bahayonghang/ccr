import type { ClaudeProfile } from '@/types'

export const CLAUDE_PROFILE_UNSET_PROVIDER_KEY = '__unset_provider__'

// ── Provider 色彩映射系统 ─────────────────────────────────────────

/** Provider 色彩配置对象 */
export interface ProviderColorConfig {
  /** 平台标识键 (如 'claude', 'codex', 'gemini') */
  key: string
  /** CSS 变量名 (如 '--color-platform-claude') */
  cssVar: string
  /** RGB CSS 变量名 (如 '--color-platform-claude-rgb')，用于 alpha 合成 */
  rgbVar: string
  /** Tailwind 平台色类名前缀 (如 'platform-claude') */
  tailwindClass: string
}

/** provider 关键词 → 平台色彩映射表 */
const PROVIDER_COLOR_MAP: Record<string, ProviderColorConfig> = {
  anthropic: {
    key: 'claude',
    cssVar: '--color-platform-claude',
    rgbVar: '--color-platform-claude-rgb',
    tailwindClass: 'platform-claude',
  },
  claude: {
    key: 'claude',
    cssVar: '--color-platform-claude',
    rgbVar: '--color-platform-claude-rgb',
    tailwindClass: 'platform-claude',
  },
  openai: {
    key: 'codex',
    cssVar: '--color-platform-codex',
    rgbVar: '--color-platform-codex-rgb',
    tailwindClass: 'platform-codex',
  },
  codex: {
    key: 'codex',
    cssVar: '--color-platform-codex',
    rgbVar: '--color-platform-codex-rgb',
    tailwindClass: 'platform-codex',
  },
  google: {
    key: 'gemini',
    cssVar: '--color-platform-gemini',
    rgbVar: '--color-platform-gemini-rgb',
    tailwindClass: 'platform-gemini',
  },
  gemini: {
    key: 'gemini',
    cssVar: '--color-platform-gemini',
    rgbVar: '--color-platform-gemini-rgb',
    tailwindClass: 'platform-gemini',
  },
}

/** 默认回退色彩 (accent-secondary 紫) */
const DEFAULT_PROVIDER_COLOR: ProviderColorConfig = {
  key: 'default',
  cssVar: '--color-accent-secondary',
  rgbVar: '--color-accent-secondary-rgb',
  tailwindClass: 'accent-secondary',
}

/**
 * 根据 provider 名称解析对应的平台色彩配置。
 * 支持精确匹配 + includes 模糊回退，大小写不敏感。
 */
export const resolveProviderColor = (provider?: string | null): ProviderColorConfig => {
  if (!provider?.trim()) return DEFAULT_PROVIDER_COLOR

  const lower = provider.trim().toLowerCase()

  // 精确匹配
  if (PROVIDER_COLOR_MAP[lower]) return PROVIDER_COLOR_MAP[lower]

  // includes 模糊回退
  for (const [keyword, config] of Object.entries(PROVIDER_COLOR_MAP)) {
    if (lower.includes(keyword) || keyword.includes(lower)) return config
  }

  return DEFAULT_PROVIDER_COLOR
}

/** Provider 关键词 → Lucide 图标名映射 */
const PROVIDER_ICON_MAP: Record<string, string> = {
  anthropic: 'Sparkles',
  claude: 'Sparkles',
  openai: 'Cpu',
  codex: 'Cpu',
  google: 'Globe',
  gemini: 'Globe',
}

/**
 * 根据 provider 名称解析对应的 Lucide 图标名。
 * 未知 provider 回退到 'Server'。
 */
export const resolveProviderIcon = (provider?: string | null): string => {
  if (!provider?.trim()) return 'Server'

  const lower = provider.trim().toLowerCase()

  if (PROVIDER_ICON_MAP[lower]) return PROVIDER_ICON_MAP[lower]

  for (const [keyword, icon] of Object.entries(PROVIDER_ICON_MAP)) {
    if (lower.includes(keyword) || keyword.includes(lower)) return icon
  }

  return 'Server'
}

/**
 * 对文本中的搜索关键词进行高亮包裹。
 * 返回包含 `<mark>` 标签的 HTML 字符串，需配合 `v-html` 使用。
 * 输入文本会进行 HTML 实体转义以防止 XSS。
 */
export const highlightSearchMatch = (text: string, query: string): string => {
  if (!query.trim() || !text) return escapeHtml(text)

  const escaped = escapeHtml(text)
  const escapedQuery = query.trim().replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${escapedQuery})`, 'gi')

  return escaped.replace(regex, '<mark class="profile-search-highlight">$1</mark>')
}

/** HTML 实体转义（防 XSS） */
const escapeHtml = (str: string): string =>
  str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')

/**
 * 将 profile 列表按 provider 分组，附带色彩配置。
 * 用于 Quick Switch 区域的分组展示。
 */
export interface ProviderGroup {
  providerKey: string
  label: string
  color: ProviderColorConfig
  icon: string
  profiles: ClaudeProfile[]
}

export const groupProfilesByProvider = (
  profiles: ClaudeProfile[],
  unsetLabel = 'Other'
): ProviderGroup[] => {
  const groupMap = new Map<string, ProviderGroup>()

  for (const profile of profiles) {
    const providerKey = getClaudeProfileProviderKey(profile.provider)
    const label = getClaudeProfileProviderLabel(profile.provider, unsetLabel)
    const existing = groupMap.get(providerKey)

    if (existing) {
      existing.profiles.push(profile)
    } else {
      groupMap.set(providerKey, {
        providerKey,
        label,
        color: resolveProviderColor(profile.provider),
        icon: resolveProviderIcon(profile.provider),
        profiles: [profile],
      })
    }
  }

  return Array.from(groupMap.values())
}

// ── 原有接口与函数 ───────────────────────────────────────────────

export interface ClaudeProfileSection {
  id: string
  providerKey: string
  title: string
  count: number
  enabledCount: number
  isCurrentProvider: boolean
  profiles: ClaudeProfile[]
}

export interface NormalizedClaudeProfilesState {
  currentProfile: string | null
  profiles: ClaudeProfile[]
  warnings: string[]
}

const normalizeProvider = (provider?: string | null): string => provider?.trim() ?? ''
const normalizeSearchValue = (value?: string | null): string => value?.trim().toLowerCase() ?? ''
const normalizeOptionalValue = (value?: string | null): string => value?.trim() ?? ''

const OFFICIAL_CLAUDE_BASE_URLS = new Set([
  'https://api.anthropic.com',
  'https://api.anthropic.com/v1',
])

const normalizeBaseUrl = (baseUrl?: string | null): string =>
  normalizeOptionalValue(baseUrl).replace(/\/+$/, '')

export const isCustomClaudeProfileBaseUrl = (baseUrl?: string | null): boolean => {
  const normalized = normalizeBaseUrl(baseUrl)
  return !!normalized && !OFFICIAL_CLAUDE_BASE_URLS.has(normalized)
}

export const isOfficialClaudeProfileBaseUrl = (baseUrl?: string | null): boolean => {
  const normalized = normalizeBaseUrl(baseUrl)
  return !!normalized && OFFICIAL_CLAUDE_BASE_URLS.has(normalized)
}

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

export const normalizeClaudeProfilesState = (
  profiles: ClaudeProfile[],
  currentProfile: string | null | undefined
): NormalizedClaudeProfilesState => {
  const warnings: string[] = []
  const profileNames = new Set(profiles.map((profile) => profile.name))
  const trimmedCurrentProfile = currentProfile?.trim() || null
  const flaggedCurrentProfiles = profiles
    .filter((profile) => profile.is_current)
    .map((profile) => profile.name)

  let canonicalCurrentProfile =
    trimmedCurrentProfile && profileNames.has(trimmedCurrentProfile) ? trimmedCurrentProfile : null

  if (trimmedCurrentProfile && !canonicalCurrentProfile) {
    warnings.push(
      `Unknown current_profile "${trimmedCurrentProfile}" returned by Claude profiles API`
    )
  }

  if (!canonicalCurrentProfile && flaggedCurrentProfiles.length > 0) {
    canonicalCurrentProfile = flaggedCurrentProfiles[0] || null
  }

  if (flaggedCurrentProfiles.length > 1) {
    warnings.push(`Multiple profiles marked as current: ${flaggedCurrentProfiles.join(', ')}`)
  }

  if (
    canonicalCurrentProfile &&
    flaggedCurrentProfiles.length > 0 &&
    (flaggedCurrentProfiles.length !== 1 || flaggedCurrentProfiles[0] !== canonicalCurrentProfile)
  ) {
    warnings.push(
      `Current profile mismatch between current_profile and row flags; normalized to "${canonicalCurrentProfile}"`
    )
  }

  return {
    currentProfile: canonicalCurrentProfile,
    profiles: profiles.map((profile) => ({
      ...profile,
      is_current: canonicalCurrentProfile === profile.name,
    })),
    warnings,
  }
}

export const filterClaudeProfiles = (profiles: ClaudeProfile[], query: string): ClaudeProfile[] => {
  const normalizedQuery = normalizeSearchValue(query)

  if (!normalizedQuery) {
    return profiles
  }

  return profiles.filter((profile) => {
    const fields = [
      profile.name,
      profile.description,
      profile.provider,
      profile.provider_type,
      profile.account,
      profile.base_url,
      profile.model,
      profile.small_fast_model,
      ...(profile.tags ?? []),
    ]

    return fields.some((field) => normalizeSearchValue(field).includes(normalizedQuery))
  })
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
    .map((section) => ({
      ...section,
      count: section.profiles.length,
      enabledCount: section.profiles.filter((profile) => profile.enabled !== false).length,
      profiles: [...section.profiles].sort(compareProfiles),
    }))
    .sort((left, right) => {
      if (left.isCurrentProvider !== right.isCurrentProvider) {
        return left.isCurrentProvider ? -1 : 1
      }

      return left.title.localeCompare(right.title, undefined, { sensitivity: 'base' })
    })
}
