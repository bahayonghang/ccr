<template>
  <div class="overview-tab">
    <UsageSourceSummaryCard
      v-if="(ctx.sourceStats?.length ?? 0) > 0"
      :format-cost="ctx.formatCost"
      :format-tokens="ctx.formatTokens"
      :selected-platform="ctx.selectedPlatform"
      :source-stats="ctx.sourceStats"
      @select-source="ctx.updateSelectedPlatform"
    />

    <section class="overview-tab__canvas overview-tab__hero">
      <div class="overview-tab__trend glass-panel rounded-2xl p-4">
        <div class="overview-tab__panel-head">
          <div class="overview-tab__trend-copy">
            <p class="overview-tab__eyebrow">
              {{ $t('usage.dashboard.chart.trendEyebrow') }}
            </p>
            <h3 class="overview-tab__panel-title">
              {{ $t('usage.dashboard.chart.trendTitle') }}
            </h3>
            <p class="overview-tab__panel-subtitle">
              {{ ctx.trendSubtitle }}
            </p>

            <div
              v-if="trendLegendItems.length > 0"
              class="overview-tab__trend-legend"
            >
              <span
                v-for="item in trendLegendItems"
                :key="item.id"
                class="overview-tab__trend-legend-item"
                :class="`overview-tab__trend-legend-item--${item.tone}`"
              >
                <span
                  class="overview-tab__trend-legend-dot"
                  aria-hidden="true"
                />
                <span class="overview-tab__trend-legend-label">{{ item.label }}</span>
                <strong class="overview-tab__trend-legend-value">{{ item.value }}</strong>
              </span>
            </div>
          </div>

          <span class="overview-tab__trend-chip">
            {{ ctx.trendGranularityLabel }}
          </span>
        </div>

        <div class="overview-tab__trend-shell">
          <component
            :is="ctx.chartComponent"
            v-if="ctx.shouldRenderTrendChart && ctx.hasRenderableTrendData"
            class="overview-tab__chart"
            type="area"
            height="100%"
            :options="ctx.trendOptions"
            :series="ctx.trendSeries"
          />
          <div
            v-else-if="ctx.hasRenderableTrendData"
            class="overview-tab__empty overview-tab__empty--trend overview-tab__empty--deferred"
          >
            {{ $t('usage.dashboard.chart.preparingTrend') }}
          </div>
          <div
            v-else
            class="overview-tab__empty overview-tab__empty--trend"
          >
            {{ $t('usage.dashboard.chart.noTrend') }}
          </div>
        </div>
      </div>

      <aside class="overview-tab__distribution glass-panel rounded-xl p-4">
        <UsageModelDistributionCard
          :chart-component="ctx.chartComponent"
          :format-cost="ctx.formatCost"
          :format-tokens="ctx.formatTokens"
          :model-distribution="ctx.modelDistribution"
          :pie-colors="ctx.pieColors"
          :pie-options="ctx.pieOptions"
          :pie-series="ctx.pieSeries"
          :should-render-chart="ctx.shouldRenderDistributionChart"
          :subtitle="ctx.distributionSubtitle"
          :title="$t('usage.dashboard.chart.costByModel')"
          variant="embedded"
        />
      </aside>
    </section>

    <section class="overview-tab__insights-strip glass-panel rounded-xl p-4">
      <div class="overview-tab__panel-head overview-tab__panel-head--strip">
        <div>
          <p class="overview-tab__eyebrow">
            {{ $t('usage.dashboard.highlights.eyebrow') }}
          </p>
          <h3 class="overview-tab__panel-title">
            {{ $t('usage.dashboard.highlights.title') }}
          </h3>
          <p class="overview-tab__panel-subtitle">
            {{ $t('usage.dashboard.highlights.subtitle') }}
          </p>
        </div>
      </div>

      <div
        v-if="ctx.overviewHighlights.length > 0"
        class="overview-tab__insight-grid"
      >
        <article
          v-for="item in ctx.overviewHighlights"
          :key="item.id"
          class="overview-tab__insight-tile"
          :class="`overview-tab__insight-tile--${getInsightTone(item.id)}`"
        >
          <span
            class="overview-tab__insight-accent"
            aria-hidden="true"
          />

          <div class="overview-tab__insight-copy">
            <span class="overview-tab__insight-label">{{ item.label }}</span>
            <strong
              class="overview-tab__insight-value"
              :title="item.value"
            >
              {{ item.value }}
            </strong>
            <span class="overview-tab__insight-detail">{{ item.detail }}</span>
          </div>
        </article>
      </div>

      <div
        v-else
        class="overview-tab__rank-empty"
      >
        {{ $t('usage.dashboard.table.noData') }}
      </div>
    </section>

    <section class="overview-tab__rankings">
      <div class="overview-tab__rank-panel glass-panel rounded-xl p-4">
        <div class="overview-tab__panel-head">
          <div>
            <p class="overview-tab__eyebrow">
              {{ $t('usage.dashboard.rankings.modelsEyebrow') }}
            </p>
            <h3 class="overview-tab__panel-title">
              {{ $t('usage.dashboard.rankings.modelsTitle') }}
            </h3>
            <p class="overview-tab__panel-subtitle">
              {{ $t('usage.dashboard.rankings.modelsSubtitle') }}
            </p>
          </div>
        </div>

        <ol
          v-if="ctx.topModelRankings.length > 0"
          class="overview-tab__rank-list"
        >
          <li
            v-for="(item, index) in ctx.topModelRankings"
            :key="item.id"
            class="overview-tab__rank-item"
          >
            <span class="overview-tab__rank-index">{{ index + 1 }}</span>

            <div class="overview-tab__rank-main">
              <div class="overview-tab__rank-row">
                <span
                  class="overview-tab__rank-label"
                  :title="item.title"
                >
                  {{ item.label }}
                </span>
                <strong class="overview-tab__rank-value">{{ item.value }}</strong>
              </div>
              <div class="overview-tab__rank-row overview-tab__rank-row--meta">
                <span class="overview-tab__rank-detail">{{ item.detail }}</span>
                <span class="overview-tab__rank-share">{{ formatShare(item.share) }}</span>
              </div>
              <div class="overview-tab__rank-bar">
                <span :style="{ width: `${Math.max(item.share * 100, 6)}%` }" />
              </div>
            </div>
          </li>
        </ol>

        <div
          v-else
          class="overview-tab__rank-empty"
        >
          {{ $t('usage.dashboard.table.noData') }}
        </div>
      </div>

      <div class="overview-tab__rank-panel glass-panel rounded-xl p-4">
        <div class="overview-tab__panel-head">
          <div>
            <p class="overview-tab__eyebrow">
              {{ $t('usage.dashboard.rankings.projectsEyebrow') }}
            </p>
            <h3 class="overview-tab__panel-title">
              {{ $t('usage.dashboard.rankings.projectsTitle') }}
            </h3>
            <p class="overview-tab__panel-subtitle">
              {{ $t('usage.dashboard.rankings.projectsSubtitle') }}
            </p>
          </div>
        </div>

        <ol
          v-if="ctx.topProjectRankings.length > 0"
          class="overview-tab__rank-list"
        >
          <li
            v-for="(item, index) in ctx.topProjectRankings"
            :key="item.id"
            class="overview-tab__rank-item"
          >
            <span class="overview-tab__rank-index">{{ index + 1 }}</span>

            <div class="overview-tab__rank-main">
              <div class="overview-tab__rank-row">
                <span
                  class="overview-tab__rank-label"
                  :title="item.title"
                >
                  {{ item.label }}
                </span>
                <strong class="overview-tab__rank-value">{{ item.value }}</strong>
              </div>
              <div class="overview-tab__rank-row overview-tab__rank-row--meta">
                <span class="overview-tab__rank-detail">{{ item.detail }}</span>
                <span class="overview-tab__rank-share">{{ formatShare(item.share) }}</span>
              </div>
              <div class="overview-tab__rank-bar">
                <span :style="{ width: `${Math.max(item.share * 100, 6)}%` }" />
              </div>
            </div>
          </li>
        </ol>

        <div
          v-else
          class="overview-tab__rank-empty"
        >
          {{ $t('usage.dashboard.table.noData') }}
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useUsageDashboardContext } from '@/views/usage/usageDashboardContext'
import UsageModelDistributionCard from './UsageModelDistributionCard.vue'
import UsageSourceSummaryCard from './UsageSourceSummaryCard.vue'

