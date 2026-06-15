<template>
  <div class="cost-tab">
    <!-- 步骤1：30 天每日费用面积图 -->
    <section class="cost-tab__chart-card">
      <header class="cost-tab__head">
        <p class="cost-tab__eyebrow">
          {{ $t('claudeCode.observer.chart.dailyTrend30') }}
        </p>
        <p class="cost-tab__sub">
          {{ $t('claudeCode.observer.chart.dailyTrend30Sub') }}
        </p>
      </header>

      <div class="cost-tab__chart-shell">
        <component
          :is="apexchart"
          v-if="hasDaily && shouldRenderChart"
          class="cost-tab__chart"
          type="area"
          height="260"
          :options="dailyOptions"
          :series="dailySeries"
        />
        <ChartPreparingState
          v-else-if="hasDaily"
          :label="$t('claudeCode.observer.chart.preparingTrend')"
        />
        <div
          v-else
          class="cost-tab__empty"
        >
          {{ $t('claudeCode.observer.empty.noTrend') }}
        </div>
      </div>
    </section>

    <!-- 步骤2：双列横向条形 —— 按 project / model 拆分 -->
    <section class="cost-tab__rank-grid">
      <article class="cost-tab__rank-card">
        <header class="cost-tab__head">
          <p class="cost-tab__eyebrow">
            {{ $t('claudeCode.observer.chart.byProject') }}
          </p>
          <p class="cost-tab__sub">
            {{ $t('claudeCode.observer.chart.byProjectSub') }}
          </p>
        </header>
        <ol
          v-if="projectRows.length > 0"
          class="cost-tab__rank-list"
        >
          <li
            v-for="(row, index) in projectRows"
            :key="`p-${row.key}-${index}`"
            class="cost-tab__rank-row"
          >
            <span
              class="cost-tab__rank-label"
              :title="row.key"
            >{{ shortenPath(row.key) }}</span>
            <span class="cost-tab__rank-bar">
              <span
                class="cost-tab__rank-bar-fill cost-tab__rank-bar-fill--primary"
                :style="{ width: `${barPercent(row.cost_usd, projectMax)}%` }"
              />
            </span>
            <span class="cost-tab__rank-value">{{ formatUsd(row.cost_usd) }}</span>
          </li>
        </ol>
        <div
          v-else
          class="cost-tab__empty cost-tab__empty--compact"
        >
          {{ $t('claudeCode.observer.empty.noTrend') }}
        </div>
      </article>

      <article class="cost-tab__rank-card">
        <header class="cost-tab__head">
          <p class="cost-tab__eyebrow">
            {{ $t('claudeCode.observer.chart.byModel') }}
          </p>
          <p class="cost-tab__sub">
            {{ $t('claudeCode.observer.chart.byModelSub') }}
          </p>
        </header>
        <ol
          v-if="modelRows.length > 0"
          class="cost-tab__rank-list"
        >
          <li
            v-for="(row, index) in modelRows"
            :key="`m-${row.key}-${index}`"
            class="cost-tab__rank-row"
          >
            <span
              class="cost-tab__rank-label"
              :title="row.key"
            >{{ row.key }}</span>
            <span class="cost-tab__rank-bar">
              <span
                class="cost-tab__rank-bar-fill cost-tab__rank-bar-fill--secondary"
                :style="{ width: `${barPercent(row.cost_usd, modelMax)}%` }"
              />
            </span>
            <span class="cost-tab__rank-value">{{ formatUsd(row.cost_usd) }}</span>
          </li>
        </ol>
        <div
          v-else
          class="cost-tab__empty cost-tab__empty--compact"
        >
          {{ $t('claudeCode.observer.empty.noTrend') }}
        </div>
      </article>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ApexChartAsync as apexchart } from './apexChart'
import ChartPreparingState from './ChartPreparingState.vue'
import type { BreakdownRow, DailyPoint } from '@/types/claudeObserver'
import { buildChartTheme } from '@/views/usage/usageChartOptions'
import { formatUsd } from './formatters'

/*
 * ========================================================================
 * Props
 * ========================================================================
 * 数据来自父级 store，组件本身保持纯展示
 */
interface Props {
  daily: DailyPoint[]
  byProject: BreakdownRow[]
  byModel: BreakdownRow[]
  animationsEnabled: boolean
  shouldRenderChart: boolean
}

const props = defineProps<Props>()

/*
 * ========================================================================
 * 衍生数据
 * ========================================================================
 */
const hasDaily = computed(() => props.daily.length > 0)

const projectRows = computed(() => props.byProject.slice(0, 10))
const modelRows = computed(() => props.byModel.slice(0, 10))

const projectMax = computed(() =>
  projectRows.value.reduce((max, row) => Math.max(max, row.cost_usd), 0),
)
const modelMax = computed(() =>
  modelRows.value.reduce((max, row) => Math.max(max, row.cost_usd), 0),
)

