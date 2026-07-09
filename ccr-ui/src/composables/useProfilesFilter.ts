// Profiles 过滤/排序/分组的平台无关核心：状态/标签/排序逻辑两平台完全一致，
// 平台差异（搜索字段、provider 归一化）通过注入策略表达。
// Claude / Codex 各自的薄包装见 useClaudeProfilesFilter / useCodexProfilesFilter。
import { computed, type ComputedRef, type Ref } from 'vue'

export type ProfilesStatusFilter = 'all' | 'active' | 'enabled' | 'disabled'
export type ProfilesSortBy = 'recent' | 'name' | 'requests' | 'enabled'

/** 两平台 Profile 的最小公共形状，供泛型过滤/洞察消费 */
export interface ProfileLike {
  name: string
  enabled?: boolean | null
  tags?: string[] | null
  usage_count?: number | null
  provider?: string | null
  /** 运行时去重审计（base_url + 主模型）所需 */
  base_url?: string | null
}

export interface ProviderOption {
  /** provider key（归一化后），空 provider 规整为统一 key */
  key: string
  /** 展示标签（原始 provider 名，未设置时回退 unsetLabel） */
  label: string
}

/** provider 归一化策略：提供则启用 provider 维度过滤并派生 allProviders 选项 */
export interface ProviderStrategy<T extends ProfileLike> {
  /** provider key（用于过滤匹配 + 去重） */
  key: (profile: T) => string
  /** 展示标签（unset 时回退 unsetLabel） */
  label: (profile: T, unsetLabel: string) => string
  /** 未设置 provider 的展示标签 */
  unsetLabel: Ref<string>
}

export interface UseProfilesFilterInput<T extends ProfileLike> {
  profiles: Ref<T[]>
  currentProfile: Ref<string | null>
  query: Ref<string>
  statusFilter: Ref<ProfilesStatusFilter>
  tagFilter: Ref<string | null>
  sortBy: Ref<ProfilesSortBy>
  /** 搜索过滤（平台决定参与匹配的字段集合）；query 为空时应原样返回 list */
  search: (list: T[], query: string) => T[]
  /** provider 过滤当前值（provider key）；与 provider 策略成对出现，null 表示不过滤 */
  providerFilter?: Ref<string | null>
  /** provider 归一化策略；提供则派生 allProviders 并启用 provider 过滤 */
  provider?: ProviderStrategy<T>
}

export interface UseProfilesFilterOutput<T extends ProfileLike> {
  /** 全部标签：从当前 profiles 派生（去重 + 字典序） */
  allTags: ComputedRef<string[]>
  /** 全部 provider 选项：未配置 provider 策略时为空数组 */
  allProviders: ComputedRef<ProviderOption[]>
  /** 应用过滤+排序后的列表 */
  filtered: ComputedRef<T[]>
  /** filtered 中已启用部分 */
  enabledList: ComputedRef<T[]>
  /** filtered 中已禁用部分 */
  disabledList: ComputedRef<T[]>
  /** 当前激活的 profile（null 表示未设置） */
  activeProfile: ComputedRef<T | null>
  /** profile 的"已启用"派生值（默认 true） */
  isEnabled: (profile: T) => boolean
  /** profile 的"当前激活"派生值 */
  isActive: (profile: T, currentName: string | null) => boolean
}

const isProfileEnabled = <T extends ProfileLike>(profile: T): boolean => profile.enabled !== false

const requestsOf = <T extends ProfileLike>(profile: T): number => profile.usage_count ?? 0

export function useProfilesFilter<T extends ProfileLike>(
  input: UseProfilesFilterInput<T>,
): UseProfilesFilterOutput<T> {
  const {
    profiles,
    currentProfile,
    query,
    statusFilter,
    tagFilter,
    sortBy,
    search,
    providerFilter,
    provider,
  } = input

  const isActive = (profile: T, currentName: string | null): boolean =>
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

  const allProviders = computed<ProviderOption[]>(() => {
    if (!provider) return []
    const map = new Map<string, string>()
    for (const profile of profiles.value) {
      const key = provider.key(profile)
      if (!map.has(key)) {
        map.set(key, provider.label(profile, provider.unsetLabel.value))
      }
    }
    return Array.from(map.entries())
      .map(([key, label]) => ({ key, label }))
      .sort((a, b) => a.label.localeCompare(b.label, undefined, { sensitivity: 'base' }))
  })

  const activeProfile = computed<T | null>(() => {
    const name = currentProfile.value
    if (!name) return null
    return profiles.value.find(profile => profile.name === name) ?? null
  })

  const filtered = computed<T[]>(() => {
    const status = statusFilter.value
    const tag = tagFilter.value
    const providerKey = providerFilter?.value ?? null
    const current = currentProfile.value

    // 搜索委托平台注入策略，query 为空时原样返回
    const matched = search(profiles.value, query.value)

    const list = matched.filter((profile) => {
      const enabled = isProfileEnabled(profile)
      const active = isActive(profile, current)
      if (status === 'enabled' && !enabled) return false
      if (status === 'disabled' && enabled) return false
      if (status === 'active' && !active) return false
      if (tag && !(profile.tags ?? []).includes(tag)) return false
      if (providerKey && provider && provider.key(profile) !== providerKey) return false
      return true
    })

    const sortFn = (() => {
      switch (sortBy.value) {
        case 'name':
          return (a: T, b: T) => a.name.localeCompare(b.name)
        case 'requests':
          return (a: T, b: T) => requestsOf(b) - requestsOf(a)
        case 'enabled':
          return (a: T, b: T) => Number(isProfileEnabled(b)) - Number(isProfileEnabled(a))
        case 'recent':
        default:
          return null
      }
    })()

    return sortFn ? [...list].sort(sortFn) : list
  })

  // 当前 profile 固定置顶：不改变其余项的相对顺序，仅将命中项前移。
  const pinCurrent = (list: T[]): T[] => {
    const current = currentProfile.value
    if (!current) return list
    const idx = list.findIndex(profile => profile.name === current)
    if (idx <= 0) return list
    return [list[idx], ...list.slice(0, idx), ...list.slice(idx + 1)]
  }

  const enabledList = computed<T[]>(() =>
    pinCurrent(filtered.value.filter(profile => isProfileEnabled(profile))),
  )

  const disabledList = computed<T[]>(() =>
    pinCurrent(filtered.value.filter(profile => !isProfileEnabled(profile))),
  )

  return {
    allTags,
    allProviders,
    filtered,
    enabledList,
    disabledList,
    activeProfile,
    isEnabled: isProfileEnabled,
    isActive,
  }
}
