<template>
  <section
    class="dashboard-usage"
    data-dashboard-usage-movement
  >
    <header class="dashboard-usage__header">
      <div class="dashboard-usage__lede">
        <p class="dashboard-usage__eyebrow">
          {{ t('dashboard.usage.eyebrow') }}
        </p>
        <h2 class="dashboard-usage__title">
          {{ t('dashboard.usage.title') }}
        </h2>
        <p class="dashboard-usage__description">
          {{ snapshotDescription }}
        </p>
      </div>

      <div
        class="dashboard-usage__range"
        role="group"
        :aria-label="t('dashboard.usage.rangeLabel')"
      >
        <button
          v-for="days in dayOptions"
          :key="days"
          type="button"
          class="dashboard-usage-range-btn"
          :data-active="activeDays === days ? 'true' : 'false'"
          :aria-pressed="activeDays === days"
          @click="$emit('change-days', days)"
        >
          {{ t(`dashboard.usage.range${days}`) }}
        </button>
      </div>
    </header>

    <div class="dashboard-usage__body">
      <div class="dashboard-usage__summary">
        <div
          v-for="item in summaryItems"
          :key="item.label"
          class="dashboard-usage-summary"
        >
          <span class="dashboard-usage-summary__label">{{ item.label }}</span>
          <strong class="dashboard-usage-summary__value">{{ item.value }}</strong>
        </div>
      </div>

      <div class="dashboard-usage__chartArea">
        <div
          class="dashboard-usage__metric"
          role="group"
          :aria-label="t('dashboard.usage.metricSelectLabel')"
        >
          <button
            v-for="metric in metricOptions"
            :key="metric"
            type="button"
            class="dashboard-usage-metric-btn"
            :data-active="selectedMetric === metric ? 'true' : 'false'"
            :aria-pressed="selectedMetric === metric"
            @click="selectedMetric = metric"
          >
            {{ getMetricLabel(metric) }}
          </button>
        </div>

        <div
          v-if="hasSeries"
          class="dashboard-usage__chart"
          :class="{ 'dashboard-usage__chart--ghost': !hasMeaningfulSeries }"
          data-dashboard-usage-bars
          @mouseleave="hoveredKey = null"
        >
          <span
            class="dashboard-usage__chart-readout"
            :data-visible="hoveredPoint ? 'true' : 'false'"
            aria-live="polite"
          >
            <template v-if="hoveredPoint">
              <span class="dashboard-usage__chart-readout-date">{{ hoveredPoint.dateLabel }}</span>
              <span class="dashboard-usage__chart-readout-value">{{ hoveredPoint.valueLabel }}</span>
              <span class="dashboard-usage__chart-readout-metric">{{ getMetricLabel(selectedMetric) }}</span>
            </template>
            <template v-else>
              <span class="dashboard-usage__chart-readout-placeholder">
                {{ t('dashboard.usage.description') }}
              </span>
            </template>
          </span>

          <div
            class="dashboard-usage__chart-grid"
            aria-hidden="true"
          >
            <span class="dashboard-usage__chart-grid-line dashboard-usage__chart-grid-line--top" />
            <span class="dashboard-usage__chart-grid-line dashboard-usage__chart-grid-line--bottom" />
          </div>

          <div class="dashboard-usage__chart-bars">
            <button
              v-for="point in chartPoints"
              :key="point.key"
              type="button"
              class="dashboard-usage-bar"
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
          class="dashboard-usage__empty"
        >
          <span class="dashboard-usage__empty-icon">
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

    <footer class="dashboard-usage__footer">
      <span class="dashboard-usage__last">{{ lastUpdatedLabel }}</span>
      <RouterLink
        to="/usage"
        class="dashboard-usage__report-link"
      >
        {{ t('dashboard.usage.fullReport') }}
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
import type { DashboardUsageMetric } from '@/views/dashboard/dashboardPresentation'

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
const metricOptions: DashboardUsageMetric[] = ['sessions', 'requests', 'tokens']
const selectedMetric = ref<DashboardUsageMetric>('requests')
const hoveredKey = ref<string | null>(null)

const formatCompact = (value?: number) => {
  if (typeof value !== 'number') return '…'
  return new Intl.NumberFormat(undefined, {
    notation: 'compact',
    maximumFractionDigits: 1,
  }).format(value)
}