const barPercent = (value: number, max: number) => {
  if (max <= 0) return 0
  return Math.max(6, Math.round((value / max) * 100))
}

const dailySeries = computed(() => [
  {
    name: 'USD',
    data: props.daily.map((point) => ({ x: point.date, y: Number(point.cost_usd.toFixed(4)) })),
  },
])

const dailyOptions = computed(() => {
  const theme = buildChartTheme()
  return {
    chart: {
      background: 'transparent',
      toolbar: { show: false },
      fontFamily: 'inherit',
      parentHeightOffset: 0,
      redrawOnParentResize: true,
      redrawOnWindowResize: true,
      animations: {
        enabled: props.animationsEnabled,
        speed: 220,
        easing: 'easeinout',
      },
    },
    theme: { mode: theme.mode },
    colors: [theme.primary],
    stroke: { curve: 'smooth', width: 2 },
    fill: {
      type: 'gradient',
      gradient: {
        shadeIntensity: 1,
        opacityFrom: 0.32,
        opacityTo: 0.05,
        stops: [0, 100],
      },
    },
    dataLabels: { enabled: false },
    grid: {
      borderColor: theme.grid,
      strokeDashArray: 3,
      padding: { left: 12, right: 12 },
    },
    xaxis: {
      type: 'datetime' as const,
      labels: {
        style: { colors: theme.textMuted, fontSize: '11px' },
        datetimeUTC: false,
      },
      axisBorder: { show: false },
      axisTicks: { color: theme.grid },
    },
    yaxis: {
      labels: {
        style: { colors: theme.textMuted, fontSize: '11px' },
        formatter: (value: number) => `$${value.toFixed(2)}`,
      },
    },
    tooltip: {
      theme: theme.mode,
      x: { format: 'yyyy-MM-dd' },
      y: { formatter: (value: number) => `$${value.toFixed(4)}` },
    },
    legend: { show: false },
  }
})

/*
 * ========================================================================
 * 工具：格式化与路径裁剪
 * ========================================================================
 */
const shortenPath = (raw: string) => {
  if (!raw) return ''
  if (raw.length <= 42) return raw
  const segments = raw.replace(/\\/g, '/').split('/').filter(Boolean)
  if (segments.length <= 2) return raw
  return `…/${segments.slice(-2).join('/')}`
}
</script>

<style scoped>
.cost-tab {
  display: grid;
  gap: 0.9rem;
}

.cost-tab__chart-card,
.cost-tab__rank-card {
  border-radius: 1.1rem;
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--surface-card-shadow);
  padding: 1rem 1.1rem;
}

.cost-tab__head {
  display: grid;
  gap: 0.18rem;
  margin-bottom: 0.7rem;
}

.cost-tab__eyebrow {
  color: var(--color-text-primary);
  font-size: 0.9rem;
  font-weight: 650;
}

.cost-tab__sub {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  line-height: 1.4;
}

.cost-tab__chart-shell {
  position: relative;
  height: 260px;
  min-width: 0;
}

.cost-tab__chart {
  width: 100%;
  height: 100%;
}

.cost-tab__chart :deep(.apexcharts-canvas),
.cost-tab__chart :deep(svg) {
  width: 100% !important;
  height: 100% !important;
}

.cost-tab__rank-grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.cost-tab__rank-list {
  display: grid;
  gap: 0.55rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.cost-tab__rank-row {
  display: grid;
  grid-template-columns: minmax(0, 7rem) minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.55rem;
  font-size: 0.8rem;
}

.cost-tab__rank-label {
  overflow: hidden;
  color: var(--color-text-primary);
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cost-tab__rank-bar {
  height: 0.42rem;
  border-radius: 9999px;
  background: rgb(var(--color-border-default-rgb) / 18%);
  overflow: hidden;
}

.cost-tab__rank-bar-fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  transition: width var(--motion-standard-duration) var(--motion-standard-ease);
}

.cost-tab__rank-bar-fill--primary {
  background: rgb(var(--color-accent-primary-rgb) / 80%);
}

.cost-tab__rank-bar-fill--secondary {
  background: rgb(var(--color-accent-secondary-rgb) / 80%);
}

.cost-tab__rank-value {
  color: var(--color-text-primary);
  font-variant-numeric: tabular-nums;
  font-weight: 650;
}

.cost-tab__empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  border-radius: 1rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 24%);
  color: var(--color-text-muted);
  font-size: 0.85rem;
}

.cost-tab__empty--compact {
  min-height: 120px;
}

@media (width < 900px) {
  .cost-tab__rank-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (prefers-reduced-motion: reduce) {
  .cost-tab__rank-bar-fill {
    transition: none;
  }
}
</style>
