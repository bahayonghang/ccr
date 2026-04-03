<template>
  <div class="distribution-card glass-panel rounded-[28px] p-5">
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
          height="280"
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
}

const props = defineProps<Props>()

const hasData = computed(() => {
  if (!props.shouldLoadCharts || props.modelDistribution.length === 0) {
    return false
  }

  return props.pieSeries.some((value) => value > 0)
})
</script>

<style scoped>
.distribution-card {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.distribution-card__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.distribution-card__title {
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 650;
}

.distribution-card__subtitle {
  margin-top: 0.35rem;
  color: var(--color-text-secondary);
  font-size: 0.8rem;
  line-height: 1.6;
}

.distribution-card__badge {
  display: inline-flex;
  min-height: 2rem;
  min-width: 2rem;
  align-items: center;
  justify-content: center;
  padding: 0 0.75rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 18%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-text-primary);
  font-size: 0.75rem;
  font-weight: 700;
}

.distribution-card__body {
  display: grid;
  gap: 1rem;
  align-items: start;
}

.distribution-card__chart-shell {
  display: grid;
  width: 100%;
  min-width: 0;
  min-height: clamp(16rem, 34vh, 18rem);
  place-items: center;
  align-self: start;
}

.distribution-card__chart {
  width: 100%;
  min-height: 280px;
}

.distribution-card__empty {
  display: flex;
  min-height: 280px;
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
  gap: 0.7rem;
  min-width: 0;
  max-height: min(24rem, 48vh);
  overflow-y: auto;
}

.distribution-card__legend-item {
  display: grid;
  gap: 0.45rem;
  min-width: 0;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 44%);
  padding: 0.85rem 0.9rem;
}

.distribution-card__legend-row {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: space-between;
  gap: 0.85rem;
}

.distribution-card__legend-main {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.75rem;
}

.distribution-card__legend-copy {
  min-width: 0;
}

.distribution-card__swatch {
  height: 0.82rem;
  width: 0.82rem;
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
  font-size: 0.86rem;
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
  margin-top: 0.22rem;
  color: var(--color-text-secondary);
  font-size: 0.76rem;
  font-variant-numeric: tabular-nums;
}

.distribution-card__legend-stats {
  flex-shrink: 0;
}

.distribution-card__share {
  color: var(--color-text-primary);
  font-size: 0.82rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.distribution-card__bar {
  height: 0.42rem;
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
  .distribution-card__body {
    grid-template-columns: clamp(13rem, 24vw, 16rem) minmax(0, 1fr);
    align-items: start;
  }
}
</style>
