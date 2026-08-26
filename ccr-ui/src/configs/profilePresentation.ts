import { toVendorKey, type ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import type { ClaudeProfile, CodexProfile, GrokProfileDto } from '@/types'

export interface ProfileFieldSlot {
  /** 列头与卡片字段的 label i18n key */
  labelKey: string
  /** 表格列宽 */
  columnWidth: string
  /** 表格中是否渲染为 chip */
  chip?: boolean
}

export interface ProfilePresentation<TRecord = unknown> {
  key: string
  /** 页头字形方块内的单字母 */
  glyph: string
  nameKey: string
  /** 配置文件名，显示在面包屑右侧徽标 */
  configFile: string
  /** 配置路径说明 i18n key */
  configPathKey: string
  fieldSlots: readonly [ProfileFieldSlot, ProfileFieldSlot, ProfileFieldSlot, ProfileFieldSlot]
  /** 平台 typed DTO → 展示投影。入参已经过凭据剥离 */
  project: (record: TRecord, ctx: { current: string | null }) => ProfileDisplayRecord
}

/** antigravity 尚无 typed DTO；层二注册用结构占位，不进入路由。 */
export interface AntigravityProfileRecord {
  name: string
  description?: string | null
  base_url?: string | null
  model?: string | null
  auth_mode?: string | null
  region?: string | null
  tags?: readonly string[] | null
  enabled?: boolean | null
}

const slot = (value: string | null | undefined): string => {
  if (typeof value !== 'string') return ''
  return value
}

const tagsOf = (tags: readonly string[] | null | undefined): readonly string[] =>
  tags ?? []

const flattenSearchPart = (
  part: string | null | undefined | readonly string[],
): string[] => {
  if (part == null) return []
  if (typeof part === 'string') return part ? [part] : []
  const items: string[] = []
  for (const item of part) {
    if (item) items.push(item)
  }
  return items
}

const joinSearch = (
  parts: Array<string | null | undefined | readonly string[]>,
): string => parts.flatMap(flattenSearchPart).join(' ').toLowerCase()

const isCurrent = (
  name: string,
  flag: boolean | undefined,
  ctx: { current: string | null },
): boolean => {
  if (ctx.current != null) return name === ctx.current
  return flag === true
}

const authLabelKey = (authKey: string): string => `profilePresentation.auth.${authKey}`

export const claudeProfilePresentation: ProfilePresentation<ClaudeProfile> = {
  key: 'claude',
  glyph: 'C',
  nameKey: 'profilePresentation.name.claude',
  configFile: 'profiles.toml',
  configPathKey: 'profilePresentation.configPath.claude',
  fieldSlots: [
    { labelKey: 'profilePresentation.fields.baseUrl', columnWidth: '13.5rem' },
    { labelKey: 'profilePresentation.fields.model', columnWidth: '11rem' },
    { labelKey: 'profilePresentation.fields.authMode', columnWidth: '6.5rem', chip: true },
    { labelKey: 'profilePresentation.fields.provider', columnWidth: '8.5rem' },
  ],
  project: (record, ctx) => {
    const tags = tagsOf(record.tags)
    const authKey = record.auth_mode ?? 'subscription'
    return {
      name: record.name,
      description: slot(record.description),
      enabled: record.enabled !== false,
      current: isCurrent(record.name, record.is_current, ctx),
      tags,
      slots: [
        slot(record.base_url),
        slot(record.model),
        slot(record.auth_mode),
        slot(record.provider),
      ],
      searchText: joinSearch([record.name, record.description, record.base_url, tags]),
      vendorKey: toVendorKey(record.base_url),
      authKey,
      authLabelKey: authLabelKey(authKey),
      badges: [],
      sortKeys: { name: record.name, usageCount: record.usage_count ?? 0 },
    }
  },
}

export const codexProfilePresentation: ProfilePresentation<CodexProfile> = {
  key: 'codex',
  glyph: 'X',
  nameKey: 'profilePresentation.name.codex',
  configFile: 'CCR Unified',
  configPathKey: 'profilePresentation.configPath.codex',
  fieldSlots: [
    { labelKey: 'profilePresentation.fields.baseUrl', columnWidth: '13.5rem' },
    { labelKey: 'profilePresentation.fields.model', columnWidth: '11rem' },
    { labelKey: 'profilePresentation.fields.authMode', columnWidth: '6.5rem', chip: true },
    { labelKey: 'profilePresentation.fields.wireApi', columnWidth: '8.5rem' },
  ],
  project: (record, ctx) => {
    const tags = tagsOf(record.tags)
    const authKey = record.auth_mode ?? 'no_auth'
    const badges: Array<ProfileDisplayRecord['badges'][number]> = []
    if (record.auth_source) {
      badges.push({ labelKey: 'profilePresentation.badges.auth_source', tone: 'neutral' })
    }
    if (record.openai_login_method === 'chatgpt') {
      badges.push({ labelKey: 'profilePresentation.badges.openai_login_chatgpt', tone: 'accent' })
    } else if (record.openai_login_method === 'api') {
      badges.push({ labelKey: 'profilePresentation.badges.openai_login_api', tone: 'neutral' })
    }
    return {
      name: record.name,
      description: slot(record.description),
      enabled: record.enabled !== false,
      current: isCurrent(record.name, record.is_current, ctx),
      tags,
      slots: [
        slot(record.base_url),
        slot(record.model),
        slot(record.auth_mode),
        slot(record.wire_api),
      ],
      searchText: joinSearch([record.name, record.description, record.base_url, tags]),
      vendorKey: toVendorKey(record.base_url),
      authKey,
      authLabelKey: authLabelKey(authKey),
      badges,
      sortKeys: { name: record.name, usageCount: record.usage_count ?? 0 },
    }
  },
}

export const grokProfilePresentation: ProfilePresentation<GrokProfileDto> = {
  key: 'grok',
  glyph: 'G',
  nameKey: 'profilePresentation.name.grok',
  configFile: 'grok.toml',
  configPathKey: 'profilePresentation.configPath.grok',
  fieldSlots: [
    { labelKey: 'profilePresentation.fields.baseUrl', columnWidth: '13.5rem' },
    { labelKey: 'profilePresentation.fields.model', columnWidth: '11rem' },
    { labelKey: 'profilePresentation.fields.authMode', columnWidth: '6.5rem', chip: true },
    { labelKey: 'profilePresentation.fields.reasoningEffort', columnWidth: '8.5rem' },
  ],
  project: (record, ctx) => {
    const tags = tagsOf(record.tags)
    const authKey = record.profile_kind === 'official' ? 'official' : record.auth_mode
    const kindKey =
      record.profile_kind === 'official'
        ? 'profilePresentation.badges.official'
        : 'profilePresentation.badges.third_party'
    return {
      name: record.name,
      description: slot(record.description),
      enabled: record.enabled !== false,
      current: isCurrent(record.name, undefined, ctx),
      tags,
      slots: [
        slot(record.base_url_display),
        slot(record.model),
        slot(record.auth_mode),
        slot(record.reasoning_effort),
      ],
      searchText: joinSearch([
        record.name,
        record.description,
        record.base_url_display,
        tags,
      ]),
      vendorKey: toVendorKey(record.base_url_display),
      authKey,
      authLabelKey: authLabelKey(authKey),
      badges: [
        {
          labelKey: kindKey,
          tone: record.profile_kind === 'official' ? 'accent' : 'neutral',
        },
      ],
      sortKeys: { name: record.name, usageCount: 0 },
    }
  },
}

export const antigravityProfilePresentation: ProfilePresentation<AntigravityProfileRecord> = {
  key: 'antigravity',
  glyph: 'A',
  nameKey: 'profilePresentation.name.antigravity',
  configFile: 'antigravity.toml',
  configPathKey: 'profilePresentation.configPath.antigravity',
  fieldSlots: [
    { labelKey: 'profilePresentation.fields.baseUrl', columnWidth: '13.5rem' },
    { labelKey: 'profilePresentation.fields.model', columnWidth: '11rem' },
    { labelKey: 'profilePresentation.fields.authMode', columnWidth: '6.5rem', chip: true },
    { labelKey: 'profilePresentation.fields.region', columnWidth: '8.5rem' },
  ],
  project: (record, ctx) => {
    const tags = tagsOf(record.tags)
    const authKey = record.auth_mode ?? 'api_key'
    return {
      name: record.name,
      description: slot(record.description),
      enabled: record.enabled !== false,
      current: isCurrent(record.name, undefined, ctx),
      tags,
      slots: [
        slot(record.base_url),
        slot(record.model),
        slot(record.auth_mode),
        slot(record.region),
      ],
      searchText: joinSearch([record.name, record.description, record.base_url, tags]),
      vendorKey: toVendorKey(record.base_url),
      authKey,
      authLabelKey: authLabelKey(authKey),
      badges: [],
      sortKeys: { name: record.name, usageCount: 0 },
    }
  },
}

export const profilePresentations = {
  claude: claudeProfilePresentation,
  codex: codexProfilePresentation,
  grok: grokProfilePresentation,
  antigravity: antigravityProfilePresentation,
} as const
