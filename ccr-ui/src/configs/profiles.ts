import {
  addClaudeProfile,
  addCodexProfile,
  applyClaudeProfile,
  applyCodexProfile,
  deleteClaudeProfile,
  deleteCodexProfile,
  exportClaudeProfiles,
  exportCodexProfiles,
  listClaudeProfiles,
  listCodexProfiles,
  updateClaudeProfile,
  updateCodexProfile,
} from '@/api'
import { claudeProfileOff } from '@/api/domains/claude'
import { codexProfileOff } from '@/api/domains/codex'
import {
  applyGrokProfile,
  deleteGrokProfile,
  grokProfileOff,
  listGrokProfiles,
} from '@/api/domains/grok'
import { probeLocalEnvironment, type EnvironmentProbe } from '@/configs/probeLocal'
import { surfaceNotify, type SurfaceNotify } from '@/configs/surfaceNotify'

export interface ProfileRecord {
  name: string
  description?: string | null
  enabled?: boolean | null
  tags?: string[] | null
  model?: string | null
  baseUrl?: string | null
  authMode?: string | null
}

export interface ProfileDraft {
  name: string
  description?: string
  model?: string
  tags?: string[]
}

export interface ProfilesSnapshot {
  profiles: ProfileRecord[]
  current: string | null
}

export interface ProfilesFeatures {
  commandPalette?: boolean
  quickRail?: boolean
  rawSource?: boolean
  localOnly?: boolean
  profileOff?: boolean
  recovery?: boolean
}

export interface ProfilesConfig {
  cacheKey: string
  homePath: string
  module: string
  i18nPrefix: string
  titleKey: string
  subtitleKey: string
  icon: string
  backTo: string
  editIcon: string
  features: ProfilesFeatures
  notify: SurfaceNotify
  probe?: () => Promise<EnvironmentProbe>
  list: () => Promise<ProfilesSnapshot>
  apply: (name: string) => Promise<void>
  remove: (name: string) => Promise<void>
  create?: (draft: ProfileDraft) => Promise<void>
  update?: (name: string, draft: ProfileDraft) => Promise<void>
  profileOff?: () => Promise<void>
  exportAll?: () => Promise<void>
}

const asRecord = (value: unknown): Record<string, unknown> =>
  value && typeof value === 'object' ? (value as Record<string, unknown>) : {}

const asName = (value: unknown): string => (typeof value === 'string' ? value : '')

const toProfile = (value: unknown): ProfileRecord | null => {
  const source = asRecord(value)
  const name = asName(source.name)
  if (!name) return null
  return {
    name,
    description: typeof source.description === 'string' ? source.description : null,
    enabled: typeof source.enabled === 'boolean' ? source.enabled : null,
    tags: Array.isArray(source.tags)
      ? source.tags.filter((item): item is string => typeof item === 'string')
      : null,
    model: typeof source.model === 'string' ? source.model : null,
    baseUrl: typeof source.base_url === 'string' ? source.base_url : typeof source.baseUrl === 'string' ? source.baseUrl : null,
    authMode: typeof source.auth_mode === 'string' ? source.auth_mode : typeof source.authMode === 'string' ? source.authMode : null,
  }
}

const readList = (payload: unknown, key: string): unknown[] => {
  const source = asRecord(payload)
  const nested = source[key]
  if (Array.isArray(nested)) return nested
  if (Array.isArray(payload)) return payload
  const profiles = source.profiles
  return Array.isArray(profiles) ? profiles : []
}

const readCurrent = (payload: unknown): string | null => {
  const source = asRecord(payload)
  const current = source.current ?? source.current_profile ?? source.currentProfile
  return typeof current === 'string' && current ? current : null
}

export const claudeProfilesConfig: ProfilesConfig = {
  cacheKey: 'profiles-claude',
  homePath: '/claude-code/profiles',
  module: 'claude-code',
  i18nPrefix: 'claudeProfiles',
  titleKey: 'claudeProfiles.title',
  subtitleKey: 'claudeProfiles.subtitle',
  icon: 'Layers',
  backTo: '/claude-code',
  editIcon: 'Pencil',
  features: { commandPalette: true, quickRail: true, rawSource: true, profileOff: true },
  notify: surfaceNotify,
  list: async () => {
    const payload = await listClaudeProfiles()
    return {
      profiles: readList(payload, 'profiles').map(toProfile).filter((item): item is ProfileRecord => item !== null),
      current: readCurrent(payload),
    }
  },
  apply: async (name) => {
    await applyClaudeProfile(name)
  },
  remove: async (name) => {
    await deleteClaudeProfile(name)
  },
  create: async (draft) => {
    await addClaudeProfile(draft)
  },
  update: async (name, draft) => {
    await updateClaudeProfile(name, draft)
  },
  profileOff: async () => {
    await claudeProfileOff()
  },
  exportAll: async () => {
    await exportClaudeProfiles()
  },
}

export const grokProfilesConfig: ProfilesConfig = {
  cacheKey: 'profiles-grok',
  homePath: '/grok/profiles',
  module: 'grok',
  i18nPrefix: 'grok.profiles',
  titleKey: 'grok.profiles.title',
  subtitleKey: 'grok.profiles.subtitle',
  icon: 'Layers',
  backTo: '/grok',
  editIcon: 'Pencil',
  features: { commandPalette: true, quickRail: true, localOnly: true, profileOff: true, recovery: true },
  notify: surfaceNotify,
  probe: probeLocalEnvironment,
  list: async () => {
    const payload = await listGrokProfiles()
    if ('status' in payload && payload.status === 'unsupported_environment') {
      return { profiles: [], current: null }
    }
    return {
      profiles: readList(payload, 'profiles').map(toProfile).filter((item): item is ProfileRecord => item !== null),
      current: readCurrent(payload),
    }
  },
  apply: async (name) => {
    await applyGrokProfile(name)
  },
  remove: async (name) => {
    await deleteGrokProfile(name)
  },
  profileOff: async () => {
    await grokProfileOff()
  },
}

export const codexProfilesConfig: ProfilesConfig = {
  cacheKey: 'profiles-codex',
  homePath: '/codex/profiles',
  module: 'codex',
  i18nPrefix: 'codex.profiles',
  titleKey: 'codex.profiles.title',
  subtitleKey: 'codex.profiles.subtitle',
  icon: 'Layers',
  backTo: '/codex',
  editIcon: 'Edit2',
  features: { commandPalette: true, quickRail: true, rawSource: true, profileOff: true },
  notify: surfaceNotify,
  list: async () => {
    const payload = await listCodexProfiles()
    return {
      profiles: readList(payload, 'profiles').map(toProfile).filter((item): item is ProfileRecord => item !== null),
      current: readCurrent(payload),
    }
  },
  apply: async (name) => {
    await applyCodexProfile(name)
  },
  remove: async (name) => {
    await deleteCodexProfile(name)
  },
  create: async (draft) => {
    await addCodexProfile(draft)
  },
  update: async (name, draft) => {
    await updateCodexProfile(name, draft)
  },
  profileOff: async () => {
    await codexProfileOff()
  },
  exportAll: async () => {
    await exportCodexProfiles()
  },
}

export const profilesConfigs = {
  claude: claudeProfilesConfig,
  grok: grokProfilesConfig,
  codex: codexProfilesConfig,
} as const
