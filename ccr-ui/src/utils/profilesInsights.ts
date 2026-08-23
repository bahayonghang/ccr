// Profiles 派生洞察的平台无关核心（纯函数）：分布统计（provider/auth/tags）与健康审计
// （弃用 auth / 缺失字段 / 运行时重复），平台差异通过 config 注入。
// 08-22-state-logic-port 批次 5：由 composable 迁为纯变换——Ref 入参变数组，
// ComputedRef 出参变普通值，调用方用 useMemo 缓存。
// Claude / Codex 薄包装见 utils/{claude,codex}ProfilesInsights.ts。
import type { ProfileLike } from './profilesFilter'

export interface ProviderBreakdownItem {
  /** provider 名称，空值规整为 'Unknown' */
  provider: string
  count: number
  /** 整数百分比 0-100 */
  pct: number
}

export interface AuthModeBreakdownItem<Mode extends string> {
  mode: Mode
  count: number
  pct: number
}

export interface TagFrequencyItem {
  tag: string
  count: number
}

export interface MissingFieldIssue<T extends ProfileLike, Field extends string> {
  profile: T
  missing: Field[]
}

export interface DuplicateRuntimeIssue<T extends ProfileLike> {
  /** 'base_url|model' 形式的复合 key，仅供渲染 key 使用 */
  key: string
  profiles: T[]
}

/** 平台注入的洞察策略 */
export interface ProfilesInsightsConfig<
  T extends ProfileLike,
  Mode extends string,
  Field extends string,
> {
  /** 全部 auth 模式（固定顺序，便于稳定布局） */
  authModes: readonly Mode[]
  /** 已弃用 auth 模式集合；不传或为空表示该平台无弃用概念 */
  deprecatedAuthModes?: ReadonlySet<Mode>
  /** 取 profile 的 auth 模式（含默认回退） */
  authModeOf: (profile: T) => Mode
  /** 计算单个 profile 的缺失必填字段（空数组表示无缺失） */
  missingFieldsOf: (profile: T) => Field[]
  /** 运行时去重所用的主模型（空串表示该 profile 不参与去重） */
  primaryRuntimeModel: (profile: T) => string
}

export interface ProfilesInsightsResult<
  T extends ProfileLike,
  Mode extends string,
  Field extends string,
> {
  providerBreakdown: ProviderBreakdownItem[]
  authModeBreakdown: AuthModeBreakdownItem<Mode>[]
  topTags: TagFrequencyItem[]
  /** 使用已弃用 auth 模式的 profile 列表（无弃用概念时恒为空） */
  deprecatedAuthIssues: T[]
  /** 缺少必填字段的 profile 列表（带具体缺失项） */
  missingFieldIssues: MissingFieldIssue<T, Field>[]
  /** 同 base_url + 主模型 组合出现两次及以上的 profile 群组 */
  duplicateRuntimeIssues: DuplicateRuntimeIssue<T>[]
  /** 三类问题的总条数 */
  totalIssueCount: number
}

const UNKNOWN_PROVIDER = 'Unknown'
const PROVIDER_TOP_N = 5
const TAG_TOP_N = 8

const safePercent = (count: number, total: number): number => {
  if (total <= 0) return 0
  return Math.round((count / total) * 100)
}

const profileProvider = <T extends ProfileLike>(profile: T): string => {
  const raw = profile.provider?.trim()
  return raw && raw.length > 0 ? raw : UNKNOWN_PROVIDER
}

const buildProviderBreakdown = <T extends ProfileLike>(
  profiles: T[],
): ProviderBreakdownItem[] => {
  if (profiles.length === 0) return []
  const counts = new Map<string, number>()
  for (const profile of profiles) {
    const key = profileProvider(profile)
    counts.set(key, (counts.get(key) ?? 0) + 1)
  }
  const total = profiles.length
  return Array.from(counts.entries())
    .map(([provider, count]) => ({ provider, count, pct: safePercent(count, total) }))
    .sort((a, b) => b.count - a.count || a.provider.localeCompare(b.provider))
    .slice(0, PROVIDER_TOP_N)
}

