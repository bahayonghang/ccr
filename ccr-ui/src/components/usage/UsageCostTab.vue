<template>
  <section class="cost-tab">
    <article class="cost-tab__anchor glass-panel">
      <div>
        <p class="cost-tab__eyebrow">
          {{ $t('usage.dashboard.cost.eyebrow') }}
        </p>
        <h3>{{ $t('usage.dashboard.cost.title') }}</h3>
        <p>{{ $t('usage.dashboard.cost.subtitle') }}</p>
      </div>
      <div class="cost-tab__anchor-value">
        <span>{{ $t('usage.dashboard.cost.totalCost') }}</span>
        <strong>{{ ctx.formatCost(totalCost) }}</strong>
        <small>{{ totalRequests.toLocaleString() }} {{ $t('usage.dashboard.table.requests') }}</small>
      </div>
    </article>

    <article class="cost-tab__chart-card glass-panel">
      <div class="cost-tab__section-head">
        <div>
          <p class="cost-tab__eyebrow">
            {{ $t('usage.dashboard.cost.trendEyebrow') }}
          </p>
          <h3>{{ $t('usage.dashboard.cost.trendTitle') }}</h3>
        </div>
        <span>{{ ctx.trends.length.toLocaleString() }} {{ $t('usage.dashboard.tokens.days') }}</span>
      </div>

      <component
        :is="ctx.chartComponent"
        v-if="hasTrendRows"
        type="bar"
        height="300"
        :options="chartOptions"
        :series="chartSeries"
      />
      <div
        v-else
        class="cost-tab__empty"
      >
        {{ $t('usage.dashboard.table.noData') }}
      </div>
    </article>

    <section class="cost-tab__rankings">
      <article class="cost-tab__ranking-card glass-panel">
        <div class="cost-tab__section-head">
          <div>
            <p class="cost-tab__eyebrow">
              {{ $t('usage.dashboard.cost.sourceEyebrow') }}
            </p>
            <h3>{{ $t('usage.dashboard.cost.sourceTitle') }}</h3>
          </div>
        </div>

        <ol
          v-if="sourceRankings.length > 0"
          class="cost-tab__ranking-list"
        >
          <li
            v-for="(item, index) in sourceRankings"
            :key="item.source"
            class="cost-tab__ranking-item"
          >
            <span class="cost-tab__rank">{{ index + 1 }}</span>
            <div class="cost-tab__ranking-main">
              <span class="cost-tab__ranking-row">
                <strong>{{ sourceLabel(item.source) }}</strong>
                <b>{{ ctx.formatCost(item.total_cost) }}</b>
              </span>
              <span class="cost-tab__ranking-row cost-tab__ranking-row--meta">
                <small>{{ ctx.formatTokens(item.total_tokens) }} · {{ item.event_count.toLocaleString() }} {{ $t('usage.dashboard.table.requests') }}</small>
                <small>{{ formatPercent(item.share_cost) }}</small>
              </span>
              <span class="cost-tab__bar">
                <span :style="{ width: `${barWidth(item.share_cost)}%` }" />
              </span>
            </div>
          </li>
        </ol>

        <div
          v-else
          class="cost-tab__empty cost-tab__empty--compact"
        >
          {{ $t('usage.dashboard.table.noData') }}
        </div>
      </article>

      <article class="cost-tab__ranking-card glass-panel">
        <div class="cost-tab__section-head">
          <div>
            <p class="cost-tab__eyebrow">
              {{ $t('usage.dashboard.cost.modelEyebrow') }}
            </p>
            <h3>{{ $t('usage.dashboard.cost.modelTitle') }}</h3>
          </div>
        </div>

        <ol
          v-if="modelRankings.length > 0"
          class="cost-tab__ranking-list"
        >
          <li
            v-for="(item, index) in modelRankings"
            :key="item.model"
            class="cost-tab__ranking-item"
          >
            <span class="cost-tab__rank">{{ index + 1 }}</span>
            <div class="cost-tab__ranking-main">
              <span class="cost-tab__ranking-row">
                <strong :title="item.model">{{ item.model }}</strong>
                <b>{{ ctx.formatCost(modelCost(item)) }}</b>
              </span>
              <span class="cost-tab__ranking-row cost-tab__ranking-row--meta">
                <small>{{ ctx.formatTokens(item.total_tokens) }} · {{ item.request_count.toLocaleString() }} {{ $t('usage.dashboard.table.requests') }}</small>
                <small>{{ formatPercent(modelShare(item)) }}</small>
              </span>
              <span class="cost-tab__bar">
                <span :style="{ width: `${barWidth(modelShare(item))}%` }" />
              </span>
            </div>
          </li>
        </ol>

        <div
          v-else
          class="cost-tab__empty cost-tab__empty--compact"
        >
          {{ $t('usage.dashboard.table.noData') }}
        </div>
      </article>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ModelStat, UsagePlatform } from '@/types/usage'