const ctx = useUsageDashboardContext()

const formatShare = (value: number) => `${Math.round(value * 100)}%`

type TrendLegendTone = 'primary' | 'secondary' | 'tertiary'

type TrendLegendItem = {
  id: string
  label: string
  value: string
  tone: TrendLegendTone
}

const formatTrendLegendValue = (value: number) => {
  if (value >= 1e9) return `${(value / 1e9).toFixed(1)}B`
  if (value >= 1e6) return `${(value / 1e6).toFixed(1)}M`
  if (value >= 1e3) return `${(value / 1e3).toFixed(1)}K`
  return value.toLocaleString()
}

const trendLegendTones: TrendLegendTone[] = ['primary', 'secondary', 'tertiary']

const trendLegendItems = computed<TrendLegendItem[]>(() =>
  ctx.trendSeries.slice(0, 3).map((series, index) => ({
    id: `${series.name}-${index}`,
    label: series.name,
    value: formatTrendLegendValue(series.data.reduce((sum, point) => sum + point.y, 0)),
    tone: trendLegendTones[index] ?? 'primary',
  }))
)

type InsightTone = 'sand' | 'rose' | 'sky' | 'amber'

const insightToneById: Record<string, InsightTone> = {
  density: 'sand',
  'top-model': 'rose',
  'top-project': 'sky',
  cache: 'amber',
}

