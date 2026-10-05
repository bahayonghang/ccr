// Grok Profiles 过滤/排序/分组（纯函数薄包装）：注入 Grok 平台差异（搜索字段）
// 后委托 utils/profilesFilter 核心。Ref 入参改为普通值（调用方 useMemo 缓存）。
import type { GrokProfileDto } from '@/types'
import {
  buildProfilesFilter,
  type ProfilesFilterResult,
  type ProfilesSortBy,
  type ProfilesStatusFilter,
} from '@/utils/profilesFilter'

export type GrokProfilesStatusFilter = ProfilesStatusFilter
export type GrokProfilesSortBy = ProfilesSortBy

export interface GrokProfilesFilterInput {
  profiles: GrokProfileDto[]
  currentProfile: string | null
  query: string
  statusFilter: GrokProfilesStatusFilter
  tagFilter: string | null
  sortBy: GrokProfilesSortBy
}

export type GrokProfilesFilterResult = ProfilesFilterResult<GrokProfileDto>

const searchProfiles = (profiles: GrokProfileDto[], query: string): GrokProfileDto[] => {
  const normalized = query.trim().toLowerCase()
  if (!normalized) return profiles
  return profiles.filter((profile) =>
    [
      profile.name,
      profile.description ?? '',
      profile.base_url_display ?? '',
      profile.model ?? '',
      profile.provider ?? '',
      profile.api_backend ?? '',
      profile.reasoning_effort ?? '',
      profile.env_key ?? '',
      ...profile.tags,
    ].some((value) => value.toLowerCase().includes(normalized)),
  )
}

export function buildGrokProfilesFilter(
  input: GrokProfilesFilterInput,
): GrokProfilesFilterResult {
  return buildProfilesFilter<GrokProfileDto>({
    ...input,
    search: searchProfiles,
  })
}
