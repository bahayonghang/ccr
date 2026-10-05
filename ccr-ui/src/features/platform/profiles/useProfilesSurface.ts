import { useCallback, useMemo, useState } from 'react'
import type { ProfileDisplayRecord } from '@/configs/profileDisplayRecord'
import {
  useProfilesViewStore,
  type ProfilesSurfaceViewMode,
} from '@/configs/profilesSurfaceRuntime'
import type { ProfilesStatusFilter, ProviderOption } from '@/utils/profilesFilter'

export type ProfilesSurfaceSortBy = 'name' | 'usage'

export interface ProfilesStats {
  total: number
  vendorCount: number
  tagCounts: Record<string, number>
  authCounts: Record<string, number>
}

export interface UseProfilesSurfaceArgs {
  platformKey: string
  records: readonly ProfileDisplayRecord[]
  current: string | null
}

interface FilterState {
  query: string
  tagFilter: string | null
  providerFilter: string | null
  statusFilter: ProfilesStatusFilter
}

const matchesStatus = (
  record: ProfileDisplayRecord,
  status: ProfilesStatusFilter,
): boolean => {
  if (status === 'all') return true
  if (status === 'active') return record.current
  if (status === 'enabled') return record.enabled
  return !record.enabled
}

const recordMatches = (record: ProfileDisplayRecord, filters: FilterState): boolean => {
  if (filters.query && !record.searchText.includes(filters.query)) return false
  if (filters.tagFilter !== null && !record.tags.includes(filters.tagFilter)) return false
  if (filters.providerFilter !== null && record.vendorKey !== filters.providerFilter) {
    return false
  }
  return matchesStatus(record, filters.statusFilter)
}

const compareRecords = (
  left: ProfileDisplayRecord,
  right: ProfileDisplayRecord,
  sortBy: ProfilesSurfaceSortBy,
): number => {
  if (sortBy === 'name') return left.sortKeys.name.localeCompare(right.sortKeys.name)
  return right.sortKeys.usageCount - left.sortKeys.usageCount
}

const collectStats = (records: readonly ProfileDisplayRecord[]): ProfilesStats => {
  const vendors = new Set<string>()
  const tagCounts: Record<string, number> = {}
  const authCounts: Record<string, number> = {}
  for (const record of records) {
    if (record.vendorKey) vendors.add(record.vendorKey)
    for (const tag of record.tags) {
      tagCounts[tag] = (tagCounts[tag] ?? 0) + 1
    }
    authCounts[record.authKey] = (authCounts[record.authKey] ?? 0) + 1
  }
  return {
    total: records.length,
    vendorCount: vendors.size,
    tagCounts,
    authCounts,
  }
}

const uniqueSorted = (values: Iterable<string>): string[] => {
  const items = [...new Set(values)]
  items.sort((left, right) => left.localeCompare(right))
  return items
}

/** 呈现层状态：筛选、视图、空编辑目标；不发起请求。 */
export function useProfilesSurface(args: UseProfilesSurfaceArgs) {
  const { platformKey, records } = args
  const [query, setQuery] = useState('')
  const [tagFilter, setTagFilter] = useState<string | null>(null)
  const [providerFilter, setProviderFilter] = useState<string | null>(null)
  const [statusFilter, setStatusFilter] = useState<ProfilesStatusFilter>('all')
  const [sortBy, setSortBy] = useState<ProfilesSurfaceSortBy>('name')
  const [editorTarget, setEditorTarget] = useState<string | null>(null)
  const [sourceMode, setSourceMode] = useState(false)
  const [inspectorOpen, setInspectorOpen] = useState(false)
  const [focusedName, setFocusedName] = useState<string | null>(null)

  const viewMode = useProfilesViewStore(
    (state) => state.viewByPlatform[platformKey] ?? 'card',
  )
  const persistView = useProfilesViewStore((state) => state.setView)
  const setViewMode = useCallback(
    (mode: ProfilesSurfaceViewMode) => {
      persistView(platformKey, mode)
    },
    [persistView, platformKey],
  )

  const filters = useMemo<FilterState>(
    () => ({
      query: query.trim().toLowerCase(),
      tagFilter,
      providerFilter,
      statusFilter,
    }),
    [providerFilter, query, statusFilter, tagFilter],
  )

  const filtered = useMemo(() => {
    const next = records.filter((record) => recordMatches(record, filters))
    next.sort((left, right) => compareRecords(left, right, sortBy))
    return next
  }, [filters, records, sortBy])

  const stats = useMemo(() => collectStats(records), [records])

  const allTags = useMemo(() => {
    const tags: string[] = []
    for (const record of records) {
      for (const tag of record.tags) tags.push(tag)
    }
    return uniqueSorted(tags)
  }, [records])

  const allProviders = useMemo<ProviderOption[]>(() => {
    const keys: string[] = []
    for (const record of records) {
      if (record.vendorKey) keys.push(record.vendorKey)
    }
    return uniqueSorted(keys).map((key) => ({ key, label: key }))
  }, [records])

  const clearFilters = useCallback(() => {
    setQuery('')
    setTagFilter(null)
    setProviderFilter(null)
    setStatusFilter('all')
  }, [])

  const previewName = focusedName ?? args.current
  const previewRecord = useMemo(
    () => records.find((record) => record.name === previewName) ?? null,
    [previewName, records],
  )
  const currentRecord = useMemo(
    () => records.find((record) => record.current) ?? null,
    [records],
  )

  return {
    query,
    setQuery,
    tagFilter,
    setTagFilter,
    providerFilter,
    setProviderFilter,
    statusFilter,
    setStatusFilter,
    sortBy,
    setSortBy,
    viewMode,
    setViewMode,
    filtered,
    stats,
    allTags,
    allProviders,
    editorTarget,
    setEditorTarget,
    sourceMode,
    setSourceMode,
    inspectorOpen,
    setInspectorOpen,
    focusedName,
    setFocusedName,
    previewRecord,
    currentRecord,
    clearFilters,
  }
}
