<template>
  <article class="platform-usage-chart">
    <div class="platform-usage-chart__head">
      <div>
        <p>{{ eyebrow }}</p>
        <h3>{{ title }}</h3>
      </div>
      <span>{{ windowLabel }}</span>
    </div>

    <div
      v-if="!trends.length"
      class="platform-usage-chart__empty"
    >
      {{ emptyLabel }}
    </div>

    <component
      :is="ApexChart"
      v-else-if="canRenderApex"
      class="platform-usage-chart__apex"
      :type="chartType"
      height="286"
      :options="chartOptions"
      :series="series"
    />

    <div
      v-else
      class="platform-usage-chart__fallback"
      aria-hidden="true"
    >
      <span
        v-for="bar in fallbackBars"
        :key="bar.key"
        :style="{ height: `${bar.height}%` }"
      />
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import type { ApexOptions } from 'apexcharts'
import type { DailyTrend } from '@/types/usage'
import type { PlatformUsageMetric } from '@/types/platformUsageInsight'
import { buildChartTheme } from '@/views/usage/usageChartOptions'
import { formatCost, formatTokens } from '@/views/usage/usageSummaryCards'

const props = defineProps<{
  title: string
  eyebrow: string
  windowLabel: string
  emptyLabel: string
  metric: PlatformUsageMetric
  trends: DailyTrend[]
}>()

const ApexChart = defineAsyncComponent({
  loader: () => import('@/utils/apexChartsCore'),
  suspensible: false,
})

const canRenderApex = computed(() => {
  if (typeof navigator === 'undefined') return true
  return !/jsdom/i.test(navigator.userAgent)
})

const theme = computed(() => buildChartTheme())

const chartType = computed(() => (props.metric === 'tokens' ? 'bar' : props.metric === 'requests' ? 'line' : 'area'))

const categories = computed(() => props.trends.map((trend) => trend.date))

const series = computed(() => {
  if (props.metric === 'tokens') {
    return [
      { name: 'Input', data: props.trends.map((trend) => trend.input_tokens) },
      { name: 'Output', data: props.trends.map((trend) => trend.output_tokens) },
      { name: 'Cache read', data: props.trends.map((trend) => trend.cache_read_tokens) },
      { name: 'Cache write', data: props.trends.map((trend) => trend.cache_creation_tokens) },
    ]
  }

  if (props.metric === 'requests') {
    return [
      { name: 'Requests', data: props.trends.map((trend) => trend.request_count) },
    ]
  }

  return [
    { name: 'Cost', data: props.trends.map((trend) => trend.cost_usd) },
  ]
})

const fallbackValues = computed(() => {
  if (props.metric === 'tokens') return props.trends.map((trend) => trend.total_tokens)
  if (props.metric === 'requests') return props.trends.map((trend) => trend.request_count)
  return props.trends.map((trend) => trend.cost_usd)
})

const fallbackBars = computed(() => {
  const maxValue = Math.max(...fallbackValues.value, 0)
  return fallbackValues.value.map((value, index) => ({
    key: `${props.trends[index]?.date ?? index}-${index}`,
    height: maxValue > 0 ? Math.max(8, (value / maxValue) * 100) : 8,
  }))
})

const chartOptions = computed<ApexOptions>(() => {
  const chartTheme = theme.value
  const isTokenChart = props.metric === 'tokens'
  const isCostChart = props.metric === 'cost'

  return {
    chart: {
      id: `platform-usage-${props.metric}`,
      toolbar: { show: false },
      animations: { enabled: !window.matchMedia?.('(prefers-reduced-motion: reduce)').matches },
      fontFamily: 'var(--font-sans)',
      background: 'transparent',
      stacked: isTokenChart,
    },
    colors: [
      chartTheme.primary,
      chartTheme.secondary,
      chartTheme.tertiary,
      chartTheme.quaternary,
    ],
    dataLabels: { enabled: false },
    fill: {
      type: isCostChart ? 'gradient' : 'solid',
      gradient: {
        shadeIntensity: 0.2,
        opacityFrom: 0.35,
        opacityTo: 0.02,
      },
      opacity: isTokenChart ? 0.72 : 1,
    },
    grid: {
      borderColor: chartTheme.grid,
      strokeDashArray: 3,
      padding: { left: 8, right: 12 },
    },
    legend: {
      show: isTokenChart,
      labels: { colors: chartTheme.textSecondary },
      markers: { size: 8 },
    },
    plotOptions: {
      bar: {
        borderRadius: 5,
        columnWidth: '56%',
      },
    },
    stroke: {
      width: isTokenChart ? 0 : 2.4,
      curve: 'smooth' as const,
    },
    theme: { mode: chartTheme.mode },
    tooltip: {
      theme: chartTheme.mode,
      y: {
        formatter: (value: number) => {
          if (props.metric === 'cost') return formatCost(value)
          if (props.metric === 'tokens') return formatTokens(value)
          return Math.round(value).toLocaleString()
        },
      },
    },
    xaxis: {
      categories: categories.value,
      labels: {
        rotate: 0,
        trim: true,
        style: {
          colors: chartTheme.textMuted,
          fontSize: '11px',
        },
      },
      axisBorder: { show: false },
      axisTicks: { show: false },
      tickAmount: categories.value.length > 16 ? 6 : undefined,
    },
    yaxis: {
      labels: {
        style: {
          colors: chartTheme.textMuted,
          fontSize: '11px',
        },
        formatter: (value: number) => {
          if (props.metric === 'cost') return formatCost(value)
          if (props.metric === 'tokens') return formatTokens(value)
          return Math.round(value).toLocaleString()
        },
      },
    },
  }
})
</script>

<style scoped>
.platform-usage-chart {
  display: grid;
  gap: 0.9rem;
  min-width: 0;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 1.25rem;
  background: var(--color-bg-surface);
  padding: 1rem;
}

.platform-usage-chart__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.platform-usage-chart__head p {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 760;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.platform-usage-chart__head h3 {
  margin-top: 0.18rem;
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 730;
  letter-spacing: -0.02em;
}

.platform-usage-chart__head span {
  flex: none;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 999px;
  padding: 0.26rem 0.55rem;
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 690;
}

.platform-usage-chart__apex {
  min-height: 286px;
}

.platform-usage-chart__empty,
.platform-usage-chart__fallback {
  min-height: 286px;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 16%);
  border-radius: 1rem;
}

.platform-usage-chart__empty {
  display: grid;
  place-items: center;
  color: var(--color-text-muted);
  font-size: 0.84rem;
  text-align: center;
}

.platform-usage-chart__fallback {
  display: flex;
  align-items: end;
  gap: 0.22rem;
  padding: 1rem;
  background: transparent;
}

.platform-usage-chart__fallback span {
  flex: 1;
  min-width: 0.16rem;
  border-radius: 999px 999px 0 0;
  background: rgb(var(--color-accent-primary-rgb) / 55%);
}
</style>
