// Profiles 过滤/排序/分组的平台无关核心（纯函数）。
// 状态/标签/排序逻辑各平台完全一致，平台差异（搜索字段、provider 归一化）通过注入策略表达。
// 08-22-state-logic-port 批次 5：由 composable（ref/computed 包装）迁为纯变换——
// Ref 入参变普通值，ComputedRef 出参变普通值，调用方在组件内用 useMemo 缓存。
// Claude / Codex / Grok 的薄包装见 utils/{claude,codex,grok}ProfilesFilter.ts。

export type ProfilesStatusFilter = 'all' | 'active' | 'enabled' | 'disabled'
export type ProfilesSortBy = 'recent' | 'name' | 'requests' | 'enabled'

/** 各平台 Profile 的最小公共形状，供泛型过滤/洞察消费 */
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
  unsetLabel: string
}

export interface ProfilesFilterInput<T extends ProfileLike> {
  profiles: T[]
  currentProfile: string | null
  query: string
  statusFilter: ProfilesStatusFilter
  tagFilter: string | null
  sortBy: ProfilesSortBy
  /** 搜索过滤（平台决定参与匹配的字段集合）；query 为空时应原样返回 list */
  search: (list: T[], query: string) => T[]
  /** provider 过滤当前值（provider key）；与 provider 策略成对出现，null 表示不过滤 */
  providerFilter?: string | null
  /** provider 归一化策略；提供则派生 allProviders 并启用 provider 过滤 */
  provider?: ProviderStrategy<T>
}

export interface ProfilesFilterResult<T extends ProfileLike> {
  /** 全部标签：从当前 profiles 派生（去重 + 字典序） */
  allTags: string[]
  /** 全部 provider 选项：未配置 provider 策略时为空数组 */
  allProviders: ProviderOption[]
  /** 应用过滤+排序后的列表 */
  filtered: T[]
  /** filtered 中已启用部分 */
  enabledList: T[]
  /** filtered 中已禁用部分 */
  disabledList: T[]
  /** 当前激活的 profile（null 表示未设置） */
  activeProfile: T | null
  /** profile 的"已启用"派生值（默认 true） */
  isEnabled: (profile: T) => boolean
  /** profile 的"当前激活"派生值 */
  isActive: (profile: T, currentName: string | null) => boolean
}

const isProfileEnabled = <T extends ProfileLike>(profile: T): boolean => profile.enabled !== false

const requestsOf = <T extends ProfileLike>(profile: T): number => profile.usage_count ?? 0

export const isProfileActive = (profile: ProfileLike, currentName: string | null): boolean =>
  Boolean(currentName) && profile.name === currentName

const collectAllTags = <T extends ProfileLike>(profiles: T[]): string[] => {
  const set = new Set<string>()
  for (const profile of profiles) {
    for (const tag of (profile.tags ?? []).filter(Boolean)) {
      set.add(tag)
    }
  }
  return Array.from(set).sort()
}

const collectAllProviders = <T extends ProfileLike>(
  profiles: T[],
  provider: ProviderStrategy<T> | undefined,
): ProviderOption[] => {
  if (!provider) return []
  const map = new Map<string, string>()
  for (const profile of profiles) {
    const key = provider.key(profile)
    if (!map.has(key)) {
      map.set(key, provider.label(profile, provider.unsetLabel))
    }
  }
  return Array.from(map.entries())
    .map(([key, label]) => ({ key, label }))
    .sort((a, b) => a.label.localeCompare(b.label, undefined, { sensitivity: 'base' }))
}

const sortProfiles = <T extends ProfileLike>(list: T[], sortBy: ProfilesSortBy): T[] => {
  switch (sortBy) {
    case 'name':
      return [...list].sort((a, b) => a.name.localeCompare(b.name))
    case 'requests':
      return [...list].sort((a, b) => requestsOf(b) - requestsOf(a))
    case 'enabled':
      return [...list].sort(
        (a, b) => Number(isProfileEnabled(b)) - Number(isProfileEnabled(a)),
      )
    case 'recent':
    default:
      return list
  }
}

const filterProfiles = <T extends ProfileLike>(
  input: ProfilesFilterInput<T>,
  isActive: (profile: T, currentName: string | null) => boolean,
): T[] => {
  const { profiles, currentProfile, query, statusFilter, tagFilter, search, providerFilter, provider } = input
  // 搜索委托平台注入策略，query 为空时原样返回
  const matched = search(profiles, query)

  const providerKey = providerFilter ?? null
  const list = matched.filter((profile) => {
    const enabled = isProfileEnabled(profile)
    const active = isActive(profile, currentProfile)
    if (statusFilter === 'enabled' && !enabled) return false
    if (statusFilter === 'disabled' && enabled) return false
    if (statusFilter === 'active' && !active) return false
    if (tagFilter && !(profile.tags ?? []).includes(tagFilter)) return false
    if (providerKey && provider && provider.key(profile) !== providerKey) return false
    return true
  })

  return sortProfiles(list, input.sortBy)
}

/** 当前 profile 固定置顶：不改变其余项的相对顺序，仅将命中项前移。 */
const pinCurrent = <T extends ProfileLike>(list: T[], current: string | null): T[] => {
  if (!current) return list
  const idx = list.findIndex((profile) => profile.name === current)
  if (idx <= 0) return list
  return [list[idx], ...list.slice(0, idx), ...list.slice(idx + 1)]
}

export function buildProfilesFilter<T extends ProfileLike>(
  input: ProfilesFilterInput<T>,
): ProfilesFilterResult<T> {
  const { profiles, currentProfile } = input
  const filtered = filterProfiles(input, isProfileActive)

  return {
    allTags: collectAllTags(profiles),
    allProviders: collectAllProviders(profiles, input.provider),
    filtered,
    enabledList: pinCurrent(filtered.filter(isProfileEnabled), currentProfile),
    disabledList: pinCurrent(
      filtered.filter((profile) => !isProfileEnabled(profile)),
      currentProfile,
    ),
    activeProfile: currentProfile
      ? (profiles.find((profile) => profile.name === currentProfile) ?? null)
      : null,
    isEnabled: isProfileEnabled,
    isActive: isProfileActive,
  }
}
