// Codex Profiles 过滤/排序/分组：薄包装，注入 Codex 平台差异（搜索字段）
// 后委托平台无关核心 useProfilesFilter。公共 API 保持稳定，供视图/工具栏复用。
import type { Ref } from 'vue'
import type { CodexProfile } from '@/types'
import {
  useProfilesFilter,
  type ProfilesSortBy,
  type ProfilesStatusFilter,
  type UseProfilesFilterOutput,
} from './useProfilesFilter'

export type CodexProfilesStatusFilter = ProfilesStatusFilter
export type CodexProfilesSortBy = ProfilesSortBy

export interface UseCodexProfilesFilterInput {
  profiles: Ref<CodexProfile[]>
  currentProfile: Ref<string | null>
  query: Ref<string>
  statusFilter: Ref<CodexProfilesStatusFilter>
  tagFilter: Ref<string | null>
  sortBy: Ref<CodexProfilesSortBy>
}

export type UseCodexProfilesFilterOutput = UseProfilesFilterOutput<CodexProfile>

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

const searchCodexProfiles = (list: CodexProfile[], query: string): CodexProfile[] => {
  const q = query.trim()
  return q ? list.filter(profile => matchesQuery(profile, q)) : list
}

export function useCodexProfilesFilter(
  input: UseCodexProfilesFilterInput,
): UseCodexProfilesFilterOutput {
  return useProfilesFilter<CodexProfile>({
    profiles: input.profiles,
    currentProfile: input.currentProfile,
    query: input.query,
    statusFilter: input.statusFilter,
    tagFilter: input.tagFilter,
    sortBy: input.sortBy,
    search: searchCodexProfiles,
  })
}
