<template>
  <section class="tokens-tab">
    <article class="tokens-tab__chart-card glass-panel">
      <div class="tokens-tab__chart-head">
        <div>
          <p class="tokens-tab__eyebrow">
            {{ $t('usage.dashboard.tokens.eyebrow') }}
          </p>
          <h3>{{ $t('usage.dashboard.tokens.title') }}</h3>
          <p>
            {{ $t('usage.dashboard.tokens.subtitle') }}
          </p>
        </div>

        <div
          class="tokens-tab__mode"
          role="tablist"
        >
          <button
            v-for="mode in modes"
            :key="mode"
            type="button"
            :aria-selected="activeMode === mode"
            :class="{ 'tokens-tab__mode-button--active': activeMode === mode }"
            class="tokens-tab__mode-button"
            @click="activeMode = mode"
          >
            {{ $t(`usage.dashboard.tokens.modes.${mode}`) }}
          </button>
        </div>
      </div>

      <div class="tokens-tab__totals">
        <article
          v-for="item in totalItems"
          :key="item.key"
          class="tokens-tab__total"
        >
          <span
            class="tokens-tab__swatch"
            :style="{ background: item.color }"
          />
          <span>{{ item.label }}</span>
          <strong>{{ ctx.formatTokens(item.value) }}</strong>
        </article>
      </div>

      <component
        :is="ctx.chartComponent"
        v-if="hasRows"
        :key="activeMode"
        type="bar"
        height="320"
        :options="chartOptions"
        :series="chartSeries"
      />

      <div
        v-else
        class="tokens-tab__empty"
      >
        {{ $t('usage.dashboard.table.noData') }}
      </div>
    </article>

    <article class="tokens-tab__table-card glass-panel">
      <div class="tokens-tab__table-head">
        <div>
          <p class="tokens-tab__eyebrow">
            {{ $t('usage.dashboard.tokens.tableEyebrow') }}
          </p>
          <h3>{{ $t('usage.dashboard.tokens.tableTitle') }}</h3>
        </div>
        <span>{{ rows.length.toLocaleString() }} {{ $t('usage.dashboard.tokens.days') }}</span>
      </div>

      <div
        v-if="hasRows"
        class="tokens-tab__table-shell"
      >
        <table class="tokens-tab__table">
          <colgroup>
            <col class="tokens-tab__col tokens-tab__col--date">
            <col class="tokens-tab__col">
            <col class="tokens-tab__col">
            <col class="tokens-tab__col">
            <col class="tokens-tab__col">
            <col class="tokens-tab__col">
            <col class="tokens-tab__col">
          </colgroup>
          <thead>
            <tr>
              <th>{{ $t('usage.dashboard.tokens.date') }}</th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.input') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.assistantOutput') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.cacheRead') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.cacheWrite') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.reasoning') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.tokens') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="row in rows"
              :key="row.date"
            >
              <td>{{ row.date }}</td>
              <td class="is-right">
                {{ ctx.formatTokens(row.inputTokens) }}
              </td>
              <td class="is-right">
                {{ ctx.formatTokens(row.assistantOutputTokens) }}
              </td>
              <td class="is-right">
                {{ ctx.formatTokens(row.cacheReadTokens) }}
              </td>
              <td class="is-right">
                {{ ctx.formatTokens(row.cacheCreationTokens) }}
              </td>
              <td class="is-right">
                {{ ctx.formatTokens(row.reasoningOutputTokens) }}
              </td>
              <td class="is-right tokens-tab__total-cell">
                {{ ctx.formatTokens(row.totalTokens) }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div
        v-else
        class="tokens-tab__empty"
      >
        {{ $t('usage.dashboard.table.noData') }}
      </div>
    </article>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUsageDashboardContext } from '@/views/usage/usageDashboardContext'
import {
  buildChartAnimations,
  buildChartTheme,
  type ChartThemeState,
} from '@/views/usage/usageChartOptions'
import {
  getUsageTokenRowChartTotal,
  sumUsageTokenBreakdownRows,
  toUsageTokenBreakdownRows,
  type UsageTokenBreakdownMode,
  type UsageTokenSeriesKey,
} from '@/views/usage/usageTokenBreakdown'

const ctx = useUsageDashboardContext()

