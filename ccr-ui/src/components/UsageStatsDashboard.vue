<template>
  <Card
    variant="glass"
    class="relative overflow-hidden"
  >
    <div
      class="absolute top-0 right-0 h-64 w-64 rounded-full bg-accent-secondary/5 blur-3xl -mr-16 -mt-16"
    />

    <div class="relative z-10 space-y-6">
      <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div class="space-y-2">
          <div>
            <h2 class="flex items-center gap-2 text-xl font-bold text-text-primary">
              <SIcon
                name="BarChart3"
                size="w-5 h-5"
                class="text-accent-secondary"
              />
              {{ $t('usageStats.title') }}
            </h2>
            <p class="mt-1 text-sm text-text-muted">
              {{ $t('usageStats.updated') }} {{ formatLastUpdated(overview?.last_updated) }} ·
              {{
                $t('usageStats.rangeSummary', {
                  days: selectedDays,
                  count: overview?.summary?.total_sessions ?? 0,
                })
              }}
            </p>
          </div>

          <div
            v-if="diagnosticMessage || bootstrapNotes.length"
            class="flex flex-wrap gap-2"
          >
            <span
              v-if="diagnosticMessage"
              class="rounded-full border border-accent-warning/25 bg-accent-warning/10 px-3 py-1 text-[11px] text-text-primary"
            >
              {{ diagnosticMessage }}
            </span>
            <span
              v-for="note in bootstrapNotes"
              :key="note"
              class="rounded-full border border-accent-info/25 bg-accent-info/10 px-3 py-1 text-[11px] text-text-primary"
            >
              {{ note }}
            </span>
          </div>
        </div>

        <div class="flex flex-wrap items-center gap-3">
          <div class="flex items-center gap-1 rounded-lg bg-bg-surface/50 p-1">
            <button
              v-for="range in dateRanges"
              :key="range.days"
              class="rounded-md px-3 py-1 text-xs font-medium transition-colors duration-200"
              :class="
                selectedDays === range.days
                  ? 'bg-accent-secondary text-white shadow-sm'
                  : 'text-text-secondary hover:bg-bg-elevated/50 hover:text-text-primary'
              "
              @click="setDateRange(range.days)"
            >
              {{ range.label }}
            </button>
          </div>

          <div class="flex items-center gap-1 rounded-lg bg-bg-surface/50 p-1">
            <button
              v-for="mode in viewModes"
              :key="mode.value"
              class="rounded-md px-4 py-1.5 text-sm font-medium transition-colors duration-200"
              :class="
                viewMode === mode.value
                  ? 'bg-accent-secondary text-white shadow-sm'
                  : 'text-text-secondary hover:bg-bg-elevated/50 hover:text-text-primary'
              "
              @click="viewMode = mode.value"
            >
              {{ $t(mode.labelKey) }}
            </button>
          </div>
        </div>
      </div>

      <div
        v-if="loading"
        class="flex h-72 items-center justify-center"
      >
        <div class="h-8 w-8 animate-spin rounded-full border-b-2 border-accent-secondary" />
      </div>
      <div
        v-else-if="error"
        class="flex h-72 flex-col items-center justify-center gap-2 text-center"
      >
        <p class="text-sm font-semibold text-accent-danger">
          {{ error }}
        </p>
        <p class="text-xs text-text-muted">
          {{ $t('usageStats.fullReportHint') }}
        </p>
      </div>
      <div
        v-else-if="isFullyEmpty"
        class="flex h-72 flex-col items-center justify-center gap-3 text-center"
      >
        <p class="text-base font-semibold text-text-primary">
          {{ $t('usageStats.noDataTitle') }}
        </p>
        <p class="max-w-xl text-sm text-text-secondary">
          {{ diagnosticMessage }}
        </p>
        <p class="text-xs text-text-muted">
          {{ $t('usageStats.fullReportHint') }}
        </p>
      </div>
      <div v-else-if="hasChartData">
        <UsageStatsChart
          :data="overview?.series ?? []"
          :view-mode="viewMode"
        />
      </div>
      <div
        v-else
        class="flex h-72 items-center justify-center text-text-muted"
      >
        {{ $t('usageStats.noData') }}
      </div>

      <div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <div
          v-for="card in summaryCards"
          :key="card.label"
          class="group relative overflow-hidden rounded-xl border border-border-default/30 bg-bg-elevated/30 p-4 transition-[color,background-color,box-shadow] duration-300"
          :class="card.hoverClass"
        >
          <div
            class="absolute top-0 left-0 h-full w-1 rounded-r"
            :class="card.barClass"
          />
          <div class="ml-1 flex items-start gap-3">
            <div
              class="shrink-0 rounded-lg p-2 transition-colors"
              :class="card.badgeClass"
            >
              <SIcon
                :name="card.icon"
                size="w-4 h-4"
                :class="card.iconClass"
              />
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-[11px] font-medium uppercase tracking-wider text-text-muted">
                {{ card.label }}
              </p>
              <p class="mt-0.5 text-2xl font-bold tabular-nums text-text-primary">
                {{ card.value }}
              </p>
              <p class="mt-1 text-[11px] text-text-secondary">
                {{ card.note }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <div class="h-px bg-gradient-to-r from-transparent via-border-default/50 to-transparent" />

      <div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <div
          v-for="card in platformCards"
          :key="card.key"
          class="rounded-xl border p-4 transition-[color,background-color,box-shadow] duration-300"
          :class="card.containerClass"
        >
          <div class="mb-3 flex items-center gap-2">
            <div
              class="h-2.5 w-2.5 rounded-full ring-2"
              :class="card.dotClass"
            />
            <span
              class="font-semibold"
              :class="card.titleClass"
            >
              {{ card.label }}
            </span>
            <span class="ml-auto font-mono text-xs tabular-nums text-text-muted">
              {{ formatNumber(card.stats.sessions) }}
            </span>
          </div>
          <div class="space-y-1.5 text-xs text-text-secondary">
            <div class="flex items-center gap-1.5">
              <SIcon
                name="Hash"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatNumber(card.stats.requests) }} {{ $t('usageStats.requests') }}</span>
            </div>
            <div class="flex items-center gap-1.5">
              <SIcon
                name="BarChart3"
                size="w-3 h-3"
                class="opacity-40"
              />
              <span>{{ formatTokens(card.stats.tokens) }} {{ $t('usageStats.tokens') }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import UsageStatsChart from '@/components/UsageStatsChart.vue'
import { getHomeUsageOverviewV2 } from '@/api'
import type {
  HomeOverviewPlatformStats,
  HomeOverviewViewMode,
  HomeUsageOverviewResponse,
  Platform,
} from '@/types/usage'
import { logger } from '@/utils/logger'

type HomePlatformKey = Platform

const { t } = useI18n()

const viewMode = ref<HomeOverviewViewMode>('sessions')
const viewModes = [
  { value: 'sessions' as HomeOverviewViewMode, labelKey: 'usageStats.viewModes.sessions' },
  { value: 'requests' as HomeOverviewViewMode, labelKey: 'usageStats.viewModes.requests' },
  { value: 'tokens' as HomeOverviewViewMode, labelKey: 'usageStats.viewModes.tokens' },
]

const selectedDays = ref(30)
const dateRanges = [
  { days: 7, label: '7D' },
  { days: 30, label: '30D' },
  { days: 90, label: '90D' },
]

const overview = ref<HomeUsageOverviewResponse | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const OVERVIEW_CACHE_TTL = 60_000
const overviewCache = new Map<number, { data: HomeUsageOverviewResponse; ts: number }>()

const emptyStats = (): HomeOverviewPlatformStats => ({
  sessions: 0,
  requests: 0,
  tokens: 0,
})

const getPlatformStats = (platform: HomePlatformKey): HomeOverviewPlatformStats => {
  return overview.value?.by_platform?.[platform] ?? emptyStats()
}

const getChartValue = (platform: HomePlatformKey, mode: HomeOverviewViewMode) => {
  return (item: HomeUsageOverviewResponse['series'][number]) => {
    const stats = item?.[platform] ?? emptyStats()
    switch (mode) {
      case 'sessions':
        return stats.sessions
      case 'requests':
        return stats.requests
      case 'tokens':
        return stats.tokens
      default:
        return stats.sessions
    }
  }
}

const hasSeriesDataForMode = (
  data: HomeUsageOverviewResponse | null,
  mode: HomeOverviewViewMode
) => {
  const series = data?.series ?? []
  return series.some((item) => {
    const claudeValue = getChartValue('claude', mode)(item)
    const codexValue = getChartValue('codex', mode)(item)
    const geminiValue = getChartValue('gemini', mode)(item)
    return claudeValue + codexValue + geminiValue > 0
  })
}

const hasChartData = computed(() => {
  return hasSeriesDataForMode(overview.value, viewMode.value)
})

const diagnosticMessage = computed(() => {
  switch (overview.value?.empty_reason) {
    case 'no_usage_logs':
      return t('usageStats.noUsageLogs')
    case 'no_session_index':
      return t('usageStats.noSessionIndex')
    case 'no_usage_and_sessions':
      return t('usageStats.noUsageAndSessions')
    default:
      return ''
  }
})

const bootstrapNotes = computed(() => {
  const notes: string[] = []
  const bootstrap = overview.value?.bootstrap
  if (!bootstrap) return notes
  if (bootstrap.usage_imported_records > 0) {
    notes.push(
      t('usageStats.bootstrapImported', { count: formatNumber(bootstrap.usage_imported_records) })
    )
  }
  if (bootstrap.session_reindex_attempted && bootstrap.indexed_sessions > 0) {
    notes.push(
      t('usageStats.bootstrapIndexed', { count: formatNumber(bootstrap.indexed_sessions) })
    )
  }
  return notes
})

const isFullyEmpty = computed(() => overview.value?.empty_reason === 'no_usage_and_sessions')

const summaryCards = computed(() => {
  const summary = overview.value?.summary
  return [
    {
      label: t('usageStats.sessions'),
      value: formatNumber(summary?.total_sessions ?? 0),
      note: t('usageStats.startedInRange'),
      icon: 'Activity',
      barClass: 'bg-emerald-500/60',
      badgeClass: 'bg-emerald-500/10 group-hover:bg-emerald-500/20',
      iconClass: 'text-emerald-400',
      hoverClass: 'hover:border-emerald-500/30',
    },
    {
      label: t('usageStats.requests'),
      value: formatNumber(summary?.total_requests ?? 0),
      note: t('usageStats.inSelectedRange'),
      icon: 'MessageSquare',
      barClass: 'bg-blue-500/60',
      badgeClass: 'bg-blue-500/10 group-hover:bg-blue-500/20',
      iconClass: 'text-blue-400',
      hoverClass: 'hover:border-blue-500/30',
    },
    {
      label: t('usageStats.activeDays'),
      value: formatNumber(summary?.active_days ?? 0),
      note: t('usageStats.daysWithActivity'),
      icon: 'Calendar',
      barClass: 'bg-amber-500/60',
      badgeClass: 'bg-amber-500/10 group-hover:bg-amber-500/20',
      iconClass: 'text-amber-400',
      hoverClass: 'hover:border-amber-500/30',
    },
    {
      label: t('usageStats.platforms'),
      value: formatNumber(summary?.platforms ?? 0),
      note: t('usageStats.trackedPlatforms'),
      icon: 'Layers',
      barClass: 'bg-purple-500/60',
      badgeClass: 'bg-purple-500/10 group-hover:bg-purple-500/20',
      iconClass: 'text-purple-400',
      hoverClass: 'hover:border-purple-500/30',
    },
  ]
})

const platformCards = computed(() => {
  const summary = overview.value?.summary
  return [
    {
      key: 'all',
      label: 'All',
      stats: {
        sessions: summary?.total_sessions ?? 0,
        requests: summary?.total_requests ?? 0,
        tokens: summary?.total_tokens ?? 0,
      },
      containerClass: 'bg-bg-elevated/40 border-border-default/50 hover:bg-bg-elevated/60',
      dotClass: 'bg-text-secondary ring-text-secondary/20',
      titleClass: 'text-text-primary',
    },
    {
      key: 'codex',
      label: 'Codex',
      stats: getPlatformStats('codex'),
      containerClass:
        'bg-orange-500/5 border-orange-500/20 hover:border-orange-500/40 hover:bg-orange-500/10',
      dotClass: 'bg-orange-500 ring-orange-500/20',
      titleClass: 'text-orange-400',
    },
    {
      key: 'claude',
      label: 'Claude',
      stats: getPlatformStats('claude'),
      containerClass:
        'bg-pink-500/5 border-pink-500/20 hover:border-pink-500/40 hover:bg-pink-500/10',
      dotClass: 'bg-pink-400 ring-pink-400/20',
      titleClass: 'text-pink-400',
    },
    {
      key: 'gemini',
      label: 'Gemini',
      stats: getPlatformStats('gemini'),
      containerClass:
        'bg-blue-500/5 border-blue-500/20 hover:border-blue-500/40 hover:bg-blue-500/10',
      dotClass: 'bg-blue-500 ring-blue-500/20',
      titleClass: 'text-blue-400',
    },
  ]
})

const loadOverview = async (days: number, force: boolean = false) => {
  const cached = overviewCache.get(days)
  const now = Date.now()
  if (!force && cached && now - cached.ts < OVERVIEW_CACHE_TTL) {
    overview.value = cached.data
    error.value = null
    loading.value = false
    return
  }

  loading.value = true
  error.value = null

  try {
    const data = await getHomeUsageOverviewV2<HomeUsageOverviewResponse>(days)
    overview.value = data
    overviewCache.set(days, { data, ts: now })

    if (!hasSeriesDataForMode(data, viewMode.value)) {
      if (hasSeriesDataForMode(data, 'sessions')) {
        viewMode.value = 'sessions'
      } else if (hasSeriesDataForMode(data, 'requests')) {
        viewMode.value = 'requests'
      } else if (hasSeriesDataForMode(data, 'tokens')) {
        viewMode.value = 'tokens'
      }
    }
  } catch (loadError) {
    error.value = loadError instanceof Error ? loadError.message : String(loadError)
    logger.error('[UsageStatsDashboard] failed to load home overview', loadError)
  } finally {
    loading.value = false
  }
}

const setDateRange = async (days: number) => {
  if (selectedDays.value === days && overview.value) return
  selectedDays.value = days
  await loadOverview(days)
}

const formatNumber = (num: number): string => {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toString()
}

const formatTokens = (num: number): string => {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toString()
}

const formatLastUpdated = (dateStr?: string): string => {
  if (!dateStr) return 'N/A'
  try {
    return new Date(dateStr).toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return dateStr
  }
}

onMounted(async () => {
  await loadOverview(selectedDays.value)
})
</script>