import { formatPercent } from '@/views/usage/usageSummaryCards'
import { buildChartTheme, type ChartThemeState } from '@/views/usage/usageChartOptions'
import { useUsageDashboardContext } from '@/views/usage/usageDashboardContext'

const ctx = useUsageDashboardContext()

const { t } = useI18n()

const sourceLabels: Record<UsagePlatform, string> = {
  claude: 'Claude',
  codex: 'Codex',
  gemini: 'Antigravity',
  opencode: 'OpenCode',
}

const theme = computed<ChartThemeState>(() => buildChartTheme())
const totalCost = computed(() =>
  ctx.summary?.total_cost_usd ??
  ctx.trends.reduce((sum, item) => sum + item.cost_usd, 0)
)
const totalRequests = computed(() => ctx.summary?.total_requests ?? 0)
const hasTrendRows = computed(() => ctx.trends.length > 0)
const chartSeries = computed(() => [
  {
    name: tLabel('usage.dashboard.cost.totalCost', 'Total Cost'),
    data: ctx.trends.map((item) => item.cost_usd),
  },
])
const chartOptions = computed(() => ({
  chart: {
    background: 'transparent',
    fontFamily: 'inherit',
    toolbar: { show: false },
    animations: { enabled: false },
  },
  theme: { mode: theme.value.mode },
  colors: [theme.value.warning],
  plotOptions: {
    bar: {
      borderRadius: 6,
      columnWidth: '54%',
    },
  },
  dataLabels: { enabled: false },
  fill: { opacity: 0.84 },
  grid: {
    borderColor: theme.value.grid,
    strokeDashArray: 4,
    padding: { left: 6, right: 8, bottom: 4, top: 8 },
  },
  tooltip: {
    theme: theme.value.mode,
    y: {
      formatter: (value: number) => ctx.formatCost(value),
    },
  },
  xaxis: {
    categories: ctx.trends.map((item) => item.date),
    tickAmount: ctx.trends.length > 18 ? 8 : undefined,
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
      formatter: (value: number) => ctx.formatCost(value),
    },
  },
}))

const sourceRankings = computed(() =>
  [...ctx.sourceStats]
    .filter((item) => item.total_cost > 0 || item.total_tokens > 0 || item.event_count > 0)
    .sort((left, right) =>
      right.total_cost - left.total_cost ||
      right.total_tokens - left.total_tokens ||
      left.source.localeCompare(right.source),
    )
)

const modelRankings = computed(() =>
  [...ctx.modelStats]
    .filter((item) => modelCost(item) > 0 || item.total_tokens > 0 || item.request_count > 0)
    .sort((left, right) =>
      modelCost(right) - modelCost(left) ||
      right.total_tokens - left.total_tokens ||
      left.model.localeCompare(right.model),
    )
    .slice(0, 10)
)

// wire 上 source 是 string；已知平台显示品牌名，未知值原样回显
const sourceLabel = (source: string) => sourceLabels[source as UsagePlatform] ?? source
const modelCost = (model: ModelStat) => model.cost_with_cache ?? model.total_cost
const modelShare = (model: ModelStat) =>
  totalCost.value > 0 ? modelCost(model) / totalCost.value : 0
const barWidth = (share: number) => Math.min(100, Math.max(share * 100, share > 0 ? 4 : 0))

