import { useCallback } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { EnvironmentInfo } from '@/types/generated/environment/EnvironmentInfo'
import {
  GROK_OVERVIEW_STALE_TIME,
  GROK_VERSION_STALE_TIME,
  fetchGrokEnvironment,
  fetchGrokOverview,
  fetchGrokVersion,
  grokKeys,
  type GrokOverviewLoadResult,
} from './queries'

export function useGrokHomeQueries() {
  const environmentQuery = useQuery({
    queryKey: grokKeys.environment(),
    queryFn: fetchGrokEnvironment,
    staleTime: 0,
  })
  const environment = (environmentQuery.data ?? null) as EnvironmentInfo | null
  const environmentId = environment?.id ?? null

  const overviewQuery = useQuery({
    queryKey: grokKeys.overview(environmentId),
    queryFn: fetchGrokOverview,
    enabled: environmentQuery.isSuccess && environment?.env_type === 'local',
    staleTime: GROK_OVERVIEW_STALE_TIME,
  })
  const overviewResult = (overviewQuery.data ?? null) as GrokOverviewLoadResult | null
  const overview = overviewResult?.status === 'ok' ? overviewResult.data : null

  const versionQuery = useQuery({
    queryKey: grokKeys.version(environmentId),
    queryFn: fetchGrokVersion,
    enabled: overview !== null,
    staleTime: GROK_VERSION_STALE_TIME,
  })

  const refetchEnvironment = environmentQuery.refetch
  const refetchOverview = overviewQuery.refetch
  const refetchVersion = versionQuery.refetch

  const refresh = useCallback(
    async (force = false) => {
      await refetchEnvironment()
      if (environment?.env_type !== 'local') return
      const tasks: Array<Promise<unknown>> = [refetchOverview()]
      if (overview || force) tasks.push(refetchVersion())
      await Promise.allSettled(tasks)
    },
    [environment?.env_type, overview, refetchEnvironment, refetchOverview, refetchVersion],
  )

  return {
    environment,
    environmentQuery,
    overviewQuery,
    overviewResult,
    overview,
    versionQuery,
    refresh,
  }
}
