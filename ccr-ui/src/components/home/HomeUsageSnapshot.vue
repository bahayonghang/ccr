<template>
  <section
    class="home-usage"
    data-home-usage-preview
  >
    <header class="home-usage__header">
      <div class="home-usage__lede">
        <p class="home-usage__eyebrow">
          {{ t('home.usageSnapshotEyebrow') }}
        </p>
        <h2 class="home-usage__title">
          {{ t('home.usageSnapshotTitle') }}
        </h2>
        <p class="home-usage__description">
          {{ snapshotDescription }}
        </p>
      </div>

      <div
        class="home-usage__range"
        role="group"
        :aria-label="t('home.usageRangeLabel')"
      >
        <button
          v-for="days in dayOptions"
          :key="days"
          type="button"
          class="home-usage-range-btn"
          :data-active="activeDays === days ? 'true' : 'false'"
          :aria-pressed="activeDays === days"
          @click="$emit('change-days', days)"
        >
          {{ t(`home.usageRange${days}`) }}
        </button>
      </div>
    </header>

    <div class="home-usage__body">
      <div class="home-usage__summary">
        <div
          v-for="item in summaryItems"
          :key="item.label"
          class="home-usage-summary"
        >
          <span class="home-usage-summary__label">{{ item.label }}</span>
          <strong class="home-usage-summary__value">{{ item.value }}</strong>
        </div>
      </div>

      <div class="home-usage__chartArea">
        <div
          class="home-usage__metric"
          role="group"
          :aria-label="t('home.usageMetricSelectLabel')"
        >
          <button
            v-for="metric in metricOptions"
            :key="metric"
            type="button"
            class="home-usage-metric-btn"
            :data-active="selectedMetric === metric ? 'true' : 'false'"
            :aria-pressed="selectedMetric === metric"
            @click="selectedMetric = metric"
          >
            {{ getMetricLabel(metric) }}
          </button>
        </div>

        <div
          v-if="hasSeries"
          class="home-usage__chart"
          :class="{ 'home-usage__chart--ghost': !hasMeaningfulSeries }"
          data-home-usage-bars
          @mouseleave="hoveredKey = null"
        >
          <span
            class="home-usage__chart-readout"
            :data-visible="hoveredPoint ? 'true' : 'false'"
            aria-live="polite"
          >
            <template v-if="hoveredPoint">
              <span class="home-usage__chart-readout-date">{{ hoveredPoint.dateLabel }}</span>
              <span class="home-usage__chart-readout-value">{{ hoveredPoint.valueLabel }}</span>
              <span class="home-usage__chart-readout-metric">{{ getMetricLabel(selectedMetric) }}</span>
            </template>
            <template v-else>
              <span class="home-usage__chart-readout-placeholder">
                {{ t('home.usageSnapshotDescription') }}
              </span>
            </template>
          </span>

          <div
            class="home-usage__chart-grid"
            aria-hidden="true"
          >
            <span class="home-usage__chart-grid-line home-usage__chart-grid-line--top" />
            <span class="home-usage__chart-grid-line home-usage__chart-grid-line--bottom" />
          </div>

          <div class="home-usage__chart-bars">
            <button
              v-for="point in chartPoints"
              :key="point.key"
              type="button"
              class="home-usage-bar"
              :data-active="hoveredKey === point.key ? 'true' : 'false'"
              :style="{ height: `${point.height}%` }"
              :title="point.title"
              :aria-label="point.title"
              @mouseenter="hoveredKey = point.key"
              @focus="hoveredKey = point.key"
              @blur="hoveredKey = null"
            />
          </div>
        </div>

        <div
          v-else
          class="home-usage__empty"
        >
          <span class="home-usage__empty-icon">
            <SIcon
              name="BarChart3"
              size="w-5 h-5"
            />
          </span>
          <div>
            <h3>{{ emptyTitle }}</h3>
            <p>{{ emptyDescription }}</p>
          </div>
        </div>
      </div>
    </div>

    <footer class="home-usage__footer">
      <span class="home-usage__last">{{ lastUpdatedLabel }}</span>
      <RouterLink
        to="/usage"
        class="home-usage__report-link"
      >
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
const hoveredKey = ref<string | null>(null)

