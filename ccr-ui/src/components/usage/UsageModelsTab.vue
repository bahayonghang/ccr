<template>
  <div class="models-tab">
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
      metric="tokens"
      :title="$t('usage.dashboard.chart.tokensByModel')"
    />

    <section class="models-tab__workspace glass-panel rounded-[26px] p-4">
      <div class="models-tab__table-head">
        <div>
          <h3 class="models-tab__title">
            {{ $t('usage.dashboard.models.title') }}
          </h3>
          <p class="models-tab__subtitle">
            {{ $t('usage.dashboard.models.subtitle') }}
          </p>
        </div>
      </div>

      <div
        v-if="sortedModels.length > 0"
        class="models-tab__table-shell"
      >
        <table class="models-tab__table">
          <colgroup>
            <col class="models-tab__col models-tab__col--rank">
            <col class="models-tab__col models-tab__col--model">
            <col class="models-tab__col models-tab__col--requests">
            <col class="models-tab__col models-tab__col--tokens">
            <col class="models-tab__col models-tab__col--tokens">
            <col class="models-tab__col models-tab__col--tokens">
            <col class="models-tab__col models-tab__col--tokens">
            <col class="models-tab__col models-tab__col--rate">
            <col class="models-tab__col models-tab__col--cost">
            <col class="models-tab__col models-tab__col--cost">
            <col class="models-tab__col models-tab__col--cost">
            <col class="models-tab__col models-tab__col--status">
            <col class="models-tab__col models-tab__col--share">
          </colgroup>
          <thead>
            <tr>
              <th>#</th>
              <th>{{ $t('usage.dashboard.table.model') }}</th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.requests') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.input') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.output') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.cacheRead') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.cacheWrite') }}
              </th>
              <th>
                {{ $t('usage.dashboard.table.rate') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.costWithCache') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.costWithoutCache') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.cacheSavings') }}
              </th>
              <th>
                {{ $t('usage.dashboard.table.pricingStatus') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.share') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(model, index) in sortedModels"
              :key="model.model"
            >
              <td class="models-tab__rank-cell">
                {{ index + 1 }}
              </td>
              <td>
                <div
                  class="models-tab__model-name"
                  :title="model.model"
                >
                  {{ model.model }}
                </div>
              </td>
              <td class="is-right">
                {{ model.request_count.toLocaleString() }}
              </td>
              <td class="is-right">
                {{ formatTokens(inputTokens(model)) }}
              </td>
              <td class="is-right">
                {{ formatTokens(outputTokens(model)) }}
              </td>
              <td class="is-right">
                {{ formatTokens(cacheReadTokens(model)) }}
              </td>
              <td class="is-right">
                {{ formatTokens(cacheCreationTokens(model)) }}
              </td>
              <td>
                <span class="models-tab__rate">{{ pricingRate(model) }}</span>
              </td>
              <td class="is-right">
                {{ formatCost(costWithCache(model)) }}
              </td>
              <td class="is-right">
                {{ formatCost(costWithoutCache(model)) }}
              </td>
              <td class="is-right">
                {{ formatCost(cacheSavings(model)) }}
              </td>
              <td>
                <span
                  class="models-tab__status"
                  :class="`models-tab__status--${pricingStatus(model)}`"
                  :title="model.pricing_source || pricingStatus(model)"
                >
                  {{ $t(pricingStatusKey(model)) }}
                </span>
              </td>
              <td class="is-right">
                {{ formatShare(model.total_tokens) }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div
        v-else
        class="models-tab__empty"
      >
        {{ $t('usage.dashboard.table.noData') }}
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, type Component } from 'vue'
import type { ModelStat } from '@/types/usage'
import type { ModelDistributionSlice } from '@/views/usage/usageDashboardPresentation'
import UsageModelDistributionCard from './UsageModelDistributionCard.vue'

interface Props {
  chartComponent: Component
  shouldLoadCharts: boolean
  pieSeries: number[]
  pieOptions: object
  pieColors: string[]
  distributionSubtitle: string
  modelDistribution: ModelDistributionSlice[]
  modelStats: ModelStat[]
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
}

const props = defineProps<Props>()

const totalTokens = computed(() =>
  props.modelStats.reduce((sum, item) => sum + item.total_tokens, 0),
)

const sortedModels = computed(() =>
  [...props.modelStats].sort((left, right) =>
    right.total_tokens - left.total_tokens ||
    costWithCache(right) - costWithCache(left) ||
    right.request_count - left.request_count,
  ),
)

const inputTokens = (model: ModelStat) => model.input_tokens ?? model.total_tokens
const outputTokens = (model: ModelStat) => model.output_tokens ?? 0
const cacheReadTokens = (model: ModelStat) => model.cache_read_tokens ?? 0
const cacheCreationTokens = (model: ModelStat) => model.cache_creation_tokens ?? 0
const costWithCache = (model: ModelStat) => model.cost_with_cache ?? 0
const costWithoutCache = (model: ModelStat) =>
  model.cost_without_cache ?? 0
const cacheSavings = (model: ModelStat) =>
  model.cache_savings ?? 0
const pricingRate = (model: ModelStat) => model.pricing_rate || '-'
const pricingStatus = (model: ModelStat) => model.pricing_status || 'priced'
const pricingStatusKey = (model: ModelStat) => {
  switch (pricingStatus(model)) {
    case 'unpriced':
      return 'usage.dashboard.table.statusUnpriced'
    case 'legacy_alias':
      return 'usage.dashboard.table.statusLegacyAlias'
    default:
      return 'usage.dashboard.table.statusPriced'
  }
}

const formatShare = (value: number) => {
  if (totalTokens.value <= 0) return '0%'
  return `${Math.round((value / totalTokens.value) * 100)}%`
}
</script>

<style scoped>
.models-tab {
  display: grid;
  gap: 1rem;
}

.models-tab__workspace {
  display: grid;
  gap: 1rem;
  overflow: hidden;
}

.models-tab__table-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.models-tab__title {
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 650;
}

.models-tab__subtitle {
  margin-top: 0.35rem;
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.6;
}

.models-tab__table-shell {
  max-height: 38rem;
  overflow: auto;
  border-radius: 1.15rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 44%);
}

.models-tab__table {
  min-width: 106rem;
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
}

.models-tab__table thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: 0.85rem 1rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 94%);
  color: var(--color-text-muted);
  font-size: 0.74rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-align: left;
  text-transform: uppercase;
}

