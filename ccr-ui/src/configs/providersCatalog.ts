// 共享站点目录（providers-catalog.json）的前端入口
//
// 数据源：crates/ccr-checkin/data/providers-catalog.json —— 仓库内单一事实源，
// Rust 侧通过 include_str! 内嵌（builtin_providers.rs），前端在此通过构建期
// import 引用同一份文件（Vite 原生支持 JSON import，dev/build 双模式均可解析；
// dev server 越过 ccr-ui 根目录的访问已在 vite.config.ts server.fs.allow 放行）。
import rawProvidersCatalog from '../../../crates/ccr-checkin/data/providers-catalog.json'
import type {
  ClaudeProviderTemplateOverride,
  CodexProviderTemplateOverride,
  OpenCodeProviderTemplateOverride,
  ProviderTemplate,
  ProviderTemplateCategory,
} from '@/types/providerTemplates'

/** 前端支持的 catalog schema 版本（与 Rust 侧 PROVIDERS_CATALOG_SCHEMA_VERSION 对齐） */
export const PROVIDERS_CATALOG_SCHEMA_VERSION = 1

// ═══════════════════════════════════════════════════════════
// catalog 类型（camelCase，镜像 Rust builtin_providers.rs 中的 Catalog* 结构）
// ═══════════════════════════════════════════════════════════

export interface ProvidersCatalog {
  schemaVersion: number
  providers: CatalogProviderEntry[]
}

export interface CatalogProviderEntry {
  id: string
  name: string
  description: string
  domain: string
  websiteUrl?: string
  icon: string
  /** 业务轴分类：community / commercial / official / aggregator / local */
  bizCategory: string
  /** 签到机制轴分类：standard / waf_required / cf_required / special / balance_only / cdk */
  checkinCategory?: string
  aliases?: string[]
  tags?: string[]
  checkin?: CatalogCheckinCapability
  /** 平台 override 块，结构对齐 ProviderTemplate.platforms（投影时按白名单提取） */
  platforms?: CatalogPlatformsBlock
}

export interface CatalogCheckinCapability {
  baseUrl: string
  checkinPath?: string | null
  balancePath: string
  userInfoPath: string
  authHeader: string
  authPrefix: string
  supportsCheckin: boolean
  requiresWafBypass: boolean
  requiresCfClearance: boolean
  checkinBugged: boolean
  wafCookieNames?: string[]
  cdk?: CatalogCdkConfig
  oauth?: CatalogOauthConfig
}

export interface CatalogCdkConfig {
  cdkType: string
  cdkSourceUrl: string
  topupPath?: string | null
  requiresCdkCookies: boolean
  requiresAccessToken: boolean
}

export interface CatalogOauthConfig {
  githubClientId?: string
  linuxdoClientId?: string
  oauthStatePath: string
}

export interface CatalogPlatformsBlock {
  claude?: Record<string, unknown>
  codex?: Record<string, unknown>
  opencode?: Record<string, unknown>
}

// ═══════════════════════════════════════════════════════════
// 解析与校验
// ═══════════════════════════════════════════════════════════

const isRecord = (value: unknown): value is Record<string, unknown> =>
  Boolean(value) && typeof value === 'object' && !Array.isArray(value)

const assertString = (value: unknown, path: string) => {
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`providers-catalog.json 解析失败: ${path} 必须是非空字符串`)
  }
}

const assertBoolean = (value: unknown, path: string) => {
  if (typeof value !== 'boolean') {
    throw new Error(`providers-catalog.json 解析失败: ${path} 必须是布尔值`)
  }
}

/** 逐条目结构校验（与 Rust serde 必填字段对齐，抓住双端漂移） */
const assertCatalogEntry = (entry: unknown, index: number) => {
  const path = `providers[${index}]`
  if (!isRecord(entry)) {
    throw new Error(`providers-catalog.json 解析失败: ${path} 必须是对象`)
  }

  for (const key of ['id', 'name', 'description', 'domain', 'icon', 'bizCategory'] as const) {
    assertString(entry[key], `${path}.${key}`)
  }

  if (entry.checkin !== undefined) {
    const checkin = entry.checkin
    if (!isRecord(checkin)) {
      throw new Error(`providers-catalog.json 解析失败: ${path}.checkin 必须是对象`)
    }
    for (const key of [
      'baseUrl',
      'balancePath',
      'userInfoPath',
      'authHeader',
      'authPrefix',
    ] as const) {
      assertString(checkin[key], `${path}.checkin.${key}`)
    }
    for (const key of [
      'supportsCheckin',
      'requiresWafBypass',
      'requiresCfClearance',
      'checkinBugged',
    ] as const) {
      assertBoolean(checkin[key], `${path}.checkin.${key}`)
    }
  }

  if (entry.platforms !== undefined && !isRecord(entry.platforms)) {
    throw new Error(`providers-catalog.json 解析失败: ${path}.platforms 必须是对象`)
  }
}

/**
 * 解析 catalog 并校验 schemaVersion（失败时抛出显式错误，
 * 行为对齐 Rust 侧 parse_providers_catalog）
 */
export function parseProvidersCatalog(raw: unknown): ProvidersCatalog {
  if (!isRecord(raw)) {
    throw new Error('providers-catalog.json 解析失败: 根节点必须是对象')
  }

  if (raw.schemaVersion !== PROVIDERS_CATALOG_SCHEMA_VERSION) {
    throw new Error(
      `providers-catalog.json schemaVersion 不兼容: 期望 ${PROVIDERS_CATALOG_SCHEMA_VERSION}, 实际 ${String(raw.schemaVersion)}`
    )
  }

  if (!Array.isArray(raw.providers)) {
    throw new Error('providers-catalog.json 解析失败: providers 必须是数组')
  }
  raw.providers.forEach(assertCatalogEntry)

  return raw as unknown as ProvidersCatalog
}

