// Claude Profiles 过滤/排序/分组：薄包装，注入 Claude 平台差异（搜索字段、provider 归一化）
// 后委托平台无关核心 useProfilesFilter。公共 API 保持稳定，供视图/工具栏复用。
import type { Ref } from 'vue'
import type { ClaudeProfile } from '@/types'
import {
  filterClaudeProfiles,
  getClaudeProfileProviderKey,
  getClaudeProfileProviderLabel,
} from '@/utils/claudeProfiles'
import {
  useProfilesFilter,
  type ProfilesSortBy,
  type ProfilesStatusFilter,
  type ProviderOption,
  type UseProfilesFilterOutput,
} from './useProfilesFilter'

export type ClaudeProfilesStatusFilter = ProfilesStatusFilter
export type ClaudeProfilesSortBy = ProfilesSortBy
/** provider 选项（getClaudeProfileProviderKey），空 provider 规整为统一 key */
export type ClaudeProviderOption = ProviderOption

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

export type UseClaudeProfilesFilterOutput = UseProfilesFilterOutput<ClaudeProfile>

export function useClaudeProfilesFilter(
  input: UseClaudeProfilesFilterInput,
): UseClaudeProfilesFilterOutput {
  return useProfilesFilter<ClaudeProfile>({
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
      key: profile => getClaudeProfileProviderKey(profile.provider),
      label: (profile, unset) => getClaudeProfileProviderLabel(profile.provider, unset),
      unsetLabel: input.providerUnsetLabel,
    },
  })
}