.models-tab__table tbody td {
  padding: 0.92rem 1rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  color: var(--color-text-secondary);
  font-size: 0.9rem;
  font-variant-numeric: tabular-nums;
}

.models-tab__table tbody tr:hover {
  background: rgb(var(--color-accent-primary-rgb) / 6%);
}

.models-tab__rank-cell {
  color: var(--color-text-primary);
  font-weight: 700;
}

.models-tab__model-name {
  overflow: hidden;
  color: var(--color-text-primary);
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.models-tab__rate {
  color: var(--color-text-muted);
  font-size: 0.78rem;
  white-space: nowrap;
}

.models-tab__status {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 4.8rem;
  border-radius: 999px;
  padding: 0.2rem 0.55rem;
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--color-success);
  font-size: 0.74rem;
  font-weight: 700;
  white-space: nowrap;
}

.models-tab__status--legacy_alias {
  background: rgb(var(--color-warning-rgb) / 12%);
  color: var(--color-warning);
}

.models-tab__status--unpriced {
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--color-danger);
}

.models-tab__table .is-right {
  text-align: right;
}

.models-tab__empty {
  display: flex;
  min-height: 16rem;
  align-items: center;
  justify-content: center;
  border-radius: 1.2rem;
  border: 1px dashed rgb(var(--color-accent-primary-rgb) / 16%);
  color: var(--color-text-muted);
}

.models-tab__col--rank {
  width: 4rem;
}

.models-tab__col--model {
  width: 24rem;
}

.models-tab__col--requests,
.models-tab__col--tokens,
.models-tab__col--cost,
.models-tab__col--share {
  width: 9rem;
}

.models-tab__col--rate {
  width: 10rem;
}

.models-tab__col--status {
  width: 8rem;
}
</style>
