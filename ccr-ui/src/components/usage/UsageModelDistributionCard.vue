<template>
  <div
    class="distribution-card rounded-[28px]"
    :class="[
      variant === 'panel'
        ? 'distribution-card--panel glass-panel p-5'
        : 'distribution-card--embedded',
    ]"
  >
    <div class="distribution-card__header">
      <div>
        <h3 class="distribution-card__title">
          {{ title }}
        </h3>
        <p
          v-if="subtitle"
          class="distribution-card__subtitle"
        >
          {{ subtitle }}
        </p>
      </div>

      <span
        v-if="modelDistribution.length"
        class="distribution-card__badge"
      >
        {{ modelDistribution.length }}
      </span>
    </div>

    <div class="distribution-card__body">
      <div class="distribution-card__chart-shell">
        <component
          :is="chartComponent"
          v-if="hasData"
          class="distribution-card__chart"
          type="donut"
          :height="chartHeight"
          :options="pieOptions"
          :series="pieSeries"
        />
        <div
          v-else
          class="distribution-card__empty"
        >
          {{ $t('usage.dashboard.table.noData') }}
        </div>
      </div>

      <div
        v-if="modelDistribution.length"
        class="distribution-card__legend"
      >
        <article
          v-for="(slice, index) in modelDistribution"
          :key="slice.id"
          class="distribution-card__legend-item"
        >
          <div class="distribution-card__legend-row">
            <div class="distribution-card__legend-main">
              <span
                class="distribution-card__swatch"
                :style="{ backgroundColor: pieColors[index] || pieColors[0] }"
              />

              <div class="distribution-card__legend-copy">
                <div class="distribution-card__label-row">
                  <span
                    class="distribution-card__label"
                    :title="slice.label"
                  >
                    {{ slice.label }}
                  </span>
                  <span
                    v-if="slice.isOther && slice.childCount > 1"
                    class="distribution-card__group-tag"
                  >
                    +{{ slice.childCount }}
                  </span>
                </div>
                <div class="distribution-card__meta">
                  {{ formatCost(slice.totalCost) }} · {{ formatTokens(slice.totalTokens) }}
                </div>
              </div>
            </div>

            <div class="distribution-card__legend-stats">
              <strong class="distribution-card__share">
                {{ (slice.share * 100).toFixed(0) }}%
              </strong>
            </div>
          </div>

          <div class="distribution-card__bar">
            <span
              :style="{
                width: `${Math.max(slice.share * 100, 6)}%`,
                backgroundColor: pieColors[index] || pieColors[0],
              }"
            />
          </div>
        </article>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, type Component } from 'vue'
import type { ModelDistributionSlice } from '@/views/usage/usageDashboardPresentation'

interface Props {
  title: string
  subtitle?: string
  chartComponent: Component
  shouldLoadCharts: boolean
  pieSeries: number[]
  pieOptions: object
  pieColors: string[]
  modelDistribution: ModelDistributionSlice[]
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
  variant?: 'panel' | 'embedded'
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'panel',
})

const hasData = computed(() => {
  if (!props.shouldLoadCharts || props.modelDistribution.length === 0) {
    return false
  }

  return props.pieSeries.some((value) => value > 0)
})

const chartHeight = computed(() => (props.variant === 'embedded' ? 196 : 240))
</script>

<style scoped>
.distribution-card {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
  min-width: 0;
}

.distribution-card--panel {
  border-radius: 1.55rem;
}

.distribution-card--embedded {
  gap: 0.8rem;
}

.distribution-card__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
}

.distribution-card__title {
  color: var(--color-text-primary);
  font-size: 0.98rem;
  font-weight: 650;
}

.distribution-card__subtitle {
  margin-top: 0.2rem;
  color: var(--color-text-secondary);
  font-size: 0.77rem;
  line-height: 1.45;
}

.distribution-card__badge {
  display: inline-flex;
  min-height: 1.75rem;
  min-width: 1.75rem;
  align-items: center;
  justify-content: center;
  padding: 0 0.6rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 18%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-text-primary);
  font-size: 0.72rem;
  font-weight: 700;
}

.distribution-card__body {
  display: grid;
  gap: 0.9rem;
  align-items: start;
}

.distribution-card__chart-shell {
  display: grid;
  width: 100%;
  min-width: 0;
  min-height: clamp(12rem, 26vh, 14.5rem);
  place-items: center;
  align-self: start;
}