function tLabel(key: string, fallback: string) {
  const label = t(key)
  return label === key ? fallback : label
}
</script>

<style scoped>
.cost-tab {
  display: grid;
  gap: 1rem;
}

.cost-tab__anchor,
.cost-tab__chart-card,
.cost-tab__ranking-card {
  overflow: hidden;
  border-radius: 1.6rem;
  padding: 1rem;
}

.cost-tab__anchor {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  background:
    linear-gradient(135deg, rgb(var(--color-bg-elevated-rgb) / 90%), rgb(var(--color-bg-surface-rgb) / 72%)),
    radial-gradient(circle at 100% 0%, rgb(var(--color-warning-rgb) / 12%), transparent 40%);
}

.cost-tab__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 760;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.cost-tab h3 {
  margin-top: 0.18rem;
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 730;
  letter-spacing: -0.02em;
}

.cost-tab__anchor p:not(.cost-tab__eyebrow) {
  margin-top: 0.34rem;
  max-width: 48rem;
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.6;
}

.cost-tab__anchor-value {
  display: grid;
  min-width: 12rem;
  gap: 0.16rem;
  justify-items: end;
}

.cost-tab__anchor-value span,
.cost-tab__anchor-value small,
.cost-tab__section-head span {
  color: var(--color-text-muted);
  font-size: 0.7rem;
  font-weight: 700;
}

.cost-tab__anchor-value strong {
  color: var(--color-text-primary);
  font-size: clamp(1.65rem, 2vw, 2.6rem);
  font-weight: 760;
  letter-spacing: -0.055em;
  font-variant-numeric: tabular-nums;
}

.cost-tab__chart-card,
.cost-tab__ranking-card {
  display: grid;
  gap: 0.9rem;
}

.cost-tab__section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.cost-tab__section-head span {
  flex: none;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 999px;
  padding: 0.26rem 0.55rem;
}

.cost-tab__rankings {
  display: grid;
  gap: 1rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.cost-tab__ranking-list {
  display: grid;
  gap: 0.72rem;
}

.cost-tab__ranking-item {
  display: grid;
  grid-template-columns: 2.15rem minmax(0, 1fr);
  gap: 0.7rem;
  align-items: start;
}

.cost-tab__rank {
  display: inline-grid;
  width: 1.9rem;
  height: 1.9rem;
  place-items: center;
  border-radius: 0.82rem;
  border: 1px solid rgb(var(--color-warning-rgb) / 18%);
  background: rgb(var(--color-warning-rgb) / 10%);
  color: var(--color-text-primary);
  font-size: 0.78rem;
  font-weight: 760;
}

.cost-tab__ranking-main {
  display: grid;
  min-width: 0;
  gap: 0.34rem;
}

.cost-tab__ranking-row {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: space-between;
  gap: 0.65rem;
}

.cost-tab__ranking-row strong {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.9rem;
  font-weight: 690;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cost-tab__ranking-row b {
  flex: none;
  color: var(--color-text-primary);
  font-size: 0.84rem;
  font-variant-numeric: tabular-nums;
}

.cost-tab__ranking-row small {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 0.74rem;
  line-height: 1.35;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cost-tab__ranking-row--meta small:last-child {
  flex: none;
  color: var(--color-text-muted);
  font-weight: 700;
}

.cost-tab__bar {
  overflow: hidden;
  height: 0.34rem;
  border-radius: 999px;
  background: rgb(var(--color-border-default-rgb) / 18%);
}

.cost-tab__bar span {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(
    90deg,
    rgb(var(--color-warning-rgb) / 82%),
    rgb(var(--color-accent-secondary-rgb) / 74%)
  );
}

.cost-tab__empty {
  display: grid;
  min-height: 17rem;
  place-items: center;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 18%);
  border-radius: 1.1rem;
  color: var(--color-text-muted);
  font-size: 0.86rem;
}

.cost-tab__empty--compact {
  min-height: 10rem;
}

@media (width < 980px) {
  .cost-tab__anchor,
  .cost-tab__section-head {
    flex-direction: column;
  }

  .cost-tab__anchor-value {
    justify-items: start;
  }

  .cost-tab__rankings {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
