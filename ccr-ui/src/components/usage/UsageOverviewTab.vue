<template>
  <div class="overview-tab">
    <section class="overview-tab__canvas">
      <div class="overview-tab__trend glass-panel rounded-[30px] p-4">
        <div class="overview-tab__panel-head">
          <div>
            <p class="overview-tab__eyebrow">
              {{ $t('usage.dashboard.chart.trendEyebrow') }}
            </p>
            <h3 class="overview-tab__panel-title">
              {{ $t('usage.dashboard.chart.trendTitle') }}
            </h3>
            <p class="overview-tab__panel-subtitle">
              {{ trendSubtitle }}
            </p>
          </div>

          <span class="overview-tab__trend-chip">
            {{ trendGranularityLabel }}
          </span>
        </div>

        <div class="overview-tab__trend-shell">
          <component
            :is="chartComponent"
            v-if="shouldLoadCharts && hasRenderableTrendData"
            class="overview-tab__chart"
            type="area"
            height="100%"
            :options="trendOptions"
            :series="trendSeries"
          />
          <div
            v-else
            class="overview-tab__empty overview-tab__empty--trend"
          >
            {{ $t('usage.dashboard.chart.noTrend') }}
          </div>
        </div>
      </div>

      <aside class="overview-tab__side glass-panel rounded-[28px] p-4">
        <UsageModelDistributionCard
          :chart-component="chartComponent"
          :format-cost="formatCost"
          :format-tokens="formatTokens"
          :model-distribution="modelDistribution"
          :pie-colors="pieColors"
          :pie-options="pieOptions"
          :pie-series="pieSeries"
          :should-load-charts="shouldLoadCharts"
          :subtitle="distributionSubtitle"
          :title="$t('usage.dashboard.chart.costByModel')"
          variant="embedded"
        />

        <div class="overview-tab__side-divider" />

        <div class="overview-tab__insights">
          <div class="overview-tab__panel-head">
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

          <div class="overview-tab__insight-list">
            <article
              v-for="item in overviewHighlights"
              :key="item.id"
              class="overview-tab__insight-card"
            >
              <span class="overview-tab__insight-label">{{ item.label }}</span>
              <strong
                class="overview-tab__insight-value"
                :title="item.value"
              >
                {{ item.value }}
              </strong>
              <span class="overview-tab__insight-detail">{{ item.detail }}</span>
            </article>
          </div>
        </div>
      </aside>
    </section>

    <section class="overview-tab__rankings">
      <div class="overview-tab__rank-panel glass-panel rounded-[26px] p-4">
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
          v-if="topModelRankings.length > 0"
          class="overview-tab__rank-list"
        >
          <li
            v-for="(item, index) in topModelRankings"
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

      <div class="overview-tab__rank-panel glass-panel rounded-[26px] p-4">
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
          v-if="topProjectRankings.length > 0"
          class="overview-tab__rank-list"
        >
          <li
            v-for="(item, index) in topProjectRankings"
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
import type { Component } from 'vue'
import type { ModelStat, ProjectStat } from '@/types/usage'
import type { ModelDistributionSlice } from '@/views/usage/usageDashboardPresentation'
import UsageModelDistributionCard from './UsageModelDistributionCard.vue'

type TrendSeriesItem = {
  name: string
  data: Array<{ x: string; y: number }>
}

type OverviewHighlight = {
  id: string
  label: string
  value: string
  detail: string
}

type OverviewRankItem = {
  id: string
  label: string
  title: string
  detail: string
  value: string
  share: number
}

interface Props {
  chartComponent: Component
  shouldLoadCharts: boolean
  hasRenderableTrendData: boolean
  trendSeries: TrendSeriesItem[]
  trendOptions: object
  trendSubtitle: string
  trendGranularityLabel: string
  pieSeries: number[]
  pieOptions: object
  pieColors: string[]
  distributionSubtitle: string
  modelDistribution: ModelDistributionSlice[]
  modelStats: ModelStat[]
  projectStats: ProjectStat[]
  overviewHighlights: OverviewHighlight[]
  topModelRankings: OverviewRankItem[]
  topProjectRankings: OverviewRankItem[]
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
  shortenPath: (path: string) => string
}

defineProps<Props>()

const formatShare = (value: number) => `${Math.round(value * 100)}%`
</script>

<style scoped>
.overview-tab {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}

.overview-tab__canvas {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: minmax(0, 1.7fr) minmax(22rem, 0.9fr);
  align-items: start;
}

.overview-tab__trend,
.overview-tab__rank-panel {
  position: relative;
  overflow: hidden;

  /* 防止 ApexCharts 溢出 glass-panel 容器 */
  min-width: 0;
}

.overview-tab__side {
  display: grid;
  gap: 0.8rem;
  min-width: 0;
  align-content: start;
}

.overview-tab__side-divider {
  height: 1px;
  background: linear-gradient(
    90deg,
    rgb(var(--color-border-default-rgb) / 0%),
    rgb(var(--color-border-default-rgb) / 24%),
    rgb(var(--color-border-default-rgb) / 0%)
  );
}

.overview-tab__trend {
  display: grid;
  gap: 0.65rem;
  align-content: start;
}

.overview-tab__panel-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.8rem;
  margin-bottom: 0.65rem;
}

.overview-tab__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.overview-tab__panel-title {
  margin-top: 0.14rem;
  color: var(--color-text-primary);
  font-size: 0.98rem;
  font-weight: 650;
}

.overview-tab__panel-subtitle {
  margin-top: 0.2rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  line-height: 1.45;
}

.overview-tab__trend-chip {
  display: inline-flex;
  min-height: 1.9rem;
  align-items: center;
  padding: 0 0.75rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 18%);
  background: linear-gradient(
    135deg,
    rgb(var(--color-accent-primary-rgb) / 12%),
    rgb(var(--color-accent-secondary-rgb) / 10%)
  );
  color: var(--color-text-primary);
  font-size: 0.72rem;
  font-weight: 700;
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
  min-width: 0;
  min-height: clamp(15.5rem, 31vh, 18rem);
  overflow: hidden;
  border-radius: 1.15rem;
}

.overview-tab__chart {
  display: block;
  height: 100%;
  width: 100%;
}

.overview-tab__empty--trend {
  min-height: clamp(15.5rem, 31vh, 18rem);
  width: 100%;
}

.overview-tab__insight-list {
  display: grid;
  gap: 0.6rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.overview-tab__insight-card {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  min-width: 0;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 22%);
  background: linear-gradient(
    180deg,
    rgb(var(--color-bg-elevated-rgb) / 56%),
    rgb(var(--color-bg-surface-rgb) / 28%)
  );
  padding: 0.82rem 0.88rem;
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.overview-tab__side:hover .overview-tab__insight-card {
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
}

.overview-tab__insight-label {
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.overview-tab__insight-value {
  display: block;
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.94rem;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.overview-tab__insight-detail {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  line-height: 1.4;
  font-variant-numeric: tabular-nums;
}

.overview-tab__rankings {
  display: grid;
  gap: 0.9rem;
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
  .overview-tab__canvas,
  .overview-tab__rankings {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (width < 1440px) and (width >= 1280px) {
  .overview-tab__canvas {
    grid-template-columns: minmax(0, 1.45fr) minmax(20rem, 0.92fr);
  }
}

@media (width < 900px) {
  .overview-tab__insight-list {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
