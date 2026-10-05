// Claude Profiles 过滤/排序/分组（纯函数薄包装）：注入 Claude 平台差异（搜索字段、
// provider 归一化）后委托 utils/profilesFilter 核心。公共 API 语义与原 composable 一致，
// Ref 入参改为普通值（调用方 useMemo 缓存）。
import type { ClaudeProfile } from '@/types'
import {
  filterClaudeProfiles,
  getClaudeProfileProviderKey,
  getClaudeProfileProviderLabel,
} from '@/utils/claudeProfiles'
import {
  buildProfilesFilter,
  type ProfilesFilterResult,
  type ProfilesSortBy,
  type ProfilesStatusFilter,
  type ProviderOption,
} from '@/utils/profilesFilter'

export type ClaudeProfilesStatusFilter = ProfilesStatusFilter
export type ClaudeProfilesSortBy = ProfilesSortBy
/** provider 选项（getClaudeProfileProviderKey），空 provider 规整为统一 key */
export type ClaudeProviderOption = ProviderOption

export interface ClaudeProfilesFilterInput {
  profiles: ClaudeProfile[]
  currentProfile: string | null
  query: string
  statusFilter: ClaudeProfilesStatusFilter
  tagFilter: string | null
  /** provider key（getClaudeProfileProviderKey）；null 表示不过滤 */
  providerFilter: string | null
  sortBy: ClaudeProfilesSortBy
  /** 未设置 provider 的展示标签 */
  providerUnsetLabel: string
}

export type ClaudeProfilesFilterResult = ProfilesFilterResult<ClaudeProfile>

export function buildClaudeProfilesFilter(
  input: ClaudeProfilesFilterInput,
): ClaudeProfilesFilterResult {
  return buildProfilesFilter<ClaudeProfile>({
    profiles: input.profiles,
    currentProfile: input.currentProfile,
    query: input.query,
    statusFilter: input.statusFilter,
    tagFilter: input.tagFilter,
    sortBy: input.sortBy,
    // 搜索覆盖 name/desc/provider/provider_type/account/base_url/model/small_fast_model/tags
    search: filterClaudeProfiles,
    providerFilter: input.providerFilter,
    provider: {
      key: (profile) => getClaudeProfileProviderKey(profile.provider),
      label: (profile, unset) => getClaudeProfileProviderLabel(profile.provider, unset),
      unsetLabel: input.providerUnsetLabel,
    },
  })
}
