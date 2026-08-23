import { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate } from 'react-router'
import { useQueryClient } from '@tanstack/react-query'
import {
  claudeObserverKeys,
  useClaudeObserverCacheStats,
  useClaudeObserverCostBreakdown,
  useClaudeObserverDailyTrend,
  useClaudeObserverInsight,
  useClaudeObserverSubscription,
  useClaudeObserverToolHeatmap,
  useClaudeObserverTopSessions,
  useClaudeObserverTopTools,
} from '@/features/claude/queries'
import type { TabId } from '@/features/claude/observer/ObserverChrome'
import { t } from '@/features/claude/locale'
import { readPrefersReducedMotion, REDUCED_MOTION_QUERY } from '@/utils/reducedMotion'

const TABS: { id: TabId; labelKey: string }[] = [
  { id: 'cost', labelKey: 'claudeCode.observer.tab.cost' },
  { id: 'token', labelKey: 'claudeCode.observer.tab.token' },
  { id: 'behavior', labelKey: 'claudeCode.observer.tab.behavior' },
]

function firstMessage(errors: Array<{ message: string } | null | undefined>): string | null {
  const hit = errors.find((error) => Boolean(error?.message))
  return hit?.message ?? null
}

function isInsightEmpty(insight: {
  total_value_usd: number
  month_value_usd: number
  today_value_usd: number
  total_sessions: number
} | undefined): boolean {
  if (!insight) return false
  return insight.total_value_usd === 0 && insight.month_value_usd === 0 && insight.today_value_usd === 0 && insight.total_sessions === 0
}

function resolveState(input: {
  isLoading: boolean
  loadError: string | null
  hasInsight: boolean
  empty: boolean
}) {
  if (input.isLoading) return 'loading' as const
  if (input.loadError && !input.hasInsight) return 'error' as const
  if (input.empty) return 'empty' as const
  return 'ready' as const
}

export function useObserverPanel() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [activeTab, setActiveTab] = useState<TabId>('cost')
  const [dialogOpen, setDialogOpen] = useState(false)
  const [renderedTabs, setRenderedTabs] = useState<Set<TabId>>(() => new Set(['cost']))
  const [animationsEnabled, setAnimationsEnabled] = useState(() => !readPrefersReducedMotion())

  const insightQuery = useClaudeObserverInsight()
  const dailyQuery = useClaudeObserverDailyTrend(30)
  const projectQuery = useClaudeObserverCostBreakdown('project', 30, 10)
  const modelQuery = useClaudeObserverCostBreakdown('model', 30, 10)
  const cacheQuery = useClaudeObserverCacheStats()
  const sessionQuery = useClaudeObserverTopSessions(10, 'cost')
  const heatmapQuery = useClaudeObserverToolHeatmap(30)
  const toolsQuery = useClaudeObserverTopTools(30, 10)
  const subscriptionQuery = useClaudeObserverSubscription()

  useEffect(() => {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return undefined
    const media = window.matchMedia(REDUCED_MOTION_QUERY)
    const sync = () => setAnimationsEnabled(!readPrefersReducedMotion())
    media.addEventListener('change', sync)
    return () => media.removeEventListener('change', sync)
  }, [])

  const selectTab = useCallback((id: TabId) => {
    setActiveTab(id)
    setRenderedTabs((current) => {
      if (current.has(id)) return current
      const next = new Set(current)
      next.add(id)
      return next
    })
  }, [])
  const openDialog = useCallback(() => setDialogOpen(true), [])
  const closeDialog = useCallback(() => setDialogOpen(false), [])
  const refresh = useCallback(() => {
    void queryClient.invalidateQueries({ queryKey: claudeObserverKeys.all })
  }, [queryClient])
  const goToUsage = useCallback(() => {
    void navigate('/usage')
  }, [navigate])

  const insight = insightQuery.data
  const subscription = subscriptionQuery.data ?? insight?.subscription ?? null
  const loadError = firstMessage([
    insightQuery.error,
    cacheQuery.error,
    dailyQuery.error,
    projectQuery.error,
    modelQuery.error,
  ])
  const state = resolveState({
    isLoading: insightQuery.isPending && !insight,
    loadError,
    hasInsight: Boolean(insight),
    empty: isInsightEmpty(insight),
  })
  const tabs = useMemo(() => TABS.map((tab) => ({ id: tab.id, label: t(tab.labelKey) })), [])
  const pricingNote = insight?.pricing_version
    ? t('claudeCode.observer.subtitleWithVersion', { version: insight.pricing_version })
    : t('claudeCode.observer.subtitle')
  const emptyDescription = loadError
    ? `${t('claudeCode.observer.empty.loadError')}: ${loadError}`
    : t('claudeCode.observer.empty.noUsageDesc')
  const hasRoi = Boolean(
    subscription?.mode === 'subscription' && Number.isFinite(insight?.roi ?? null) && (insight?.roi ?? 0) > 0,
  )

  return {
    activeTab,
    dialogOpen,
    renderedTabs,
    animationsEnabled,
    insight,
    subscription,
    loadError,
    state,
    pricingNote,
    emptyDescription,
    tabs,
    hasRoi,
    daily: dailyQuery.data,
    byProject: projectQuery.data,
    byModel: modelQuery.data,
    stats: cacheQuery.data,
    heatmap: heatmapQuery.data,
    topTools: toolsQuery.data,
    sessions: sessionQuery.data,
    selectTab,
    openDialog,
    closeDialog,
    refresh,
    goToUsage,
  }
}
