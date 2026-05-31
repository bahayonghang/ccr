// Claude Profiles 派生洞察：分布统计 + 健康审计
// 单一职责：从 profiles 列表派生只读视图数据，便于复用与单元测试，不直接渲染 UI
import { computed, type ComputedRef, type Ref } from 'vue'
import type { ClaudeProfile, ClaudeProfileAuthMode } from '@/types'

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

export interface AuthModeBreakdownItem {
  mode: ClaudeProfileAuthMode
  count: number
  pct: number
}

export interface TagFrequencyItem {
  tag: string
  count: number
}

export type ClaudeMissingField = 'base_url' | 'model' | 'account'

export interface MissingFieldIssue {
  profile: ClaudeProfile
  missing: ClaudeMissingField[]
}

export interface DuplicateRuntimeIssue {
  /** 'base_url|model' 形式的复合 key，仅供 v-for key 使用 */
  key: string
  profiles: ClaudeProfile[]
}

export interface ClaudeProfilesInsights {
  providerBreakdown: ComputedRef<ProviderBreakdownItem[]>
  authModeBreakdown: ComputedRef<AuthModeBreakdownItem[]>
  topTags: ComputedRef<TagFrequencyItem[]>
  /** 缺少必填字段的 profile 列表（带具体缺失项） */
  missingFieldIssues: ComputedRef<MissingFieldIssue[]>
  /** 同 base_url + 主模型 组合出现两次及以上的 profile 群组 */
  duplicateRuntimeIssues: ComputedRef<DuplicateRuntimeIssue[]>
  /** 两类问题的总条数 */
  totalIssueCount: ComputedRef<number>
}

/* ========================================================================
 * 常量
 * ======================================================================== */

const UNKNOWN_PROVIDER = 'Unknown'
const PROVIDER_TOP_N = 5
const TAG_TOP_N = 8

// Claude 仅两种 auth 模式，无废弃项；固定顺序便于稳定布局
const ALL_AUTH_MODES: readonly ClaudeProfileAuthMode[] = ['subscription', 'api_key']

/* ========================================================================
 * 工具
 * ======================================================================== */

const isBlank = (value: string | null | undefined): boolean => !value || value.trim().length === 0

const safePercent = (count: number, total: number): number => {
  if (total <= 0) return 0
  return Math.round((count / total) * 100)
}

const profileAuthMode = (profile: ClaudeProfile): ClaudeProfileAuthMode =>
  profile.auth_mode ?? 'subscription'

const profileProvider = (profile: ClaudeProfile): string => {
  const raw = profile.provider?.trim()
  return raw && raw.length > 0 ? raw : UNKNOWN_PROVIDER
}

/**
 * 是否配置了任意模型。Claude 多模型：主模型与四个映射全空才算缺失。
 */
const hasAnyModel = (profile: ClaudeProfile): boolean =>
  !isBlank(profile.model) ||
  !isBlank(profile.default_opus_model) ||
  !isBlank(profile.default_sonnet_model) ||
  !isBlank(profile.default_haiku_model) ||
  !isBlank(profile.subagent_model)

/**
 * base_url 为空是否需要报告：仅 api_key 模式必须有 base_url；
 * subscription 模式空 base_url 合法（回落本机官方登录）。
 */
const requiresBaseUrl = (profile: ClaudeProfile): boolean => profileAuthMode(profile) === 'api_key'

/** 运行时去重所用的主模型：优先 model，回退 sonnet/opus 映射。 */
const primaryRuntimeModel = (profile: ClaudeProfile): string =>
  profile.model?.trim() ||
  profile.default_sonnet_model?.trim() ||
  profile.default_opus_model?.trim() ||
  ''

/* ========================================================================
 * 主入口
 * ======================================================================== */

export function useClaudeProfilesInsights(profiles: Ref<ClaudeProfile[]>): ClaudeProfilesInsights {
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
   * 分布：Auth 模式 2 桶（保持固定顺序）
   * ---------------------------------------------------------------- */
  const authModeBreakdown = computed<AuthModeBreakdownItem[]>(() => {
    const list = profiles.value
    const total = list.length

    const counts = new Map<ClaudeProfileAuthMode, number>(ALL_AUTH_MODES.map((mode) => [mode, 0]))

    for (const profile of list) {
      const mode = profileAuthMode(profile)
      counts.set(mode, (counts.get(mode) ?? 0) + 1)
    }

    return ALL_AUTH_MODES.map((mode) => {
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
   * 健康审计：缺失必填字段
   * ---------------------------------------------------------------- */
  const missingFieldIssues = computed<MissingFieldIssue[]>(() => {
    const issues: MissingFieldIssue[] = []
    for (const profile of profiles.value) {
      const missing: ClaudeMissingField[] = []
      if (requiresBaseUrl(profile) && isBlank(profile.base_url)) missing.push('base_url')
      if (!hasAnyModel(profile)) missing.push('model')
      if (isBlank(profile.account)) missing.push('account')
      if (missing.length > 0) issues.push({ profile, missing })
    }
    return issues.sort((a, b) => a.profile.name.localeCompare(b.profile.name))
  })

  /* ----------------------------------------------------------------
   * 健康审计：重复运行时（base_url + 主模型 组合）
   * 仅当组合非空且出现两次及以上时计入
   * ---------------------------------------------------------------- */
  const duplicateRuntimeIssues = computed<DuplicateRuntimeIssue[]>(() => {
    const buckets = new Map<string, ClaudeProfile[]>()
    for (const profile of profiles.value) {
      const baseUrl = profile.base_url?.trim() ?? ''
      const model = primaryRuntimeModel(profile)
      if (!baseUrl || !model) continue
      const key = `${baseUrl}|${model}`
      const arr = buckets.get(key)
      if (arr) arr.push(profile)
      else buckets.set(key, [profile])
    }
    const result: DuplicateRuntimeIssue[] = []
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
   * 问题总数
   * ---------------------------------------------------------------- */
  const totalIssueCount = computed<number>(() => {
    const dupCount = duplicateRuntimeIssues.value.reduce(
      (acc, group) => acc + group.profiles.length,
      0
    )
    return missingFieldIssues.value.length + dupCount
  })

  return {
    providerBreakdown,
    authModeBreakdown,
    topTags,
    missingFieldIssues,
    duplicateRuntimeIssues,
    totalIssueCount,
  }
}