const getInsightTone = (id: string): InsightTone => insightToneById[id] ?? 'sand'
</script>

<style scoped>
.overview-tab {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.overview-tab__hero {
  display: grid;
  gap: 1rem;
  grid-template-columns: minmax(0, 1.72fr) minmax(19.5rem, 0.88fr);
  align-items: start;
}

.overview-tab__trend,
.overview-tab__distribution,
.overview-tab__insights-strip,
.overview-tab__rank-panel {
  position: relative;
  overflow: hidden;

  /* 防止 ApexCharts 溢出 glass-panel 容器 */
  min-width: 0;
}

.overview-tab__distribution,
.overview-tab__insights-strip,
.overview-tab__rank-panel {
  display: grid;
  gap: 0.8rem;
  align-content: start;
}

.overview-tab__trend {
  display: grid;
  gap: 0.75rem;
  align-content: start;
  min-height: clamp(25rem, 47vh, 31rem);
}

.overview-tab__panel-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.8rem;
  margin-bottom: 0.5rem;
}

.overview-tab__panel-head--strip {
  margin-bottom: 0.1rem;
}

.overview-tab__trend-copy {
  display: grid;
  gap: 0.45rem;
  min-width: 0;
}

.overview-tab__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.overview-tab__panel-title {
  margin-top: 0.16rem;
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 650;
}

.overview-tab__panel-subtitle {
  margin-top: 0.18rem;
  color: var(--color-text-secondary);
  font-size: 0.77rem;
  line-height: 1.5;
}

.overview-tab__trend-chip {
  display: inline-flex;
  min-height: 1.8rem;
  align-items: center;
  padding: 0 0.72rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 18%);
  background: linear-gradient(
    135deg,
    rgb(var(--color-accent-primary-rgb) / 12%),
    rgb(var(--color-accent-secondary-rgb) / 10%)
  );
  color: var(--color-text-primary);
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.overview-tab__trend-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 0.55rem;
}

.overview-tab__trend-legend-item {
  --overview-trend-rgb: var(--color-accent-primary-rgb);

  display: inline-flex;
  min-width: 0;
  align-items: center;
  gap: 0.42rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--overview-trend-rgb) / 16%);
  background: rgb(var(--overview-trend-rgb) / 9%);
  padding: 0.36rem 0.68rem;
  color: var(--color-text-secondary);
  font-size: 0.72rem;
}

.overview-tab__trend-legend-item--primary {
  --overview-trend-rgb: var(--color-accent-primary-rgb);
}

.overview-tab__trend-legend-item--secondary {
  --overview-trend-rgb: var(--color-accent-secondary-rgb);
}

.overview-tab__trend-legend-item--tertiary {
  --overview-trend-rgb: var(--color-info-rgb);
}

.overview-tab__trend-legend-dot {
  width: 0.48rem;
  height: 0.48rem;
  flex-shrink: 0;
  border-radius: 9999px;
  background: rgb(var(--overview-trend-rgb) / 88%);
  box-shadow: 0 0 0 4px rgb(var(--overview-trend-rgb) / 12%);
}

.overview-tab__trend-legend-label {
  min-width: 0;
  color: var(--color-text-secondary);
}

.overview-tab__trend-legend-value {
  color: var(--color-text-primary);
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}

.overview-tab__empty,
.overview-tab__rank-empty {
  display: flex;
  min-height: 240px;
  align-items: center;
  justify-content: center;
  border-radius: 1.35rem;
  border: 1px dashed rgb(var(--color-accent-primary-rgb) / 16%);
  background: rgb(var(--color-bg-elevated-rgb) / 36%);
  color: var(--color-text-muted);
}

.overview-tab__trend-shell {
  position: relative;
  display: grid;
  min-width: 0;
  height: clamp(20.5rem, 40vh, 25.5rem);
  overflow: hidden;
  border-radius: 1.3rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 16%);
  background:
    radial-gradient(circle at 22% 14%, rgb(var(--color-accent-primary-rgb) / 10%), transparent 34%),
    linear-gradient(
      180deg,
      rgb(var(--color-bg-elevated-rgb) / 40%),
      rgb(var(--color-bg-surface-rgb) / 18%)
    );
}

.overview-tab__chart {
  display: block;
  height: 100%;
  width: 100%;
}

