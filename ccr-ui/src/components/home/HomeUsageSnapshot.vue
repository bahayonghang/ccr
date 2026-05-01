<template>
  <section
    class="home-usage"
    data-home-usage-preview
  >
    <header class="home-usage__header">
      <div>
        <p class="home-section-kicker">
          {{ t('home.usageSnapshotEyebrow') }}
        </p>
        <h2>{{ t('home.usageSnapshotTitle') }}</h2>
        <p>{{ snapshotDescription }}</p>
      </div>

      <div class="home-usage__controls">
        <div
          class="home-usage__segmented"
          :aria-label="t('home.usageRangeLabel')"
        >
          <button
            v-for="days in dayOptions"
            :key="days"
            type="button"
            :class="{ 'is-active': activeDays === days }"
            @click="$emit('change-days', days)"
          >
            {{ t(`home.usageRange${days}`) }}
          </button>
        </div>
        <div
          class="home-usage__segmented"
          :aria-label="t('home.usageMetricSelectLabel')"
        >
          <button
            v-for="metric in metricOptions"
            :key="metric"
            type="button"
            :class="{ 'is-active': selectedMetric === metric }"
            @click="selectedMetric = metric"
          >
            {{ getMetricLabel(metric) }}
          </button>
        </div>
      </div>
    </header>

    <div class="home-usage__body">
      <div class="home-usage__summary">
        <div
          v-for="item in summaryItems"
          :key="item.label"
          class="home-usage-summary"
        >
          <span>{{ item.label }}</span>
          <strong>{{ item.value }}</strong>
        </div>
      </div>

      <div
        v-if="hasSeries"
        class="home-usage__chart"
        data-home-usage-bars
      >
        <span
          v-for="point in chartPoints"
          :key="point.key"
          class="home-usage-bar"
          :style="{ height: `${point.height}%` }"
          :title="point.title"
        />
      </div>

      <div
        v-else
        class="home-usage__empty"
      >
        <SIcon
          name="BarChart3"
          size="w-5 h-5"
        />
        <div>
          <h3>{{ emptyTitle }}</h3>
          <p>{{ emptyDescription }}</p>
        </div>
      </div>
    </div>

    <footer class="home-usage__footer">
      <span>{{ lastUpdatedLabel }}</span>
      <RouterLink to="/usage">
        {{ t('home.fullReport') }}
        <SIcon
          name="ArrowRight"
          size="w-4 h-4"
        />
      </RouterLink>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { translateWithFallback } from '@/i18n/formatMessage'
import SIcon from '@/components/ui/SIcon.vue'
import type { HomeUsageOverviewResponse } from '@/types/usage'
import type { HomeUsageMetric } from './types'

const props = defineProps<{
  overview: HomeUsageOverviewResponse | null
  loading: boolean
  error: string | null
  activeDays: number
}>()

defineEmits<{
  'change-days': [days: number]
}>()

const { t } = useI18n()

const dayOptions = [7, 30, 90]
const metricOptions: HomeUsageMetric[] = ['sessions', 'requests', 'tokens']
const selectedMetric = ref<HomeUsageMetric>('requests')

const formatCompact = (value?: number) => {
  if (typeof value !== 'number') return '...'
  return new Intl.NumberFormat(undefined, {
    notation: 'compact',
    maximumFractionDigits: 1,
  }).format(value)
}