.distribution-card__chart {
  width: 100%;
  min-height: 220px;
}

.distribution-card__empty {
  display: flex;
  min-height: 220px;
  width: 100%;
  align-items: center;
  justify-content: center;
  border-radius: 1.35rem;
  border: 1px dashed rgb(var(--color-accent-primary-rgb) / 16%);
  background: rgb(var(--color-bg-elevated-rgb) / 35%);
  color: var(--color-text-muted);
}

.distribution-card__legend {
  display: grid;
  gap: 0.25rem;
  min-width: 0;
  max-height: min(16rem, 34vh);
  overflow-y: auto;
}

.distribution-card__legend-item {
  display: grid;
  gap: 0.35rem;
  min-width: 0;
  border-radius: 0.95rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  background: rgb(var(--color-bg-elevated-rgb) / 28%);
  padding: 0.65rem 0.72rem;
}

.distribution-card__legend-row {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: space-between;
  gap: 0.65rem;
}

.distribution-card__legend-main {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.6rem;
}

.distribution-card__legend-copy {
  min-width: 0;
}

.distribution-card__swatch {
  height: 0.72rem;
  width: 0.72rem;
  flex-shrink: 0;
  border-radius: 9999px;
  box-shadow: 0 0 0 4px rgb(255 255 255 / 3%);
}

.distribution-card__label-row {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.45rem;
}

.distribution-card__label {
  display: block;
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.82rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.distribution-card__group-tag {
  display: inline-flex;
  min-height: 1.3rem;
  align-items: center;
  padding: 0 0.45rem;
  border-radius: 9999px;
  background: rgb(var(--color-accent-secondary-rgb) / 12%);
  color: var(--color-text-secondary);
  font-size: 0.68rem;
  font-weight: 700;
}

.distribution-card__meta {
  margin-top: 0.14rem;
  color: var(--color-text-secondary);
  font-size: 0.72rem;
  font-variant-numeric: tabular-nums;
}

.distribution-card__legend-stats {
  flex-shrink: 0;
}

.distribution-card__share {
  color: var(--color-text-primary);
  font-size: 0.78rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.distribution-card__bar {
  height: 0.34rem;
  overflow: hidden;
  border-radius: 9999px;
  background: rgb(var(--color-border-default-rgb) / 18%);
}

.distribution-card__bar span {
  display: block;
  height: 100%;
  border-radius: inherit;
}

@media (width >= 1180px) {
  .distribution-card--panel .distribution-card__body {
    grid-template-columns: clamp(11.5rem, 20vw, 14rem) minmax(0, 1fr);
    align-items: start;
  }
}

.distribution-card--embedded .distribution-card__header {
  gap: 0.6rem;
}

.distribution-card--embedded .distribution-card__title {
  font-size: 0.94rem;
}

.distribution-card--embedded .distribution-card__subtitle {
  margin-top: 0.16rem;
  font-size: 0.74rem;
}

.distribution-card--embedded .distribution-card__badge {
  min-height: 1.65rem;
  min-width: 1.65rem;
  padding: 0 0.5rem;
  font-size: 0.68rem;
}

.distribution-card--embedded .distribution-card__body {
  gap: 0.72rem;
}

.distribution-card--embedded .distribution-card__chart-shell {
  min-height: clamp(10.75rem, 22vh, 12.5rem);
}

.distribution-card--embedded .distribution-card__chart,
.distribution-card--embedded .distribution-card__empty {
  min-height: 196px;
}

.distribution-card--embedded .distribution-card__legend {
  gap: 0.2rem;
  max-height: 13rem;
}

.distribution-card--embedded .distribution-card__legend-item {
  gap: 0.28rem;
  padding: 0.58rem 0.65rem;
  border-radius: 0.88rem;
}

.distribution-card--embedded .distribution-card__legend-row {
  gap: 0.5rem;
}

.distribution-card--embedded .distribution-card__legend-main {
  gap: 0.48rem;
}

.distribution-card--embedded .distribution-card__label {
  font-size: 0.79rem;
}

.distribution-card--embedded .distribution-card__group-tag {
  min-height: 1.2rem;
  padding: 0 0.38rem;
}

.distribution-card--embedded .distribution-card__meta {
  font-size: 0.7rem;
}

.distribution-card--embedded .distribution-card__share {
  font-size: 0.76rem;
}

@media (width < 900px) {
  .distribution-card__legend {
    max-height: none;
  }
}
</style>