.overview-tab__chart :deep(.apexcharts-canvas),
.overview-tab__chart :deep(.apexcharts-inner),
.overview-tab__chart :deep(.apexcharts-svg) {
  width: 100% !important;
  height: 100% !important;
}

.overview-tab__chart :deep(svg) {
  height: 100% !important;
  max-width: none;
}

.overview-tab__chart :deep(.apexcharts-legend) {
  display: none !important;
}

.overview-tab__empty--trend {
  min-height: 100%;
  width: 100%;
}

.overview-tab__insight-grid {
  display: grid;
  gap: 0.7rem;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.overview-tab__insight-tile {
  --overview-insight-rgb: var(--color-accent-secondary-rgb);

  position: relative;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 0.8rem;
  align-items: start;
  min-width: 0;
  border-radius: 1.15rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 16%);
  background: linear-gradient(
    135deg,
    rgb(var(--overview-insight-rgb) / 11%),
    rgb(var(--color-bg-elevated-rgb) / 54%) 36%,
    rgb(var(--color-bg-surface-rgb) / 18%)
  );
  padding: 0.95rem 1rem;
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.overview-tab__insight-tile:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--overview-insight-rgb) / 22%);
}

.overview-tab__insight-tile--rose {
  --overview-insight-rgb: var(--color-accent-primary-rgb);
}

.overview-tab__insight-tile--sand {
  --overview-insight-rgb: var(--color-accent-secondary-rgb);
}

.overview-tab__insight-tile--sky {
  --overview-insight-rgb: var(--color-info-rgb);
}

.overview-tab__insight-tile--amber {
  --overview-insight-rgb: var(--color-warning-rgb);
}

.overview-tab__insight-accent {
  width: 0.45rem;
  min-height: 3.35rem;
  border-radius: 9999px;
  background: rgb(var(--overview-insight-rgb) / 82%);
  box-shadow: 0 0 0 4px rgb(var(--overview-insight-rgb) / 10%);
}

.overview-tab__insight-copy {
  display: grid;
  gap: 0.2rem;
  min-width: 0;
}

.overview-tab__insight-label {
  color: var(--color-text-muted);
  font-size: 0.67rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.overview-tab__insight-value {
  display: -webkit-box;
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.97rem;
  font-weight: 650;
  line-height: 1.3;
  text-overflow: ellipsis;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.overview-tab__insight-detail {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  line-height: 1.4;
  font-variant-numeric: tabular-nums;
}

.overview-tab__rankings {
  display: grid;
  gap: 1rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.overview-tab__rank-list {
  display: grid;
  gap: 0.7rem;
}

.overview-tab__rank-item {
  display: grid;
  grid-template-columns: 2.25rem minmax(0, 1fr);
  gap: 0.7rem;
  align-items: start;
}

.overview-tab__rank-index {
  display: inline-flex;
  height: 2rem;
  width: 2rem;
  align-items: center;
  justify-content: center;
  border-radius: 0.9rem;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 18%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-text-primary);
  font-size: 0.82rem;
  font-weight: 700;
}

.overview-tab__rank-main {
  min-width: 0;
  display: grid;
  gap: 0.35rem;
  padding-top: 0.1rem;
}

.overview-tab__rank-row {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: space-between;
  gap: 0.7rem;
}

.overview-tab__rank-row--meta {
  align-items: flex-start;
}

.overview-tab__rank-label {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.88rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.overview-tab__rank-value,
.overview-tab__rank-share {
  flex-shrink: 0;
  color: var(--color-text-primary);
  font-size: 0.8rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}

.overview-tab__rank-detail {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 0.76rem;
  line-height: 1.4;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.overview-tab__rank-bar {
  height: 0.34rem;
  overflow: hidden;
  border-radius: 9999px;
  background: rgb(var(--color-border-default-rgb) / 18%);
}

.overview-tab__rank-bar span {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(
    90deg,
    rgb(var(--color-accent-primary-rgb) / 86%),
    rgb(var(--color-accent-secondary-rgb) / 82%)
  );
}

@media (width < 1280px) {
  .overview-tab__hero,
  .overview-tab__rankings {
    grid-template-columns: minmax(0, 1fr);
  }

  .overview-tab__insight-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width < 1440px) and (width >= 1280px) {
  .overview-tab__hero {
    grid-template-columns: minmax(0, 1.45fr) minmax(20rem, 0.92fr);
  }
}

@media (width < 900px) {
  .overview-tab__insight-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .overview-tab__panel-head {
    flex-direction: column;
  }

  .overview-tab__trend {
    min-height: auto;
  }

  .overview-tab__trend-shell,
  .overview-tab__empty--trend {
    min-height: clamp(18rem, 52vh, 22rem);
  }
}
</style>