const { t } = useI18n()
const modes: UsageTokenBreakdownMode[] = ['breakdown', 'total']
const activeMode = ref<UsageTokenBreakdownMode>('breakdown')

const rows = computed(() => toUsageTokenBreakdownRows(ctx.trends))
const totals = computed(() => sumUsageTokenBreakdownRows(rows.value))
const hasRows = computed(() => rows.value.length > 0)
const theme = computed<ChartThemeState>(() => buildChartTheme())

const seriesMeta = computed<Array<{ key: UsageTokenSeriesKey; label: string; color: string }>>(() => [
  {
    key: 'input',
    label: tLabel('usage.dashboard.chart.input', 'Input'),
    color: theme.value.inputToken,
  },
  {
    key: 'assistantOutput',
    label: tLabel('usage.dashboard.chart.assistantOutput', 'Assistant Output'),
    color: theme.value.outputToken,
  },
  {
    key: 'cacheRead',
    label: tLabel('usage.dashboard.chart.cacheRead', 'Cache Read'),
    color: theme.value.cacheReadToken,
  },
  {
    key: 'cacheCreation',
    label: tLabel('usage.dashboard.chart.cacheWrite', 'Cache Write'),
    color: theme.value.cacheCreationToken,
  },
  {
    key: 'reasoning',
    label: tLabel('usage.dashboard.chart.reasoning', 'Reasoning'),
    color: theme.value.reasoningToken,
  },
])

const totalItems = computed(() => [
  { ...seriesMeta.value[0], value: totals.value.inputTokens },
  { ...seriesMeta.value[1], value: totals.value.assistantOutputTokens },
  { ...seriesMeta.value[2], value: totals.value.cacheReadTokens },
  { ...seriesMeta.value[3], value: totals.value.cacheCreationTokens },
  { ...seriesMeta.value[4], value: totals.value.reasoningOutputTokens },
])

const categories = computed(() => rows.value.map((row) => row.date))

const chartSeries = computed(() => {
  if (activeMode.value === 'total') {
    return [
      {
        name: tLabel('usage.dashboard.tokens.totalSeries', 'Total Tokens'),
        data: rows.value.map((row) => getUsageTokenRowChartTotal(row)),
      },
    ]
  }

  return seriesMeta.value.map((item) => ({
    name: item.label,
    data: rows.value.map((row) => {
      switch (item.key) {
        case 'input':
          return row.inputTokens
        case 'assistantOutput':
          return row.assistantOutputTokens
        case 'cacheRead':
          return row.cacheReadTokens
        case 'cacheCreation':
          return row.cacheCreationTokens
        case 'reasoning':
          return row.reasoningOutputTokens
      }
    }),
  }))
})

const chartColors = computed(() =>
  activeMode.value === 'total'
    ? [theme.value.primary]
    : seriesMeta.value.map((item) => item.color)
)

const chartOptions = computed(() => ({
  chart: {
    background: 'transparent',
    fontFamily: 'inherit',
    stacked: activeMode.value === 'breakdown',
    toolbar: { show: false },
    animations: buildChartAnimations(),
    // KeepAlive 重挂会触发 ApexCharts 的 parentResize 观察器,默认会全量 update 重建
    // canvas,抵消缓存收益;与 usageChartOptions.ts 的 TREND/PIE 基座保持一致地关闭。
    redrawOnParentResize: false,
    redrawOnWindowResize: false,
  },
  theme: { mode: theme.value.mode },
  colors: chartColors.value,
  plotOptions: {
    bar: {
      borderRadius: activeMode.value === 'breakdown' ? 3 : 6,
      columnWidth: '56%',
    },
  },
  dataLabels: { enabled: false },
  fill: { opacity: activeMode.value === 'breakdown' ? 0.76 : 0.82 },
  grid: {
    borderColor: theme.value.grid,
    strokeDashArray: 4,
    padding: { left: 6, right: 8, bottom: 4, top: 8 },
  },
  legend: {
    show: true,
    position: 'top' as const,
    horizontalAlign: 'right' as const,
    labels: { colors: theme.value.textSecondary },
    markers: { strokeWidth: 0 },
  },
  tooltip: {
    theme: theme.value.mode,
    shared: true,
    intersect: false,
    y: {
      formatter: (value: number) => ctx.formatTokens(value),
    },
  },
  xaxis: {
    categories: categories.value,
    tickAmount: categories.value.length > 18 ? 8 : undefined,
    labels: {
      rotate: 0,
      trim: true,
      style: { colors: theme.value.textMuted, fontSize: '11px' },
    },
    axisBorder: { show: false },
    axisTicks: { show: false },
  },
  yaxis: {
    labels: {
      style: { colors: theme.value.textMuted, fontSize: '11px' },
      formatter: (value: number) => ctx.formatTokens(value),
    },
  },
}))

