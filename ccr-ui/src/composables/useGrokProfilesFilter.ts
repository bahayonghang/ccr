import type { Ref } from 'vue'
import type { GrokProfileDto } from '@/types'
import {
  useProfilesFilter,
  type ProfilesSortBy,
  type ProfilesStatusFilter,
  type UseProfilesFilterOutput,
} from './useProfilesFilter'

export type GrokProfilesStatusFilter = ProfilesStatusFilter
export type GrokProfilesSortBy = ProfilesSortBy

export interface UseGrokProfilesFilterInput {
  profiles: Ref<GrokProfileDto[]>
  currentProfile: Ref<string | null>
  query: Ref<string>
  statusFilter: Ref<GrokProfilesStatusFilter>
  tagFilter: Ref<string | null>
  sortBy: Ref<GrokProfilesSortBy>
}

export type UseGrokProfilesFilterOutput = UseProfilesFilterOutput<GrokProfileDto>

const searchProfiles = (profiles: GrokProfileDto[], query: string): GrokProfileDto[] => {
  const normalized = query.trim().toLowerCase()
  if (!normalized) return profiles
  return profiles.filter((profile) => [
    profile.name,
    profile.description ?? '',
    profile.base_url_display ?? '',
    profile.model ?? '',
    profile.provider ?? '',
    profile.api_backend ?? '',
    profile.reasoning_effort ?? '',
    profile.env_key ?? '',
    ...profile.tags,
  ].some(value => value.toLowerCase().includes(normalized)))
}

export function useGrokProfilesFilter(
  input: UseGrokProfilesFilterInput,
): UseGrokProfilesFilterOutput {
  return useProfilesFilter<GrokProfileDto>({
    ...input,
    search: searchProfiles,
  })
}
