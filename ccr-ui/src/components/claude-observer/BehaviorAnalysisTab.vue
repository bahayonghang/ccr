<template>
  <div class="behavior-tab">
    <!-- 步骤1：来源差异提示 -->
    <p class="behavior-tab__source-note">
      {{ $t('claudeCode.observer.behavior.sourceNote') }}
    </p>

    <!-- 步骤2：左右两列 —— 热力图 + Top tools -->
    <section class="behavior-tab__grid">
      <article class="behavior-tab__card behavior-tab__heatmap-card">
        <header class="behavior-tab__head">
          <p class="behavior-tab__eyebrow">
            {{ $t('claudeCode.observer.chart.toolHeatmap') }}
          </p>
          <p class="behavior-tab__sub">
            {{ $t('claudeCode.observer.chart.toolHeatmapSub') }}
          </p>
        </header>

        <div class="behavior-tab__chart-shell">
          <component
            :is="apexchart"
            v-if="hasHeatmap && shouldRenderChart"
            class="behavior-tab__chart"
            type="heatmap"
            height="260"
            :options="heatmapOptions"
            :series="heatmapSeries"
          />
          <ChartPreparingState
            v-else-if="hasHeatmap"
            :label="$t('claudeCode.observer.chart.preparingHeatmap')"
          />
          <div
            v-else
            class="behavior-tab__empty"
          >
            {{ $t('claudeCode.observer.empty.noTrend') }}
          </div>
        </div>
      </article>

      <article class="behavior-tab__card behavior-tab__tools-card">
        <header class="behavior-tab__head">
          <p class="behavior-tab__eyebrow">
            {{ $t('claudeCode.observer.chart.topTools') }}
          </p>
          <p class="behavior-tab__sub">
            {{ $t('claudeCode.observer.chart.topToolsSub') }}
          </p>
        </header>

        <ol
          v-if="topToolRows.length > 0"
          class="behavior-tab__rank-list"
        >
          <li
            v-for="(row, index) in topToolRows"
            :key="`tool-${row.tool_name}-${index}`"
            class="behavior-tab__rank-row"
          >
            <span
              class="behavior-tab__rank-label"
              :title="row.tool_name"
            >{{ row.tool_name }}</span>
            <span class="behavior-tab__rank-bar">
              <span
                class="behavior-tab__rank-bar-fill"
                :style="{ width: `${barPercent(row.call_count, toolMax)}%` }"
              />
            </span>
            <span class="behavior-tab__rank-value">{{ row.call_count.toLocaleString() }}</span>
          </li>
        </ol>
        <div
          v-else
          class="behavior-tab__empty behavior-tab__empty--compact"
        >
          {{ $t('claudeCode.observer.empty.noTrend') }}
        </div>
      </article>
    </section>

    <!-- 步骤3：cost-efficiency 表格 -->
    <section class="behavior-tab__card">
      <header class="behavior-tab__head">
        <p class="behavior-tab__eyebrow">
          {{ $t('claudeCode.observer.behavior.efficiencyTitle') }}
        </p>
        <p class="behavior-tab__sub">
          {{ $t('claudeCode.observer.behavior.efficiencySub') }}
        </p>
      </header>

      <div
        v-if="sessionRows.length > 0"
        class="behavior-tab__table-wrapper"
      >
        <table class="behavior-tab__table">
          <thead>
            <tr>
              <th>{{ $t('claudeCode.observer.behavior.colSession') }}</th>
              <th>{{ $t('claudeCode.observer.behavior.colProject') }}</th>
              <th class="behavior-tab__table-num">
                {{ $t('claudeCode.observer.behavior.colCost') }}
              </th>
              <th class="behavior-tab__table-num">
                {{ $t('claudeCode.observer.behavior.colTools') }}
              </th>
              <th class="behavior-tab__table-num">
                {{ $t('claudeCode.observer.behavior.colCostPerTool') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="row in sessionRows"
              :key="row.session_id"
            >
              <td>
                <span
                  class="behavior-tab__mono"
                  :title="row.session_id"
                >{{ shortenId(row.session_id) }}</span>
              </td>
              <td>
                <span :title="row.project_path ?? ''">
                  {{ shortenPath(row.project_path ?? '—') }}
                </span>
              </td>
              <td class="behavior-tab__table-num">
                {{ formatUsd(row.cost_usd) }}
              </td>
              <td class="behavior-tab__table-num">
                {{ row.tool_call_count.toLocaleString() }}
              </td>
              <td class="behavior-tab__table-num">
                {{ formatUsd(costPerTool(row)) }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div
        v-else
        class="behavior-tab__empty behavior-tab__empty--compact"
      >
        {{ $t('claudeCode.observer.empty.noTrend') }}
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ApexChartAsync as apexchart } from './apexChart'
import ChartPreparingState from './ChartPreparingState.vue'
import type { HeatmapCell, SessionRow, TopToolRow } from '@/types/claudeObserver'
import { buildChartTheme } from '@/views/usage/usageChartOptions'
import { formatUsd } from './formatters'

interface Props {
  heatmap: HeatmapCell[]
  topTools: TopToolRow[]
  sessions: SessionRow[]
  animationsEnabled: boolean
  shouldRenderChart: boolean
}

const props = defineProps<Props>()

const hasHeatmap = computed(() => props.heatmap.length > 0)

const topToolRows = computed(() => props.topTools.slice(0, 10))

const toolMax = computed(() =>
  topToolRows.value.reduce((max, row) => Math.max(max, row.call_count), 0),
)

const sessionRows = computed(() => props.sessions.slice(0, 10))

const barPercent = (value: number, max: number) => {
  if (max <= 0) return 0
  return Math.max(6, Math.round((value / max) * 100))
}

const costPerTool = (row: SessionRow) => {
  if (row.tool_call_count <= 0) return 0
  return row.cost_usd / row.tool_call_count
}

/*
 * ========================================================================
 * 热力图 series：每行一个 dow，列对应 0..23
 * ========================================================================
 */
const WEEK_LABELS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']

const heatmapSeries = computed(() => {
  // 1.1 先建立 dow -> hour -> count 的矩阵
  const matrix: Record<number, Record<number, number>> = {}
  for (const cell of props.heatmap) {
    if (!matrix[cell.dow]) matrix[cell.dow] = {}
    matrix[cell.dow][cell.hour] = (matrix[cell.dow][cell.hour] ?? 0) + cell.count
  }
  // 1.2 输出固定顺序：Mon..Sun（图表自上而下读）
  const order = [1, 2, 3, 4, 5, 6, 0]
  return order.map((dow) => ({
    name: WEEK_LABELS[dow],
    data: Array.from({ length: 24 }, (_, hour) => ({
      x: `${hour.toString().padStart(2, '0')}`,
      y: matrix[dow]?.[hour] ?? 0,
    })),
  }))
})

const heatmapOptions = computed(() => {
  const theme = buildChartTheme()
  return {
    chart: {
      type: 'heatmap' as const,
      background: 'transparent',
      toolbar: { show: false },
      fontFamily: 'inherit',
      parentHeightOffset: 0,
      redrawOnParentResize: true,
      animations: {
        enabled: props.animationsEnabled,
        speed: 220,
      },
    },
    theme: { mode: theme.mode },
    dataLabels: { enabled: false },
    plotOptions: {
      heatmap: {
        radius: 3,
        enableShades: false,
        colorScale: {
          ranges: [
            { from: 0, to: 0, color: 'rgba(125,151,182,0.08)', name: '0' },
            { from: 1, to: 4, color: 'rgba(125,151,182,0.28)', name: '1-4' },
            { from: 5, to: 14, color: 'rgba(125,151,182,0.5)', name: '5-14' },
            { from: 15, to: 49, color: 'rgba(125,151,182,0.72)', name: '15-49' },
            { from: 50, to: 100000, color: 'rgba(125,151,182,0.92)', name: '50+' },
          ],
        },
      },
    },
    grid: {
      borderColor: theme.grid,
      padding: { left: 4, right: 4 },
    },
    xaxis: {
      type: 'category' as const,
      labels: {
        style: { colors: theme.textMuted, fontSize: '10px' },
      },
      axisBorder: { show: false },
      axisTicks: { show: false },
    },
    yaxis: {
      labels: { style: { colors: theme.textMuted, fontSize: '11px' } },
    },
    tooltip: { theme: theme.mode },
    legend: {
      show: false,
    },
  }
})

const shortenId = (raw: string) => {
  if (!raw) return ''
  if (raw.length <= 12) return raw
  return `${raw.slice(0, 6)}…${raw.slice(-4)}`
}

const shortenPath = (raw: string) => {
  if (!raw) return ''
  if (raw.length <= 36) return raw
  const segments = raw.replace(/\\/g, '/').split('/').filter(Boolean)
  if (segments.length <= 2) return raw
  return `…/${segments.slice(-2).join('/')}`
}
</script>

<style scoped>
.behavior-tab {
  display: grid;
  gap: 0.9rem;
}

.behavior-tab__source-note {
  color: var(--color-text-muted);
  font-size: 0.75rem;
  line-height: 1.5;
  border-radius: 0.65rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 22%);
  background: rgb(var(--color-info-rgb) / 5%);
  padding: 0.55rem 0.75rem;
  margin: 0;
}

.behavior-tab__grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
}

