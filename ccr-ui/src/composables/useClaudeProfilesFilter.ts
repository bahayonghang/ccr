// Claude Profiles 过滤/排序/分组逻辑：从视图剥离，便于复用与单元测试
import { computed, type ComputedRef, type Ref } from 'vue'
import type { ClaudeProfile } from '@/types'
import {
  filterClaudeProfiles,
  getClaudeProfileProviderKey,
  getClaudeProfileProviderLabel,
} from '@/utils/claudeProfiles'

export type ClaudeProfilesStatusFilter = 'all' | 'active' | 'enabled' | 'disabled'
export type ClaudeProfilesSortBy = 'recent' | 'name' | 'requests' | 'enabled'

export interface ClaudeProviderOption {
  /** provider key（getClaudeProfileProviderKey），空 provider 规整为统一 key */
  key: string
  /** 展示标签（原始 provider 名，未设置时回退 unsetLabel） */
  label: string
}

export interface UseClaudeProfilesFilterInput {
  profiles: Ref<ClaudeProfile[]>
  currentProfile: Ref<string | null>
  query: Ref<string>
  statusFilter: Ref<ClaudeProfilesStatusFilter>
  tagFilter: Ref<string | null>
  /** provider key（getClaudeProfileProviderKey）；null 表示不过滤 */
  providerFilter: Ref<string | null>
  sortBy: Ref<ClaudeProfilesSortBy>
  /** 未设置 provider 的展示标签 */
  providerUnsetLabel: Ref<string>
}

export interface UseClaudeProfilesFilterOutput {
  /** 全部标签：从当前 profiles 派生（去重 + 字典序） */
  allTags: ComputedRef<string[]>
  /** 全部 provider 选项：从当前 profiles 派生（去重 + 字典序），喂筛选下拉 */
  allProviders: ComputedRef<ClaudeProviderOption[]>
  /** 应用过滤+排序后的列表 */
  filtered: ComputedRef<ClaudeProfile[]>
  /** filtered 中已启用部分 */
  enabledList: ComputedRef<ClaudeProfile[]>
  /** filtered 中已禁用部分 */
  disabledList: ComputedRef<ClaudeProfile[]>
  /** 当前激活的 profile（null 表示未设置） */
  activeProfile: ComputedRef<ClaudeProfile | null>
  /** profile 的"已启用"派生值（默认 true） */
  isEnabled: (profile: ClaudeProfile) => boolean
}

const isProfileEnabled = (profile: ClaudeProfile): boolean => profile.enabled !== false

const requestsOf = (profile: ClaudeProfile): number => profile.usage_count ?? 0

export function useClaudeProfilesFilter(
  input: UseClaudeProfilesFilterInput,
): UseClaudeProfilesFilterOutput {
  const {
    profiles,
    currentProfile,
    query,
    statusFilter,
    tagFilter,
    providerFilter,
    sortBy,
    providerUnsetLabel,
  } = input

  const isActive = (profile: ClaudeProfile, currentName: string | null): boolean =>
    Boolean(currentName) && profile.name === currentName

  const allTags = computed<string[]>(() => {
    const set = new Set<string>()
    for (const profile of profiles.value) {
      for (const tag of profile.tags ?? []) {
        if (tag) set.add(tag)
      }
    }
    return Array.from(set).sort()
  })

  const allProviders = computed<ClaudeProviderOption[]>(() => {
    const map = new Map<string, string>()
    for (const profile of profiles.value) {
      const key = getClaudeProfileProviderKey(profile.provider)
      if (!map.has(key)) {
        map.set(key, getClaudeProfileProviderLabel(profile.provider, providerUnsetLabel.value))
      }
    }
    return Array.from(map.entries())
      .map(([key, label]) => ({ key, label }))
      .sort((a, b) => a.label.localeCompare(b.label, undefined, { sensitivity: 'base' }))
  })

  const activeProfile = computed<ClaudeProfile | null>(() => {
    const name = currentProfile.value
    if (!name) return null
    return profiles.value.find(profile => profile.name === name) ?? null
  })

  const filtered = computed<ClaudeProfile[]>(() => {
    const status = statusFilter.value
    const tag = tagFilter.value
    const provider = providerFilter.value
    const current = currentProfile.value

    // 搜索委托现有 util（已覆盖 name/desc/provider/provider_type/account/base_url/model/small_fast_model/tags）
    const matched = filterClaudeProfiles(profiles.value, query.value)

    const list = matched.filter((profile) => {
      const enabled = isProfileEnabled(profile)
      const active = isActive(profile, current)
      if (status === 'enabled' && !enabled) return false
      if (status === 'disabled' && enabled) return false
      if (status === 'active' && !active) return false
      if (tag && !(profile.tags ?? []).includes(tag)) return false
      if (provider && getClaudeProfileProviderKey(profile.provider) !== provider) return false
      return true
    })

    const sortFn = (() => {
      switch (sortBy.value) {
        case 'name':
          return (a: ClaudeProfile, b: ClaudeProfile) => a.name.localeCompare(b.name)
        case 'requests':
          return (a: ClaudeProfile, b: ClaudeProfile) => requestsOf(b) - requestsOf(a)
        case 'enabled':
          return (a: ClaudeProfile, b: ClaudeProfile) =>
            Number(isProfileEnabled(b)) - Number(isProfileEnabled(a))
        case 'recent':
        default:
          return null
      }
    })()

    return sortFn ? [...list].sort(sortFn) : list
  })

  const enabledList = computed<ClaudeProfile[]>(() =>
    filtered.value.filter(profile => isProfileEnabled(profile)),
  )

  const disabledList = computed<ClaudeProfile[]>(() =>
    filtered.value.filter(profile => !isProfileEnabled(profile)),
  )

  return {
    allTags,
    allProviders,
    filtered,
    enabledList,
    disabledList,
    activeProfile,
    isEnabled: isProfileEnabled,
  }
}