const formatDateTime = (value?: string) => {
  if (!value) return t('dashboard.usage.lastUpdatedNever')
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return t('dashboard.usage.lastUpdatedNever')

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

const getMetricLabel = (metric: DashboardUsageMetric) => {
  switch (metric) {
    case 'sessions':
      return t('dashboard.usage.metricSessions')
    case 'tokens':
      return t('dashboard.usage.metricTokens')
    default:
      return t('dashboard.usage.metricRequests')
  }
}

const snapshotDescription = computed(() => {
  if (props.error) return t('dashboard.usage.error')
  if (props.loading) return t('dashboard.metrics.usagePreparing')
  if (props.overview?.empty_reason) return getEmptyReasonDescription()
  return t('dashboard.usage.description')
})

const summaryItems = computed(() => [
  {
    label: t('dashboard.usage.metricSessions'),
    value: formatCompact(props.overview?.summary.total_sessions),
  },
  {
    label: t('dashboard.usage.metricRequests'),
    value: formatCompact(props.overview?.summary.total_requests),
  },
  {
    label: t('dashboard.usage.metricTokens'),
    value: formatCompact(props.overview?.summary.total_tokens),
  },
  {
    label: t('dashboard.usage.metricPlatforms'),
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
  if (props.error) return t('dashboard.usage.unavailableTitle')
  if (props.loading) return t('dashboard.metrics.usagePreparing')
  return t('dashboard.usage.emptyTitle')
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
      return t('dashboard.usage.emptyDescription')
  }
}

const emptyDescription = computed(() => {
  if (props.error) return props.error
  if (props.loading) return t('dashboard.usage.loadingDescription')
  return getEmptyReasonDescription()
})

const lastUpdatedLabel = computed(() => (
  translateWithFallback(
    t,
    'dashboard.usage.lastUpdated',
    'Updated {time}',
    { time: formatDateTime(props.overview?.last_updated) },
  )
))
</script>

<style scoped>
.dashboard-usage {
  display: grid;
  gap: 0.85rem;
  height: 100%;
  padding: var(--home-card-pad);
  border: 1px solid var(--home-border-card);
  border-radius: var(--home-card-radius);
  background: var(--home-surface-card);
  box-shadow: var(--home-elevation-raised);
}

.dashboard-usage__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.dashboard-usage__lede {
  display: grid;
  gap: 0.2rem;
  min-width: 0;
}

.dashboard-usage__eyebrow {
  margin: 0;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-usage__title {
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: var(--home-text-section);
  font-weight: 640;
  letter-spacing: var(--home-tracking-display);
}

.dashboard-usage__description {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-body);
  line-height: var(--home-leading-body);
}

.dashboard-usage__range,
.dashboard-usage__metric {
  display: inline-flex;
  gap: 0.2rem;
  padding: 0.18rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 999px;
  background: var(--home-surface-sunk);
  box-shadow: var(--home-elevation-sunk);
}

.dashboard-usage-range-btn,
.dashboard-usage-metric-btn {
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  padding: 0.3rem 0.68rem;
  text-transform: uppercase;
  transition:
    background-color var(--home-motion-duration) var(--home-motion-ease),
    color var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-usage-range-btn:hover,
.dashboard-usage-metric-btn:hover {
  color: var(--color-text-primary);
}

.dashboard-usage-range-btn:focus-visible,
.dashboard-usage-metric-btn:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.dashboard-usage-range-btn[data-active='true'] {
  background: var(--color-accent-primary);
  color: var(--color-text-inverted);
}

.dashboard-usage-metric-btn[data-active='true'] {
  background: rgb(var(--color-accent-primary-rgb) / 14%);
  color: var(--color-accent-primary);
}

.dashboard-usage__body {
  display: grid;
  grid-template-columns: minmax(0, 3.5fr) minmax(0, 8.5fr);
  gap: 0.85rem;
}

.dashboard-usage__summary {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.55rem;
  align-content: start;
}

.dashboard-usage-summary {
  display: grid;
  gap: 0.16rem;
  min-width: 0;
  padding: 0.68rem 0.76rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 58%);
}

.dashboard-usage-summary__label {
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-usage-summary__value {
  overflow: hidden;
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono-lg);
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-usage__chartArea {
  display: grid;
  grid-template-rows: auto 1fr;
  gap: 0.55rem;
  min-width: 0;
}

.dashboard-usage__metric {
  justify-self: end;
}

.dashboard-usage__chart {
  position: relative;
  display: flex;
  align-items: end;
  gap: 0.22rem;
  min-height: 10rem;
  padding: 1.5rem 0.75rem 0.5rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 46%);
  overflow: hidden;
}

.dashboard-usage__chart--ghost .dashboard-usage-bar {
  opacity: 0.34;
}

.dashboard-usage__chart-readout {
  position: absolute;
  top: 0.4rem;
  left: 0.75rem;
  right: 0.75rem;
  display: flex;
  align-items: baseline;
  gap: 0.45rem;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
  pointer-events: none;
}

.dashboard-usage__chart-readout-placeholder {
  overflow: hidden;
  color: var(--color-text-disabled);
  font-weight: 400;
  letter-spacing: 0;
  text-overflow: ellipsis;
  text-transform: none;
  white-space: nowrap;
}

.dashboard-usage__chart-readout-value {
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono);
}

.dashboard-usage__chart-grid {
  position: absolute;
  inset: 1.5rem 0.75rem 0.5rem;
  pointer-events: none;
}

.dashboard-usage__chart-grid-line {
  position: absolute;
  left: 0;
  right: 0;
  height: 0;
  border-top: 1px dashed rgb(var(--color-border-default-rgb) / 16%);
}

.dashboard-usage__chart-grid-line--top { top: 0; }
.dashboard-usage__chart-grid-line--bottom { bottom: 0; }

.dashboard-usage__chart-bars {
  position: relative;
  display: flex;
  align-items: end;
  gap: 0.22rem;
  width: 100%;
  height: 100%;
  z-index: 1;
}

.dashboard-usage-bar {
  flex: 1;
  min-width: 0.22rem;
  border: 0;
  border-radius: 4px 4px 2px 2px;
  background: linear-gradient(
    180deg,
    var(--color-accent-primary) 0%,
    rgb(var(--color-accent-secondary-rgb) / 72%) 100%
  );
  cursor: pointer;
  opacity: 0.72;
  padding: 0;
  transition:
    opacity var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-usage-bar:hover,
.dashboard-usage-bar[data-active='true'] {
  opacity: 1;
}

.dashboard-usage-bar:focus-visible {
  outline: 0;
  box-shadow: 0 0 0 2px rgb(var(--color-accent-primary-rgb) / 56%);
  opacity: 1;
}

.dashboard-usage__empty {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-height: 10rem;
  padding: 1rem;
  border: 1px dashed var(--home-border-hairline);
  border-radius: 10px;
}

.dashboard-usage__empty-icon {
  display: grid;
  place-items: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 999px;
  background: rgb(var(--color-bg-surface-rgb) / 70%);
  color: var(--color-text-muted);
}

.dashboard-usage__empty h3 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: var(--home-text-body);
  font-weight: 650;
}

.dashboard-usage__empty p {
  margin: 0.15rem 0 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  line-height: var(--home-leading-body);
}

.dashboard-usage__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding-top: 0.4rem;
  border-top: 1px solid var(--home-border-hairline);
}

.dashboard-usage__last {
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-usage__report-link {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-decoration: none;
  text-transform: uppercase;
  transition:
    color var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-usage__report-link:hover,
.dashboard-usage__report-link:focus-visible {
  color: var(--color-accent-primary);
  outline: 0;
  transform: translateX(2px);
}

@media (width <= 960px) {
  .dashboard-usage__header,
  .dashboard-usage__footer {
    flex-direction: column;
    align-items: flex-start;
  }

  .dashboard-usage__body {
    grid-template-columns: 1fr;
  }

  .dashboard-usage__metric {
    justify-self: start;
  }
}

@media (prefers-reduced-motion: reduce) {
  .dashboard-usage-bar,
  .dashboard-usage-range-btn,
  .dashboard-usage-metric-btn,
  .dashboard-usage__report-link {
    transition: none;
  }
}
</style>