const formatDateTime = (value?: string) => {
  if (!value) return t('home.usageLastUpdatedNever')
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return t('home.usageLastUpdatedNever')

  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

const getMetricLabel = (metric: HomeUsageMetric) => {
  switch (metric) {
    case 'sessions':
      return t('home.metricSessions')
    case 'tokens':
      return t('home.metricTokens')
    default:
      return t('home.metricRequests')
  }
}

const snapshotDescription = computed(() => {
  if (props.error) return t('home.usageSnapshotError')
  if (props.loading) return t('home.usagePreparing')
  if (props.overview?.empty_reason) return getEmptyReasonDescription()
  return t('home.usageSnapshotDescription')
})

const summaryItems = computed(() => [
  {
    label: t('home.metricSessions'),
    value: formatCompact(props.overview?.summary.total_sessions),
  },
  {
    label: t('home.metricRequests'),
    value: formatCompact(props.overview?.summary.total_requests),
  },
  {
    label: t('home.metricTokens'),
    value: formatCompact(props.overview?.summary.total_tokens),
  },
  {
    label: t('home.metricPlatforms'),
    value: formatCompact(props.overview?.summary.platforms),
  },
])

const getSeriesValue = (item: NonNullable<HomeUsageOverviewResponse['series']>[number]) => {
  return item.claude[selectedMetric.value] + item.codex[selectedMetric.value] + item.gemini[selectedMetric.value]
}

const chartPoints = computed(() => {
  const series = props.overview?.series ?? []
  const values = series.map(getSeriesValue)
  const max = Math.max(1, ...values)

  return series.map((item, index) => {
    const value = values[index] ?? 0
    return {
      key: item.date,
      height: Math.max(8, Math.round((value / max) * 100)),
      title: `${item.date}: ${formatCompact(value)} ${getMetricLabel(selectedMetric.value)}`,
    }
  })
})

const hasSeries = computed(() => chartPoints.value.length > 0 && chartPoints.value.some((point) => point.height > 8))

const emptyTitle = computed(() => {
  if (props.error) return t('home.usageSnapshotUnavailableTitle')
  if (props.loading) return t('home.usagePreparing')
  return t('home.usageSnapshotEmptyTitle')
})

const getEmptyReasonDescription = () => {
  switch (props.overview?.empty_reason) {
    case 'no_usage_logs':
      return t('usageStats.noUsageLogs')
    case 'no_session_index':
      return t('usageStats.noSessionIndex')
    case 'no_usage_and_sessions':
      return t('usageStats.noUsageAndSessions')
    default:
      return t('home.usageSnapshotEmptyDescription')
  }
}

const emptyDescription = computed(() => {
  if (props.error) return props.error
  if (props.loading) return t('home.usageSnapshotLoadingDescription')
  return getEmptyReasonDescription()
})

const lastUpdatedLabel = computed(() => (
  translateWithFallback(
    t,
    'home.usageLastUpdated',
    '更新于 {time}',
    { time: formatDateTime(props.overview?.last_updated) },
  )
))
</script>

<style scoped>
.home-usage {
  display: grid;
  gap: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 15%);
  border-radius: 14px;
  background: rgb(var(--color-bg-elevated-rgb) / 86%);
  padding: 1rem;
}

.home-usage__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.home-section-kicker {
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.home-usage h2,
.home-usage h3 {
  margin: 0;
  color: var(--color-text-primary);
  font-weight: 650;
  letter-spacing: 0;
}

.home-usage h2 {
  margin-top: 0.25rem;
  font-size: 1.2rem;
}

.home-usage__header p:not(.home-section-kicker) {
  max-width: 46rem;
  margin: 0.35rem 0 0;
  color: var(--color-text-secondary);
  font-size: 0.86rem;
  line-height: 1.6;
}

.home-usage__controls {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.5rem;
}

.home-usage__segmented {
  display: inline-flex;
  gap: 1px;
  overflow: hidden;
  border: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  border-radius: 8px;
  background: rgb(var(--color-border-default-rgb) / 8%);
}

.home-usage__segmented button {
  border: 0;
  background: rgb(var(--color-bg-surface-rgb) / 64%);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 0.72rem;
  font-weight: 700;
  padding: 0.44rem 0.58rem;
}

.home-usage__segmented button.is-active {
  background: var(--color-accent-primary);
  color: var(--color-text-inverted);
}

.home-usage__body {
  display: grid;
  grid-template-columns: minmax(12rem, 0.35fr) minmax(0, 1fr);
  gap: 1rem;
  align-items: stretch;
}

.home-usage__summary {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.5rem;
}

.home-usage-summary {
  display: grid;
  gap: 0.2rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 11%);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 58%);
  padding: 0.65rem;
}

.home-usage-summary span {
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.07em;
  text-transform: uppercase;
}

.home-usage-summary strong {
  overflow: hidden;
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-size: 1rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.home-usage__chart {
  display: flex;
  align-items: end;
  gap: 0.22rem;
  min-height: 9.8rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 10%);
  border-radius: 10px;
  background:
    linear-gradient(180deg, transparent 0%, rgb(var(--color-border-default-rgb) / 8%) 100%),
    rgb(var(--color-bg-surface-rgb) / 54%);
  padding: 0.75rem;
}

.home-usage-bar {
  flex: 1;
  min-width: 0.22rem;
  border-radius: 999px 999px 3px 3px;
  background: linear-gradient(180deg, var(--color-accent-primary), rgb(var(--color-accent-secondary-rgb) / 78%));
  opacity: 0.86;
}

.home-usage__empty {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-height: 9.8rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 22%);
  border-radius: 10px;
  color: var(--color-text-muted);
  padding: 0.9rem;
}

.home-usage__empty p {
  margin: 0.2rem 0 0;
  color: var(--color-text-secondary);
  font-size: 0.8rem;
  line-height: 1.55;
}

.home-usage__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  color: var(--color-text-muted);
  font-size: 0.76rem;
}

.home-usage__footer a {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--color-text-secondary);
  font-weight: 650;
}

.home-usage__footer a:hover,
.home-usage__footer a:focus-visible {
  color: var(--color-accent-primary);
}

@media (width <= 960px) {
  .home-usage__header,
  .home-usage__footer {
    align-items: flex-start;
    flex-direction: column;
  }

  .home-usage__controls {
    justify-content: flex-start;
  }

  .home-usage__body {
    grid-template-columns: 1fr;
  }
}
</style>
