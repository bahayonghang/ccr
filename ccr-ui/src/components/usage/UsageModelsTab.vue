<template>
  <div class="space-y-4">
    <div class="glass-panel rounded-xl p-4">
      <h3 class="mb-3 text-sm font-medium text-text-secondary">
        {{ $t('usage.dashboard.chart.costByModel') }}
      </h3>
      <component
        :is="chartComponent"
        v-if="shouldLoadCharts && modelStats.length"
        type="donut"
        height="280"
        :options="pieOptions"
        :series="pieSeries"
      />
      <div
        v-else
        class="flex h-[280px] items-center justify-center text-sm text-text-muted"
      >
        {{ $t('usage.dashboard.table.noData') }}
      </div>
    </div>
    <div class="glass-panel overflow-hidden rounded-xl">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border-subtle text-left text-text-muted">
            <th class="p-3">
              {{ $t('usage.dashboard.table.model') }}
            </th>
            <th class="p-3 text-right">
              {{ $t('usage.dashboard.table.requests') }}
            </th>
            <th class="p-3 text-right">
              {{ $t('usage.dashboard.table.tokens') }}
            </th>
            <th class="p-3 text-right">
              {{ $t('usage.dashboard.table.cost') }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="model in modelStats"
            :key="model.model"
            class="border-b border-border-subtle/50 transition-colors hover:bg-accent-primary/5"
          >
            <td class="p-3 font-medium text-text-primary">
              {{ model.model }}
            </td>
            <td class="p-3 text-right text-text-secondary">
              {{ model.request_count }}
            </td>
            <td class="p-3 text-right text-text-secondary">
              {{ formatTokens(model.total_tokens) }}
            </td>
            <td class="p-3 text-right text-text-secondary">
              {{ formatCost(model.total_cost) }}
            </td>
          </tr>
        </tbody>
      </table>
      <div
        v-if="!modelStats.length"
        class="p-6 text-center text-sm text-text-muted"
      >
        {{ $t('usage.dashboard.table.noData') }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import type { ModelStat } from '@/types/usage'

interface Props {
  chartComponent: Component
  shouldLoadCharts: boolean
  pieSeries: number[]
  pieOptions: object
  modelStats: ModelStat[]
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
}

defineProps<Props>()
</script>
