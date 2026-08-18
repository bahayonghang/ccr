<template>
  <section
    class="dashboard-usage"
    data-dashboard-usage-movement
  >
    <header class="dashboard-usage__header">
      <div class="dashboard-usage__lede">
        <h2 class="dashboard-usage__title">
          {{ t('dashboard.usage.title') }}
        </h2>
        <p class="dashboard-usage__description">
          {{ snapshotDescription }}
        </p>
      </div>

      <PillToggleGroup
        :options="rangeOptions"
        :model-value="activeDays"
        v-bind="{ ariaLabel: t('dashboard.usage.rangeLabel') }"
        @update:model-value="emit('change-days', $event)"
      />
    </header>

    <div class="dashboard-usage__body">
      <div class="dashboard-usage__summary">
        <StatTile
          v-for="item in summaryItems"
          :key="item.label"
          :label="item.label"
          :value="item.value"
          tone="neutral"
        />
      </div>

      <div class="dashboard-usage__chartArea">
        <PillToggleGroup
          class="dashboard-usage__metric"
          :options="metricToggleOptions"
          :model-value="selectedMetric"
          v-bind="{ ariaLabel: t('dashboard.usage.metricSelectLabel') }"
          @update:model-value="selectedMetric = $event"
        />

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
            <template v-else-if="peakPoint">
              <span class="dashboard-usage__chart-readout-metric">{{ t('dashboard.usage.peakLabel') }}</span>
              <span class="dashboard-usage__chart-readout-value">{{ peakPoint.valueLabel }}</span>
              <span class="dashboard-usage__chart-readout-date">{{ peakPoint.dateLabel }}</span>
            </template>
            <template v-else>
              <span class="dashboard-usage__chart-readout-placeholder">
                {{ t('dashboard.usage.hoverHint') }}
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
import PillToggleGroup from '@/components/ui/PillToggleGroup.vue'
import SIcon from '@/components/ui/SIcon.vue'
import StatTile from '@/components/ui/StatTile.vue'
import type { HomeUsageOverviewResponse } from '@/types/usage'
import type { DashboardUsageMetric } from '@/views/dashboard/dashboardPresentation'

const props = defineProps<{
  overview: HomeUsageOverviewResponse | null
  loading: boolean
  error: string | null
  activeDays: number
}>()

const emit = defineEmits<{
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

const rangeOptions = computed(() => dayOptions.map((days) => ({
  value: days,
  label: t(`dashboard.usage.range${days}`),
})))

const metricToggleOptions = computed(() => metricOptions.map((metric) => ({
  value: metric,
  label: getMetricLabel(metric),
})))

const isInitialLoading = computed(() => props.loading && !props.overview)

const snapshotDescription = computed(() => {
  if (props.error) return t('dashboard.usage.error')
  if (props.overview?.empty_reason) return getEmptyReasonDescription()
  if (isInitialLoading.value) return t('dashboard.metrics.usagePreparing')
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
    + item.antigravity[selectedMetric.value]
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

// 未 hover 时用峰值取代通用说明文案，既避免与卡片描述重复，又替用户先标出重点
const peakPoint = computed(() => {
  const points = chartPoints.value
  if (points.length === 0) return null

  const best = points.reduce((max, point) => (point.value > max.value ? point : max), points[0])
  return best.value > 0 ? best : null
})

const emptyTitle = computed(() => {
  if (props.error) return t('dashboard.usage.unavailableTitle')
  if (isInitialLoading.value) return t('dashboard.metrics.usagePreparing')
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
  if (props.overview?.empty_reason) return getEmptyReasonDescription()
  if (isInitialLoading.value) return t('dashboard.usage.loadingDescription')
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
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--home-card-radius);
  background: var(--color-bg-surface);
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

.dashboard-usage__title {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1.0625rem;
  font-weight: 600;
  line-height: 1.3;
  letter-spacing: 0;
}

.dashboard-usage__description {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  line-height: 1.5;
}

.dashboard-usage__body {
  display: grid;
  grid-template-columns: minmax(0, 3.5fr) minmax(0, 8.5fr);
  gap: 0.85rem;
}

.dashboard-usage__summary {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem 1rem;
  align-content: start;
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
  border: 1px solid var(--color-border-subtle);
  border-radius: 8px;
  background: var(--color-bg-elevated);
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
  font-size: 0.75rem;
  font-weight: 500;
  letter-spacing: 0;
  pointer-events: none;
}

.dashboard-usage__chart-readout-placeholder {
  overflow: hidden;
  color: var(--color-text-disabled);
  font-weight: 400;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-usage__chart-readout-value {
  color: var(--color-text-primary);
  font-size: 0.8125rem;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
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
  border-radius: 2px 2px 0 0;
  background: rgb(var(--color-text-muted-rgb) / 28%);
  cursor: pointer;
  padding: 0;
  transition:
    background-color var(--home-motion-duration) var(--home-motion-ease),
    opacity var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-usage-bar:hover,
.dashboard-usage-bar[data-active='true'] {
  background: rgb(var(--color-text-secondary-rgb) / 42%);
}

.dashboard-usage-bar:focus-visible {
  outline: 2px solid var(--color-accent-primary);
  outline-offset: 1px;
}

.dashboard-usage__empty {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-height: 10rem;
  padding: 1rem;
  border: 1px dashed var(--color-border-subtle);
  border-radius: 8px;
}

.dashboard-usage__empty-icon {
  display: grid;
  place-items: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 8px;
  background: var(--color-bg-elevated);
  color: var(--color-text-muted);
}

.dashboard-usage__empty h3 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.dashboard-usage__empty p {
  margin: 0.15rem 0 0;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  line-height: 1.5;
}

.dashboard-usage__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding-top: 0.4rem;
  border-top: 1px solid var(--color-border-subtle);
}

.dashboard-usage__last {
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  font-weight: 500;
  letter-spacing: 0;
}

.dashboard-usage__report-link {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--color-text-secondary);
  font-size: 0.8125rem;
  font-weight: 500;
  letter-spacing: 0;
  text-decoration: none;
}

.dashboard-usage__report-link:hover,
.dashboard-usage__report-link:focus-visible {
  color: var(--color-text-primary);
  outline: 0;
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
  .dashboard-usage__report-link {
    transition: none;
  }
}
</style>
