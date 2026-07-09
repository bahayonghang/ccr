<template>
  <div class="token-tab">
    <!-- 步骤1：4 个 stat card —— 命中率 / Input 未缓存 / Output / Cache hit -->
    <section class="token-tab__stats">
      <article class="token-tab__stat-card">
        <p class="token-tab__stat-label">
          {{ $t('claudeCode.observer.metric.cacheHitRate') }}
        </p>
        <p class="token-tab__stat-value">
          {{ formatPercent(stats?.hit_rate ?? 0) }}
        </p>
        <p class="token-tab__stat-detail">
          {{ $t('claudeCode.observer.metric.cacheHitRateDetail') }}
        </p>
      </article>

      <article class="token-tab__stat-card">
        <p class="token-tab__stat-label">
          {{ $t('claudeCode.observer.metric.inputUncached') }}
        </p>
        <p class="token-tab__stat-value">
          {{ formatTokens(stats?.total_input_tokens ?? 0) }}
        </p>
        <p class="token-tab__stat-detail">
          {{ $t('claudeCode.observer.metric.inputUncachedDetail') }}
        </p>
      </article>

      <article class="token-tab__stat-card">
        <p class="token-tab__stat-label">
          {{ $t('claudeCode.observer.metric.output') }}
        </p>
        <p class="token-tab__stat-value">
          {{ formatTokens(stats?.total_output_tokens ?? 0) }}
        </p>
        <p class="token-tab__stat-detail">
          {{ $t('claudeCode.observer.metric.outputDetail') }}
        </p>
      </article>

      <article class="token-tab__stat-card">
        <p class="token-tab__stat-label">
          {{ $t('claudeCode.observer.metric.cacheRead') }}
        </p>
        <p class="token-tab__stat-value">
          {{ formatTokens(stats?.total_cache_read_tokens ?? 0) }}
        </p>
        <p class="token-tab__stat-detail">
          {{ $t('claudeCode.observer.metric.cacheReadDetail') }}
        </p>
      </article>
    </section>

    <!-- 步骤2：30 天堆叠条 —— input / output / cache_read / cache_write -->
    <section class="token-tab__chart-card">
      <header class="token-tab__head">
        <p class="token-tab__eyebrow">
          {{ $t('claudeCode.observer.chart.dailyTokens30') }}
        </p>
        <p class="token-tab__sub">
          {{ $t('claudeCode.observer.chart.dailyTokens30Sub') }}
        </p>
      </header>

      <div class="token-tab__chart-shell">
        <component
          :is="apexchart"
          v-if="hasDaily && shouldRenderChart"
          class="token-tab__chart"
          type="bar"
          height="280"
          :options="stackedOptions"
          :series="stackedSeries"
        />
        <ChartPreparingState
          v-else-if="hasDaily"
          :label="$t('claudeCode.observer.chart.preparingTrend')"
        />
        <div
          v-else
          class="token-tab__empty"
        >
          {{ $t('claudeCode.observer.empty.noTrend') }}
        </div>
      </div>
    </section>

    <!-- 步骤3：cache_write 解释卡 -->
    <section class="token-tab__note">
      <p class="token-tab__note-title">
        {{ $t('claudeCode.observer.tokenDetail.cacheWriteExplainTitle') }}
      </p>
      <p class="token-tab__note-body">
        {{ $t('claudeCode.observer.tokenDetail.cacheWriteExplain') }}
      </p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ApexChartAsync as apexchart } from './apexChart'
import ChartPreparingState from './ChartPreparingState.vue'
import type { CacheStatsDto, DailyPoint } from '@/types/claudeObserver'
import { buildChartTheme } from '@/views/usage/usageChartOptions'
import { formatTokens } from './formatters'

interface Props {
  stats: CacheStatsDto | null
  daily: DailyPoint[]
  animationsEnabled: boolean
  shouldRenderChart: boolean
}

const props = defineProps<Props>()

const hasDaily = computed(() => props.daily.length > 0)

