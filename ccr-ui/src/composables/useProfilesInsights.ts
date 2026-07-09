// Profiles 派生洞察的平台无关核心：分布统计（provider/auth/tags）与健康审计
// （弃用 auth / 缺失字段 / 运行时重复）两平台算法一致，平台差异通过 config 注入。
// 单一职责：从 profiles 列表派生只读视图数据，便于复用与单元测试，不直接渲染 UI。
// Claude / Codex 各自的薄包装见 useClaudeProfilesInsights / useCodexProfilesInsights。
import { computed, type ComputedRef, type Ref } from 'vue'
import type { ProfileLike } from './useProfilesFilter'

/* ========================================================================
 * 类型定义
 * ======================================================================== */

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
  /** 'base_url|model' 形式的复合 key，仅供 v-for key 使用 */
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

export interface ProfilesInsights<
  T extends ProfileLike,
  Mode extends string,
  Field extends string,
> {
  providerBreakdown: ComputedRef<ProviderBreakdownItem[]>
  authModeBreakdown: ComputedRef<AuthModeBreakdownItem<Mode>[]>
  topTags: ComputedRef<TagFrequencyItem[]>
  /** 使用已弃用 auth 模式的 profile 列表（无弃用概念时恒为空） */
  deprecatedAuthIssues: ComputedRef<T[]>
  /** 缺少必填字段的 profile 列表（带具体缺失项） */
  missingFieldIssues: ComputedRef<MissingFieldIssue<T, Field>[]>
  /** 同 base_url + 主模型 组合出现两次及以上的 profile 群组 */
  duplicateRuntimeIssues: ComputedRef<DuplicateRuntimeIssue<T>[]>
  /** 三类问题的总条数 */
  totalIssueCount: ComputedRef<number>
}

/* ========================================================================
 * 常量
 * ======================================================================== */

const UNKNOWN_PROVIDER = 'Unknown'
const PROVIDER_TOP_N = 5
const TAG_TOP_N = 8

/* ========================================================================
 * 工具
 * ======================================================================== */

const safePercent = (count: number, total: number): number => {
  if (total <= 0) return 0
  return Math.round((count / total) * 100)
}

const profileProvider = <T extends ProfileLike>(profile: T): string => {
  const raw = profile.provider?.trim()
  return raw && raw.length > 0 ? raw : UNKNOWN_PROVIDER
}

/* ========================================================================
 * 主入口
 * ======================================================================== */

export function useProfilesInsights<
  T extends ProfileLike,
  Mode extends string,
  Field extends string,
>(
  profiles: Ref<T[]>,
  config: ProfilesInsightsConfig<T, Mode, Field>
): ProfilesInsights<T, Mode, Field> {
  /* ----------------------------------------------------------------
   * 分布：Provider 前 N
   * ---------------------------------------------------------------- */
  const providerBreakdown = computed<ProviderBreakdownItem[]>(() => {
    const list = profiles.value
    if (list.length === 0) return []

    const counts = new Map<string, number>()
    for (const profile of list) {
      const key = profileProvider(profile)
      counts.set(key, (counts.get(key) ?? 0) + 1)
    }

    const total = list.length
    return Array.from(counts.entries())
      .map(([provider, count]) => ({ provider, count, pct: safePercent(count, total) }))
      .sort((a, b) => b.count - a.count || a.provider.localeCompare(b.provider))
      .slice(0, PROVIDER_TOP_N)
  })

  /* ----------------------------------------------------------------
   * 分布：Auth 模式（按 config.authModes 固定顺序，便于稳定布局）
   * ---------------------------------------------------------------- */
  const authModeBreakdown = computed<AuthModeBreakdownItem<Mode>[]>(() => {
    const list = profiles.value
    const total = list.length

    const counts = new Map<Mode, number>(config.authModes.map((mode) => [mode, 0]))

    for (const profile of list) {
      const mode = config.authModeOf(profile)
      counts.set(mode, (counts.get(mode) ?? 0) + 1)
    }

    return config.authModes.map((mode) => {
      const count = counts.get(mode) ?? 0
      return { mode, count, pct: safePercent(count, total) }
    })
  })

  /* ----------------------------------------------------------------
   * 分布：Top Tags
   * ---------------------------------------------------------------- */
  const topTags = computed<TagFrequencyItem[]>(() => {
    const counts = new Map<string, number>()
    for (const profile of profiles.value) {
      for (const tag of profile.tags ?? []) {
        if (!tag) continue
        counts.set(tag, (counts.get(tag) ?? 0) + 1)
      }
    }
    return Array.from(counts.entries())
      .map(([tag, count]) => ({ tag, count }))
      .sort((a, b) => b.count - a.count || a.tag.localeCompare(b.tag))
      .slice(0, TAG_TOP_N)
  })

  /* ----------------------------------------------------------------
   * 健康审计：已弃用 auth 模式（无弃用概念的平台恒为空）
   * ---------------------------------------------------------------- */
  const deprecatedAuthIssues = computed<T[]>(() => {
    const deprecated = config.deprecatedAuthModes
    if (!deprecated || deprecated.size === 0) return []
    return profiles.value
      .filter((profile) => deprecated.has(config.authModeOf(profile)))
      .sort((a, b) => a.name.localeCompare(b.name))
  })

  /* ----------------------------------------------------------------
   * 健康审计：缺失必填字段
   * ---------------------------------------------------------------- */
  const missingFieldIssues = computed<MissingFieldIssue<T, Field>[]>(() => {
    const issues: MissingFieldIssue<T, Field>[] = []
    for (const profile of profiles.value) {
      const missing = config.missingFieldsOf(profile)
      if (missing.length > 0) issues.push({ profile, missing })
    }
    return issues.sort((a, b) => a.profile.name.localeCompare(b.profile.name))
  })

  /* ----------------------------------------------------------------
   * 健康审计：重复运行时（base_url + 主模型 组合）
   * 仅当组合非空且出现两次及以上时计入
   * ---------------------------------------------------------------- */
  const duplicateRuntimeIssues = computed<DuplicateRuntimeIssue<T>[]>(() => {
    const buckets = new Map<string, T[]>()
    for (const profile of profiles.value) {
      const baseUrl = profile.base_url?.trim() ?? ''
      const model = config.primaryRuntimeModel(profile)
      if (!baseUrl || !model) continue
      const key = `${baseUrl}|${model}`
      const arr = buckets.get(key)
      if (arr) arr.push(profile)
      else buckets.set(key, [profile])
    }
    const result: DuplicateRuntimeIssue<T>[] = []
    for (const [key, group] of buckets.entries()) {
      if (group.length >= 2) {
        result.push({
          key,
          profiles: [...group].sort((a, b) => a.name.localeCompare(b.name)),
        })
      }
    }
    return result.sort(
      (a, b) => b.profiles.length - a.profiles.length || a.key.localeCompare(b.key)
    )
  })

  /* ----------------------------------------------------------------
   * 问题总数（弃用 + 缺失 + 重复展开计数）
   * ---------------------------------------------------------------- */
  const totalIssueCount = computed<number>(() => {
    const dupCount = duplicateRuntimeIssues.value.reduce(
      (acc, group) => acc + group.profiles.length,
      0
    )
    return deprecatedAuthIssues.value.length + missingFieldIssues.value.length + dupCount
  })

  return {
    providerBreakdown,
    authModeBreakdown,
    topTags,
    deprecatedAuthIssues,
    missingFieldIssues,
    duplicateRuntimeIssues,
    totalIssueCount,
  }
}