const buildAuthModeBreakdown = <T extends ProfileLike, Mode extends string>(
  profiles: T[],
  config: ProfilesInsightsConfig<T, Mode, string>,
): AuthModeBreakdownItem<Mode>[] => {
  const total = profiles.length
  const counts = new Map<Mode, number>(config.authModes.map((mode) => [mode, 0]))
  for (const profile of profiles) {
    const mode = config.authModeOf(profile)
    counts.set(mode, (counts.get(mode) ?? 0) + 1)
  }
  return config.authModes.map((mode) => {
    const count = counts.get(mode) ?? 0
    return { mode, count, pct: safePercent(count, total) }
  })
}

const buildTopTags = <T extends ProfileLike>(profiles: T[]): TagFrequencyItem[] => {
  const counts = new Map<string, number>()
  for (const profile of profiles) {
    for (const tag of (profile.tags ?? []).filter(Boolean)) {
      counts.set(tag, (counts.get(tag) ?? 0) + 1)
    }
  }
  return Array.from(counts.entries())
    .map(([tag, count]) => ({ tag, count }))
    .sort((a, b) => b.count - a.count || a.tag.localeCompare(b.tag))
    .slice(0, TAG_TOP_N)
}

const buildDeprecatedAuthIssues = <T extends ProfileLike, Mode extends string>(
  profiles: T[],
  config: ProfilesInsightsConfig<T, Mode, string>,
): T[] => {
  const deprecated = config.deprecatedAuthModes
  if (!deprecated || deprecated.size === 0) return []
  return profiles
    .filter((profile) => deprecated.has(config.authModeOf(profile)))
    .sort((a, b) => a.name.localeCompare(b.name))
}

const buildMissingFieldIssues = <T extends ProfileLike, Field extends string>(
  profiles: T[],
  config: ProfilesInsightsConfig<T, string, Field>,
): MissingFieldIssue<T, Field>[] => {
  const issues: MissingFieldIssue<T, Field>[] = []
  for (const profile of profiles) {
    const missing = config.missingFieldsOf(profile)
    if (missing.length > 0) issues.push({ profile, missing })
  }
  return issues.sort((a, b) => a.profile.name.localeCompare(b.profile.name))
}

/** 重复运行时（base_url + 主模型 组合），仅当组合非空且出现两次及以上时计入。 */
const buildDuplicateRuntimeIssues = <T extends ProfileLike>(
  profiles: T[],
  primaryRuntimeModel: (profile: T) => string,
): DuplicateRuntimeIssue<T>[] => {
  const buckets = new Map<string, T[]>()
  for (const profile of profiles) {
    const baseUrl = profile.base_url?.trim() ?? ''
    const model = primaryRuntimeModel(profile)
    if (!baseUrl || !model) continue
    const key = `${baseUrl}|${model}`
    const arr = buckets.get(key)
    if (arr) arr.push(profile)
    else buckets.set(key, [profile])
  }
  const result: DuplicateRuntimeIssue<T>[] = []
  for (const [key, group] of buckets.entries()) {
    if (group.length >= 2) {
      result.push({ key, profiles: [...group].sort((a, b) => a.name.localeCompare(b.name)) })
    }
  }
  return result.sort(
    (a, b) => b.profiles.length - a.profiles.length || a.key.localeCompare(b.key),
  )
}

export function buildProfilesInsights<
  T extends ProfileLike,
  Mode extends string,
  Field extends string,
>(
  profiles: T[],
  config: ProfilesInsightsConfig<T, Mode, Field>,
): ProfilesInsightsResult<T, Mode, Field> {
  const deprecatedAuthIssues = buildDeprecatedAuthIssues(profiles, config)
  const missingFieldIssues = buildMissingFieldIssues(profiles, config)
  const duplicateRuntimeIssues = buildDuplicateRuntimeIssues(
    profiles,
    config.primaryRuntimeModel,
  )

  const dupCount = duplicateRuntimeIssues.reduce((acc, group) => acc + group.profiles.length, 0)

  return {
    providerBreakdown: buildProviderBreakdown(profiles),
    authModeBreakdown: buildAuthModeBreakdown(profiles, config),
    topTags: buildTopTags(profiles),
    deprecatedAuthIssues,
    missingFieldIssues,
    duplicateRuntimeIssues,
    totalIssueCount: deprecatedAuthIssues.length + missingFieldIssues.length + dupCount,
  }
}
