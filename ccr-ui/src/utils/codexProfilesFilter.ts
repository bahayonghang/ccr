// Codex Profiles 过滤/排序/分组（纯函数薄包装）：注入 Codex 平台差异（搜索字段）
// 后委托 utils/profilesFilter 核心。Ref 入参改为普通值（调用方 useMemo 缓存）。
import type { CodexProfile } from '@/types'
import {
  buildProfilesFilter,
  type ProfilesFilterResult,
  type ProfilesSortBy,
  type ProfilesStatusFilter,
} from '@/utils/profilesFilter'

export type CodexProfilesStatusFilter = ProfilesStatusFilter
export type CodexProfilesSortBy = ProfilesSortBy

export interface CodexProfilesFilterInput {
  profiles: CodexProfile[]
  currentProfile: string | null
  query: string
  statusFilter: CodexProfilesStatusFilter
  tagFilter: string | null
  sortBy: CodexProfilesSortBy
}

export type CodexProfilesFilterResult = ProfilesFilterResult<CodexProfile>

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
  return haystacks.some((s) => s.toLowerCase().includes(q))
}

const searchCodexProfiles = (list: CodexProfile[], query: string): CodexProfile[] => {
  const q = query.trim()
  return q ? list.filter((profile) => matchesQuery(profile, q)) : list
}

export function buildCodexProfilesFilter(
  input: CodexProfilesFilterInput,
): CodexProfilesFilterResult {
  return buildProfilesFilter<CodexProfile>({
    ...input,
    search: searchCodexProfiles,
  })
}
