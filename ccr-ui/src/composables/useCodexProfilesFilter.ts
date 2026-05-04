// Codex Profiles 过滤/排序/分组逻辑：从视图剥离，便于复用与单元测试
import { computed, type ComputedRef, type Ref } from 'vue'
import type { CodexProfile } from '@/types'

export type CodexProfilesStatusFilter = 'all' | 'active' | 'enabled' | 'disabled'
export type CodexProfilesSortBy = 'recent' | 'name' | 'requests' | 'enabled'

export interface UseCodexProfilesFilterInput {
  profiles: Ref<CodexProfile[]>
  currentProfile: Ref<string | null>
  query: Ref<string>
  statusFilter: Ref<CodexProfilesStatusFilter>
  tagFilter: Ref<string | null>
  sortBy: Ref<CodexProfilesSortBy>
}

export interface UseCodexProfilesFilterOutput {
  /** 全部标签：从当前 profiles 派生（去重 + 字典序） */
  allTags: ComputedRef<string[]>
  /** 应用过滤+排序后的列表 */
  filtered: ComputedRef<CodexProfile[]>
  /** filtered 中已启用部分 */
  enabledList: ComputedRef<CodexProfile[]>
  /** filtered 中已禁用部分 */
  disabledList: ComputedRef<CodexProfile[]>
  /** 当前激活的 profile（null 表示未设置） */
  activeProfile: ComputedRef<CodexProfile | null>
  /** profile 的"已启用"派生值（默认 true） */
  isEnabled: (profile: CodexProfile) => boolean
  /** profile 的"当前激活"派生值 */
  isActive: (profile: CodexProfile, currentName: string | null) => boolean
}

const isProfileEnabled = (profile: CodexProfile): boolean => profile.enabled !== false

const matchesQuery = (profile: CodexProfile, query: string): boolean => {
  if (!query) return true
  const q = query.toLowerCase()
  const haystacks: string[] = [
    profile.name,
    profile.description ?? '',
    profile.base_url ?? '',
    profile.model ?? '',
    profile.provider ?? '',
    profile.account ?? '',
    ...(profile.tags ?? []),
  ]
  return haystacks.some(s => s.toLowerCase().includes(q))
}

const requestsOf = (profile: CodexProfile): number => profile.usage_count ?? 0

export function useCodexProfilesFilter(
  input: UseCodexProfilesFilterInput,
): UseCodexProfilesFilterOutput {
  const { profiles, currentProfile, query, statusFilter, tagFilter, sortBy } = input

  const isActive = (profile: CodexProfile, currentName: string | null): boolean =>
    Boolean(currentName) && profile.name === currentName

  const allTags = computed<string[]>(() => {
    const set = new Set<string>()
    for (const p of profiles.value) {
      for (const t of p.tags ?? []) {
        if (t) set.add(t)
      }
    }
    return Array.from(set).sort()
  })

  const activeProfile = computed<CodexProfile | null>(() => {
    const name = currentProfile.value
    if (!name) return null
    return profiles.value.find(p => p.name === name) ?? null
  })

  const filtered = computed<CodexProfile[]>(() => {
    const q = query.value.trim()
    const status = statusFilter.value
    const tag = tagFilter.value
    const cur = currentProfile.value

    const list = profiles.value.filter(p => {
      const enabled = isProfileEnabled(p)
      const active = isActive(p, cur)
      if (status === 'enabled' && !enabled) return false
      if (status === 'disabled' && enabled) return false
      if (status === 'active' && !active) return false
      if (tag && !(p.tags ?? []).includes(tag)) return false
      if (!matchesQuery(p, q)) return false
      return true
    })

    const sortFn = (() => {
      switch (sortBy.value) {
        case 'name':
          return (a: CodexProfile, b: CodexProfile) => a.name.localeCompare(b.name)
        case 'requests':
          return (a: CodexProfile, b: CodexProfile) => requestsOf(b) - requestsOf(a)
        case 'enabled':
          return (a: CodexProfile, b: CodexProfile) =>
            Number(isProfileEnabled(b)) - Number(isProfileEnabled(a))
        case 'recent':
        default:
          return null
      }
    })()

    return sortFn ? [...list].sort(sortFn) : list
  })

  const enabledList = computed<CodexProfile[]>(() =>
    filtered.value.filter(p => isProfileEnabled(p)),
  )

  const disabledList = computed<CodexProfile[]>(() =>
    filtered.value.filter(p => !isProfileEnabled(p)),
  )

  return {
    allTags,
    filtered,
    enabledList,
    disabledList,
    activeProfile,
    isEnabled: isProfileEnabled,
    isActive,
  }
}