function tLabel(key: string, fallback: string) {
  const label = t(key)
  return label === key ? fallback : label
}
</script>

<style scoped>
.tokens-tab {
  display: grid;
  gap: 1rem;
}

.tokens-tab__chart-card,
.tokens-tab__table-card {
  display: grid;
  gap: 1rem;
  overflow: hidden;
  border-radius: 1.6rem;
  padding: 1rem;
}

.tokens-tab__chart-head,
.tokens-tab__table-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.tokens-tab__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 760;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.tokens-tab h3 {
  margin-top: 0.18rem;
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 730;
  letter-spacing: -0.02em;
}

.tokens-tab__chart-head p:not(.tokens-tab__eyebrow) {
  margin-top: 0.34rem;
  max-width: 48rem;
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.6;
}

.tokens-tab__mode {
  display: inline-flex;
  flex: none;
  gap: 0.24rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 15%);
  border-radius: 999px;
  background: rgb(var(--color-bg-elevated-rgb) / 54%);
  padding: 0.22rem;
}

.tokens-tab__mode-button {
  border-radius: 999px;
  padding: 0.38rem 0.72rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  font-weight: 700;
  transition:
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.tokens-tab__mode-button--active {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: var(--color-text-primary);
}

.tokens-tab__totals {
  display: grid;
  gap: 0.62rem;
  grid-template-columns: repeat(5, minmax(0, 1fr));
}

.tokens-tab__total {
  display: grid;
  gap: 0.26rem;
  min-width: 0;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 1rem;
  background: rgb(var(--color-bg-elevated-rgb) / 38%);
  padding: 0.72rem 0.8rem;
}

.tokens-tab__total span:not(.tokens-tab__swatch) {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 740;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.tokens-tab__total strong {
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 720;
  font-variant-numeric: tabular-nums;
}

.tokens-tab__swatch {
  width: 1.35rem;
  height: 0.18rem;
  border-radius: 999px;
}

.tokens-tab__table-head span {
  flex: none;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 999px;
  padding: 0.26rem 0.55rem;
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 690;
}

.tokens-tab__table-shell {
  max-height: 34rem;
  overflow: auto;
  border: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  border-radius: 1.15rem;
  background: rgb(var(--color-bg-elevated-rgb) / 42%);
}

.tokens-tab__table {
  min-width: 62rem;
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
}

.tokens-tab__table thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: 0.78rem 0.9rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 94%);
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 730;
  letter-spacing: 0.08em;
  text-align: left;
  text-transform: uppercase;
}

.tokens-tab__table tbody td {
  padding: 0.82rem 0.9rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 11%);
  color: var(--color-text-secondary);
  font-size: 0.84rem;
  font-variant-numeric: tabular-nums;
}

.tokens-tab__table tbody tr:hover {
  background: rgb(var(--color-accent-primary-rgb) / 5%);
}

.tokens-tab__table .is-right {
  text-align: right;
}

.tokens-tab__total-cell {
  color: var(--color-text-primary);
  font-weight: 720;
}

.tokens-tab__col {
  width: 9rem;
}

.tokens-tab__col--date {
  width: 10rem;
}

.tokens-tab__empty {
  display: grid;
  min-height: 18rem;
  place-items: center;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 18%);
  border-radius: 1.1rem;
  color: var(--color-text-muted);
  font-size: 0.86rem;
}

@media (width < 980px) {
  .tokens-tab__chart-head,
  .tokens-tab__table-head {
    flex-direction: column;
  }

  .tokens-tab__mode {
    width: 100%;
  }

  .tokens-tab__mode-button {
    flex: 1;
  }

  .tokens-tab__totals {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