const stackedSeries = computed(() => [
  {
    name: 'Cache read',
    data: props.daily.map((p) => ({ x: p.date, y: p.cache_read_tokens })),
  },
  {
    name: 'Cache write',
    data: props.daily.map((p) => ({ x: p.date, y: p.cache_write_tokens })),
  },
  {
    name: 'Input',
    data: props.daily.map((p) => ({ x: p.date, y: p.input_tokens })),
  },
  {
    name: 'Output',
    data: props.daily.map((p) => ({ x: p.date, y: p.output_tokens })),
  },
])

const stackedOptions = computed(() => {
  const theme = buildChartTheme()
  // 颜色：cache 用陶色/沙色，input 用 info 蓝，output 用 success 绿
  const colors = [theme.primary, theme.secondary, theme.tertiary, getComputedStyle(document.documentElement).getPropertyValue('--chart-color-1').trim() || '#5b8a62']
  return {
    chart: {
      type: 'bar' as const,
      stacked: true,
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
    colors,
    plotOptions: {
      bar: { columnWidth: '55%', borderRadius: 2 },
    },
    dataLabels: { enabled: false },
    stroke: { width: 0 },
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
        formatter: (value: number) => formatTokens(value),
      },
    },
    tooltip: {
      theme: theme.mode,
      x: { format: 'yyyy-MM-dd' },
      y: { formatter: (value: number) => formatTokens(value) },
    },
    legend: {
      position: 'bottom' as const,
      labels: { colors: theme.textSecondary },
      markers: { width: 10, height: 10, radius: 5 },
    },
  }
})

const formatPercent = (rate: number) => {
  const pct = Math.max(0, Math.min(1, rate)) * 100
  return `${pct.toFixed(1)}%`
}
</script>

<style scoped>
.token-tab {
  display: grid;
  gap: 0.9rem;
}

.token-tab__stats {
  display: grid;
  gap: 0.7rem;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.token-tab__stat-card {
  border-radius: 0.95rem;
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  backdrop-filter: var(--surface-card-blur);
  padding: 0.85rem 0.95rem;
  display: grid;
  gap: 0.28rem;
}

.token-tab__stat-label {
  color: var(--color-text-muted);
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.token-tab__stat-value {
  color: var(--color-text-primary);
  font-size: 1.5rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}

.token-tab__stat-detail {
  color: var(--color-text-secondary);
  font-size: 0.74rem;
}

.token-tab__chart-card {
  border-radius: 1.1rem;
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--surface-card-shadow);
  padding: 1rem 1.1rem;
}

.token-tab__head {
  display: grid;
  gap: 0.18rem;
  margin-bottom: 0.7rem;
}

.token-tab__eyebrow {
  color: var(--color-text-primary);
  font-size: 0.9rem;
  font-weight: 650;
}

.token-tab__sub {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  line-height: 1.4;
}

.token-tab__chart-shell {
  position: relative;
  height: 280px;
  min-width: 0;
}

.token-tab__chart {
  width: 100%;
  height: 100%;
}

.token-tab__chart :deep(svg) {
  width: 100% !important;
  height: 100% !important;
}

.token-tab__empty {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  border-radius: 1rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 24%);
  color: var(--color-text-muted);
  font-size: 0.85rem;
}

.token-tab__note {
  border-radius: 0.95rem;
  border: 1px solid rgb(var(--color-warning-rgb) / 22%);
  background: linear-gradient(
    180deg,
    rgb(var(--color-warning-rgb) / 8%),
    rgb(var(--color-bg-elevated-rgb, 255 255 255) / 0%)
  );
  padding: 0.85rem 1rem;
  display: grid;
  gap: 0.3rem;
}

.token-tab__note-title {
  color: var(--color-text-primary);
  font-size: 0.85rem;
  font-weight: 650;
}

.token-tab__note-body {
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  line-height: 1.6;
}

@media (width < 1100px) {
  .token-tab__stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width < 600px) {
  .token-tab__stats {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