.behavior-tab__card {
  border-radius: 1.1rem;
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--surface-card-shadow);
  padding: 1rem 1.1rem;
  min-width: 0;
}

.behavior-tab__head {
  display: grid;
  gap: 0.18rem;
  margin-bottom: 0.7rem;
}

.behavior-tab__eyebrow {
  color: var(--color-text-primary);
  font-size: 0.9rem;
  font-weight: 650;
}

.behavior-tab__sub {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  line-height: 1.4;
}

.behavior-tab__chart-shell {
  position: relative;
  height: 260px;
  min-width: 0;
}

.behavior-tab__chart {
  width: 100%;
  height: 100%;
}

.behavior-tab__chart :deep(svg) {
  width: 100% !important;
  height: 100% !important;
}

.behavior-tab__rank-list {
  display: grid;
  gap: 0.5rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.behavior-tab__rank-row {
  display: grid;
  grid-template-columns: minmax(0, 8rem) minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.55rem;
  font-size: 0.8rem;
}

.behavior-tab__rank-label {
  overflow: hidden;
  color: var(--color-text-primary);
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.behavior-tab__rank-bar {
  height: 0.42rem;
  border-radius: 9999px;
  background: rgb(var(--color-border-default-rgb) / 18%);
  overflow: hidden;
}

.behavior-tab__rank-bar-fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: rgb(var(--color-info-rgb) / 75%);
  transition: width var(--motion-standard-duration) var(--motion-standard-ease);
}

.behavior-tab__rank-value {
  color: var(--color-text-primary);
  font-variant-numeric: tabular-nums;
  font-weight: 650;
}

.behavior-tab__table-wrapper {
  width: 100%;
  overflow-x: auto;
}

.behavior-tab__table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8rem;
}

.behavior-tab__table th,
.behavior-tab__table td {
  padding: 0.5rem 0.7rem;
  text-align: left;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 16%);
}

.behavior-tab__table th {
  color: var(--color-text-muted);
  font-weight: 600;
  font-size: 0.72rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.behavior-tab__table td {
  color: var(--color-text-primary);
}

.behavior-tab__table tbody tr:hover {
  background: rgb(var(--color-accent-primary-rgb) / 4%);
}

.behavior-tab__table-num {
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.behavior-tab__mono {
  font-family: var(--font-mono);
  font-size: 0.78rem;
  color: var(--color-text-secondary);
}

.behavior-tab__empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  border-radius: 1rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 24%);
  color: var(--color-text-muted);
  font-size: 0.85rem;
}

.behavior-tab__empty--compact {
  min-height: 120px;
}

@media (width < 1100px) {
  .behavior-tab__grid {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (prefers-reduced-motion: reduce) {
  .behavior-tab__rank-bar-fill {
    transition: none;
  }
}
</style>