const formatCompact = (value?: number) => {
  if (typeof value !== 'number') return '…'
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

const formatDateLabel = (value: string) => {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return new Intl.DateTimeFormat(undefined, { month: 'short', day: '2-digit' }).format(date)
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
  return item.claude[selectedMetric.value]
    + item.codex[selectedMetric.value]
    + item.gemini[selectedMetric.value]
    + (item.opencode?.[selectedMetric.value] ?? 0)
}

const chartPoints = computed(() => {
  const series = props.overview?.series ?? []
  const values = series.map(getSeriesValue)
  const max = Math.max(1, ...values)

  return series.map((item, index) => {
    const value = values[index] ?? 0
    return {
      key: item.date,
      date: item.date,
      dateLabel: formatDateLabel(item.date),
      value,
      valueLabel: formatCompact(value),
      height: Math.max(6, Math.round((value / max) * 100)),
      title: `${formatDateLabel(item.date)} · ${formatCompact(value)} ${getMetricLabel(selectedMetric.value)}`,
    }
  })
})

const hasSeries = computed(() => chartPoints.value.length > 0)
const hasMeaningfulSeries = computed(() => chartPoints.value.some((point) => point.value > 0))

const hoveredPoint = computed(() => {
  if (!hoveredKey.value) return null
  return chartPoints.value.find((point) => point.key === hoveredKey.value) ?? null
})

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
  gap: 0.85rem;
  padding: var(--home-card-pad);
  border: 1px solid var(--home-border-card);
  border-radius: var(--home-card-radius);
  background: var(--home-surface-card);
  box-shadow: var(--home-elevation-raised);
}

.home-usage__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.home-usage__lede {
  display: grid;
  gap: 0.2rem;
  min-width: 0;
  max-width: 48rem;
}

.home-usage__eyebrow {
  margin: 0;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-usage__title {
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: var(--home-text-section);
  font-weight: 620;
  letter-spacing: var(--home-tracking-display);
}

.home-usage__description {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-body);
  letter-spacing: var(--home-tracking-body);
  line-height: var(--home-leading-body);
}

.home-usage__range {
  display: inline-flex;
  gap: 0.2rem;
  padding: 0.18rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 999px;
  background: var(--home-surface-sunk);
  box-shadow: var(--home-elevation-sunk);
}

.home-usage-range-btn {
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  padding: 0.32rem 0.7rem;
  text-transform: uppercase;
  transition:
    background-color var(--home-motion-duration) var(--home-motion-ease),
    color var(--home-motion-duration) var(--home-motion-ease);
}

.home-usage-range-btn:hover {
  color: var(--color-text-primary);
}

.home-usage-range-btn:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.home-usage-range-btn[data-active='true'] {
  background: var(--color-accent-primary);
  color: var(--color-text-inverted);
}

.home-usage__body {
  display: grid;
  grid-template-columns: minmax(0, 4fr) minmax(0, 8fr);
  gap: 0.85rem;
  align-items: stretch;
}

.home-usage__summary {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-content: start;
  gap: 0.55rem;
}

.home-usage-summary {
  display: grid;
  gap: 0.18rem;
  padding: 0.7rem 0.8rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 64%);
}

.home-usage-summary__label {
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-usage-summary__value {
  overflow: hidden;
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono-lg);
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.home-usage__chartArea {
  display: grid;
  grid-template-rows: auto 1fr;
  gap: 0.55rem;
  min-width: 0;
}

.home-usage__metric {
  justify-self: end;
  display: inline-flex;
  gap: 0.2rem;
  padding: 0.18rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 999px;
  background: var(--home-surface-sunk);
}

.home-usage-metric-btn {
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  padding: 0.28rem 0.6rem;
  text-transform: uppercase;
  transition:
    background-color var(--home-motion-duration) var(--home-motion-ease),
    color var(--home-motion-duration) var(--home-motion-ease);
}

.home-usage-metric-btn:hover {
  color: var(--color-text-primary);
}

.home-usage-metric-btn:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.home-usage-metric-btn[data-active='true'] {
  background: rgb(var(--color-accent-primary-rgb) / 14%);
  color: var(--color-accent-primary);
}

.home-usage__chart {
  position: relative;
  display: flex;
  align-items: end;
  gap: 0.22rem;
  min-height: 10rem;
  padding: 1.5rem 0.75rem 0.5rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 48%);
  overflow: hidden;
}