/** 构建期解析好的 catalog 单例（解析失败属于打包错误，模块加载时直接抛出） */
export const PROVIDERS_CATALOG: ProvidersCatalog = parseProvidersCatalog(rawProvidersCatalog)

// ═══════════════════════════════════════════════════════════
// 投影：catalog 条目 → ProviderTemplate
// ═══════════════════════════════════════════════════════════

/** bizCategory（业务轴）→ 模板分类映射；公益/商业中转站归入 third_party */
const BIZ_CATEGORY_TO_TEMPLATE_CATEGORY: Record<string, ProviderTemplateCategory> = {
  official: 'official',
  aggregator: 'aggregator',
  commercial: 'third_party',
  community: 'third_party',
  local: 'local',
}

const pickString = (source: Record<string, unknown>, key: string): string | undefined =>
  typeof source[key] === 'string' && source[key] ? (source[key] as string) : undefined

const pickStringList = (source: Record<string, unknown>, key: string): string[] | undefined => {
  const value = source[key]
  if (!Array.isArray(value)) return undefined
  const items = value.filter((item): item is string => typeof item === 'string' && item.length > 0)
  return items.length > 0 ? items : undefined
}

const compactOverride = <T extends Record<string, unknown>>(override: T): T | undefined => {
  const entries = Object.entries(override).filter(([, value]) => value !== undefined)
  return entries.length > 0 ? (Object.fromEntries(entries) as T) : undefined
}

// 平台 override 白名单投影：只提取模板契约允许的非敏感字段
// （遵守 provider-template-contracts mapper 白名单，绝不透传密钥类字段）
const pickClaudeOverride = (
  source: Record<string, unknown>
): ClaudeProviderTemplateOverride | undefined =>
  compactOverride({
    baseUrl: pickString(source, 'baseUrl'),
    provider: pickString(source, 'provider'),
    providerType: pickString(source, 'providerType'),
    model: pickString(source, 'model'),
    smallFastModel: pickString(source, 'smallFastModel'),
    defaultOpusModel: pickString(source, 'defaultOpusModel'),
    defaultSonnetModel: pickString(source, 'defaultSonnetModel'),
    defaultHaikuModel: pickString(source, 'defaultHaikuModel'),
    subagentModel: pickString(source, 'subagentModel'),
    description: pickString(source, 'description'),
  })

const pickCodexOverride = (
  source: Record<string, unknown>
): CodexProviderTemplateOverride | undefined =>
  compactOverride({
    baseUrl: pickString(source, 'baseUrl'),
    websiteUrl: pickString(source, 'websiteUrl'),
    apiKeyUrl: pickString(source, 'apiKeyUrl'),
    modelCatalog: pickStringList(source, 'modelCatalog'),
    model: pickString(source, 'model'),
    provider: pickString(source, 'provider'),
    providerType: pickString(source, 'providerType'),
    description: pickString(source, 'description'),
    protocol: pickString(source, 'protocol'),
  })

const pickOpenCodeOverride = (
  source: Record<string, unknown>
): OpenCodeProviderTemplateOverride | undefined =>
  compactOverride({
    id: pickString(source, 'id'),
    name: pickString(source, 'name'),
    npm: pickString(source, 'npm'),
    baseURL: pickString(source, 'baseURL'),
  })

const dedupe = (values: Array<string | undefined>) => [
  ...new Set(values.map((value) => value?.trim()).filter(Boolean) as string[]),
]

/**
 * 把 catalog 条目投影为 ProviderTemplate。
 * 仅带 platforms 块的站点参与投影（无 platforms 的特殊签到站返回 null）。
 */
export function catalogEntryToProviderTemplate(
  entry: CatalogProviderEntry
): ProviderTemplate | null {
  if (!entry.platforms) return null

  const claude = entry.platforms.claude ? pickClaudeOverride(entry.platforms.claude) : undefined
  const codex = entry.platforms.codex ? pickCodexOverride(entry.platforms.codex) : undefined
  const opencode = entry.platforms.opencode
    ? pickOpenCodeOverride(entry.platforms.opencode)
    : undefined
  if (!claude && !codex && !opencode) return null

  return {
    id: entry.id,
    name: entry.name,
    aliases: dedupe([entry.domain, ...(entry.aliases ?? [])]),
    category: BIZ_CATEGORY_TO_TEMPLATE_CATEGORY[entry.bizCategory] ?? 'third_party',
    websiteUrl: entry.websiteUrl,
    tags: dedupe([
      ...(entry.tags ?? []),
      entry.bizCategory,
      'checkin',
      claude ? 'claude' : undefined,
      codex ? 'codex' : undefined,
      opencode ? 'opencode' : undefined,
    ]),
    baseUrls: dedupe([claude?.baseUrl, codex?.baseUrl, opencode?.baseURL]),
    source: 'built_in',
    platforms: {
      ...(claude ? { claude } : {}),
      ...(codex ? { codex } : {}),
      ...(opencode ? { opencode } : {}),
    },
  }
}

/** catalog 中带 platforms 块的签到站投影出的模板列表（并入 BUILT_IN_PROVIDER_TEMPLATES） */
export const CHECKIN_CATALOG_PROVIDER_TEMPLATES: ProviderTemplate[] = PROVIDERS_CATALOG.providers
  .map(catalogEntryToProviderTemplate)
  .filter((template): template is ProviderTemplate => template !== null)
