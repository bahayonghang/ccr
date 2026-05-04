// Codex Profiles 派生洞察：分布统计 + 健康审计
// 单一职责：从 profiles 列表派生只读视图数据，便于复用与单元测试，不直接渲染 UI
import { computed, type ComputedRef, type Ref } from 'vue'
import type { CodexProfile, CodexProfileAuthMode } from '@/types'

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
  mode: CodexProfileAuthMode
  count: number
  pct: number
}

export interface TagFrequencyItem {
  tag: string
  count: number
}

export type MissingField = 'base_url' | 'model'

export interface MissingFieldIssue {
  profile: CodexProfile
  missing: MissingField[]
}

export interface DuplicateRuntimeIssue {
  /** 'base_url|model' 形式的复合 key，仅供 v-for key 使用 */
  key: string
  profiles: CodexProfile[]
}

export interface CodexProfilesInsights {
  providerBreakdown: ComputedRef<ProviderBreakdownItem[]>
  authModeBreakdown: ComputedRef<AuthModeBreakdownItem[]>
  topTags: ComputedRef<TagFrequencyItem[]>
  /** 使用已弃用 auth 模式的 profile 列表 */
  deprecatedAuthIssues: ComputedRef<CodexProfile[]>
  /** 缺少必填字段的 profile 列表（带具体缺失项） */
  missingFieldIssues: ComputedRef<MissingFieldIssue[]>
  /** 同 base_url + model 组合出现两次及以上的 profile 群组 */
  duplicateRuntimeIssues: ComputedRef<DuplicateRuntimeIssue[]>
  /** 三类问题的总条数 */
  totalIssueCount: ComputedRef<number>
}

/* ========================================================================
 * 常量
 * ======================================================================== */

const UNKNOWN_PROVIDER = 'Unknown'
const PROVIDER_TOP_N = 5
const TAG_TOP_N = 8

const ALL_AUTH_MODES: readonly CodexProfileAuthMode[] = [
  'openai_chatgpt',
  'openai_api_key',
  'provider_env_key',
  'no_auth',
]

const DEPRECATED_AUTH_MODES = new Set<CodexProfileAuthMode>([
  'openai_chatgpt',
  'provider_env_key',
])

/* ========================================================================
 * 工具
 * ======================================================================== */

const isBlank = (value: string | null | undefined): boolean =>
  !value || value.trim().length === 0

const safePercent = (count: number, total: number): number => {
  if (total <= 0) return 0
  return Math.round((count / total) * 100)
}

const profileAuthMode = (profile: CodexProfile): CodexProfileAuthMode =>
  profile.auth_mode ?? 'no_auth'

const profileProvider = (profile: CodexProfile): string => {
  const raw = profile.provider?.trim()
  return raw && raw.length > 0 ? raw : UNKNOWN_PROVIDER
}

/**
 * base_url 为空的 profile 是否仍然合规：
 * 仅 openai_chatgpt 模式允许空 base_url（官方 ChatGPT 登录运行时）。
 */
const requiresBaseUrl = (profile: CodexProfile): boolean =>
  profileAuthMode(profile) !== 'openai_chatgpt'

/* ========================================================================
 * 主入口
 * ======================================================================== */

export function useCodexProfilesInsights(
  profiles: Ref<CodexProfile[]>,
): CodexProfilesInsights {
  /* ----------------------------------------------------------------
   * 分布：Provider 前 N
   * ---------------------------------------------------------------- */
  const providerBreakdown = computed<ProviderBreakdownItem[]>(() => {
    const list = profiles.value
    if (list.length === 0) return []

    // 1.1 按 provider 计数
    const counts = new Map<string, number>()
    for (const p of list) {
      const key = profileProvider(p)
      counts.set(key, (counts.get(key) ?? 0) + 1)
    }

    // 1.2 排序：count desc → provider asc
    const total = list.length
    return Array.from(counts.entries())
      .map(([provider, count]) => ({
        provider,
        count,
        pct: safePercent(count, total),
      }))
      .sort((a, b) => b.count - a.count || a.provider.localeCompare(b.provider))
      .slice(0, PROVIDER_TOP_N)
  })

  /* ----------------------------------------------------------------
   * 分布：Auth 模式 4 桶（保持固定顺序，便于稳定布局）
   * ---------------------------------------------------------------- */
  const authModeBreakdown = computed<AuthModeBreakdownItem[]>(() => {
    const list = profiles.value
    const total = list.length

    // 2.1 初始化所有桶为 0
    const counts = new Map<CodexProfileAuthMode, number>(
      ALL_AUTH_MODES.map(mode => [mode, 0]),
    )

    // 2.2 累加
    for (const p of list) {
      const mode = profileAuthMode(p)
      counts.set(mode, (counts.get(mode) ?? 0) + 1)
    }

    // 2.3 按 ALL_AUTH_MODES 顺序输出
    return ALL_AUTH_MODES.map(mode => {
      const count = counts.get(mode) ?? 0
      return { mode, count, pct: safePercent(count, total) }
    })
  })

  /* ----------------------------------------------------------------
   * 分布：Top Tags
   * ---------------------------------------------------------------- */
  const topTags = computed<TagFrequencyItem[]>(() => {
    const counts = new Map<string, number>()
    for (const p of profiles.value) {
      for (const tag of p.tags ?? []) {
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
   * 健康审计：已弃用 auth 模式
   * ---------------------------------------------------------------- */
  const deprecatedAuthIssues = computed<CodexProfile[]>(() =>
    profiles.value
      .filter(p => DEPRECATED_AUTH_MODES.has(profileAuthMode(p)))
      .sort((a, b) => a.name.localeCompare(b.name)),
  )

  /* ----------------------------------------------------------------
   * 健康审计：缺失必填字段
   * ---------------------------------------------------------------- */
  const missingFieldIssues = computed<MissingFieldIssue[]>(() => {
    const issues: MissingFieldIssue[] = []
    for (const p of profiles.value) {
      const missing: MissingField[] = []
      if (requiresBaseUrl(p) && isBlank(p.base_url)) missing.push('base_url')
      if (isBlank(p.model)) missing.push('model')
      if (missing.length > 0) issues.push({ profile: p, missing })
    }
    return issues.sort((a, b) => a.profile.name.localeCompare(b.profile.name))
  })

  /* ----------------------------------------------------------------
   * 健康审计：重复运行时（base_url + model 组合）
   * 仅当组合非空且出现两次及以上时计入
   * ---------------------------------------------------------------- */
  const duplicateRuntimeIssues = computed<DuplicateRuntimeIssue[]>(() => {
    const buckets = new Map<string, CodexProfile[]>()
    for (const p of profiles.value) {
      const baseUrl = p.base_url?.trim() ?? ''
      const model = p.model?.trim() ?? ''
      if (!baseUrl || !model) continue
      const key = `${baseUrl}|${model}`
      const arr = buckets.get(key)
      if (arr) arr.push(p)
      else buckets.set(key, [p])
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
      (a, b) =>
        b.profiles.length - a.profiles.length || a.key.localeCompare(b.key),
    )
  })

  /* ----------------------------------------------------------------
   * 三类问题总数
   * ---------------------------------------------------------------- */
  const totalIssueCount = computed<number>(() => {
    const dupCount = duplicateRuntimeIssues.value.reduce(
      (acc, group) => acc + group.profiles.length,
      0,
    )
    return (
      deprecatedAuthIssues.value.length
      + missingFieldIssues.value.length
      + dupCount
    )
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