.home-usage__chart--ghost .home-usage-bar {
  opacity: 0.34;
}

.home-usage__chart-readout {
  position: absolute;
  top: 0.4rem;
  left: 0.75rem;
  right: 0.75rem;
  display: flex;
  align-items: baseline;
  justify-content: flex-start;
  gap: 0.45rem;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
  pointer-events: none;
}

.home-usage__chart-readout-placeholder {
  color: var(--color-text-disabled);
  font-size: var(--home-text-meta);
  font-weight: 400;
  letter-spacing: 0;
  text-transform: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.home-usage__chart-readout-date {
  color: var(--color-text-muted);
}

.home-usage__chart-readout-value {
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono);
}

.home-usage__chart-readout-metric {
  color: var(--color-text-muted);
}

.home-usage__chart-grid {
  position: absolute;
  inset: 1.5rem 0.75rem 0.5rem;
  pointer-events: none;
}

.home-usage__chart-grid-line {
  position: absolute;
  left: 0;
  right: 0;
  height: 0;
  border-top: 1px dashed rgb(var(--color-border-default-rgb) / 16%);
}

.home-usage__chart-grid-line--top { top: 0; }
.home-usage__chart-grid-line--bottom { bottom: 0; }

.home-usage__chart-bars {
  position: relative;
  display: flex;
  align-items: end;
  gap: 0.22rem;
  width: 100%;
  height: 100%;
  z-index: 1;
}

.home-usage-bar {
  position: relative;
  flex: 1;
  min-width: 0.22rem;
  border: 0;
  padding: 0;
  border-radius: 4px 4px 2px 2px;
  background: linear-gradient(
    180deg,
    var(--color-accent-primary) 0%,
    rgb(var(--color-accent-secondary-rgb) / 72%) 100%
  );
  opacity: 0.7;
  cursor: pointer;
  transition:
    opacity var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.home-usage-bar:hover,
.home-usage-bar[data-active='true'] {
  opacity: 1;
}

.home-usage-bar:focus-visible {
  outline: 0;
  box-shadow: 0 0 0 2px rgb(var(--color-accent-primary-rgb) / 56%);
  opacity: 1;
}

.home-usage__empty {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-height: 10rem;
  padding: 1rem;
  border: 1px dashed var(--home-border-hairline);
  border-radius: 10px;
}

.home-usage__empty-icon {
  display: grid;
  place-items: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 999px;
  background: rgb(var(--color-bg-surface-rgb) / 70%);
  color: var(--color-text-muted);
}

.home-usage__empty h3 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: var(--home-text-body);
  font-weight: 600;
}

.home-usage__empty p {
  margin: 0.15rem 0 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  line-height: var(--home-leading-body);
}

.home-usage__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding-top: 0.4rem;
  border-top: 1px solid var(--home-border-hairline);
}

.home-usage__last {
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-usage__report-link {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-decoration: none;
  text-transform: uppercase;
  transition: color var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.home-usage__report-link:hover,
.home-usage__report-link:focus-visible {
  color: var(--color-accent-primary);
  transform: translateX(2px);
  outline: 0;
}

@media (width <= 960px) {
  .home-usage__header,
  .home-usage__footer {
    flex-direction: column;
    align-items: flex-start;
  }

  .home-usage__body {
    grid-template-columns: 1fr;
  }

  .home-usage__metric {
    justify-self: start;
  }
}

@media (prefers-reduced-motion: reduce) {
  .home-usage-bar,
  .home-usage-range-btn,
  .home-usage-metric-btn,
  .home-usage__report-link {
    transition: none;
  }
}
</style>
