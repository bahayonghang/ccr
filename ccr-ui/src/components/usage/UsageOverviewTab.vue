<template>
  <div class="space-y-4">
    <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
      <div
        v-for="card in summaryCards"
        :key="card.label"
        class="glass-panel rounded-xl p-5"
      >
        <div class="mb-1.5 text-xs text-text-muted">
          {{ card.label }}
        </div>
        <div class="tabular-nums text-2xl font-bold text-text-primary">
          {{ card.value }}
        </div>
      </div>
    </div>

    <div class="grid gap-4 lg:grid-cols-3">
      <div class="glass-panel rounded-xl p-4 lg:col-span-2">
        <h3 class="mb-3 text-sm font-medium text-text-secondary">
          {{ $t('usage.dashboard.chart.trendTitle') }}
        </h3>
        <component
          :is="chartComponent"
          v-if="shouldLoadCharts && trendSeries[0]?.data?.length"
          type="area"
          height="320"
          :options="trendOptions"
          :series="trendSeries"
        />
        <div
          v-else
          class="flex h-[320px] items-center justify-center text-sm text-text-muted"
        >
          {{ $t('usage.dashboard.chart.noTrend') }}
        </div>
      </div>

      <div class="glass-panel rounded-xl p-4">
        <h3 class="mb-3 text-sm font-medium text-text-secondary">
          {{ $t('usage.dashboard.chart.costByModel') }}
        </h3>
        <component
          :is="chartComponent"
          v-if="shouldLoadCharts && modelStats.length"
          type="donut"
          height="320"
          :options="pieOptions"
          :series="pieSeries"
        />
        <div
          v-else
          class="flex h-[320px] items-center justify-center text-sm text-text-muted"
        >
          {{ $t('usage.dashboard.table.noData') }}
        </div>
      </div>
    </div>

    <div class="grid gap-4 lg:grid-cols-2">
      <div class="glass-panel overflow-hidden rounded-xl">
        <div class="border-b border-border-subtle/50 px-4 py-3 text-sm font-medium text-text-secondary">
          Top {{ $t('usage.dashboard.tabs.models') }}
        </div>
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
                {{ $t('usage.dashboard.table.cost') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="model in modelStats.slice(0, 5)"
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

      <div class="glass-panel overflow-hidden rounded-xl">
        <div class="border-b border-border-subtle/50 px-4 py-3 text-sm font-medium text-text-secondary">
          Top {{ $t('usage.dashboard.tabs.projects') }}
        </div>
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border-subtle text-left text-text-muted">
              <th class="p-3">
                {{ $t('usage.dashboard.table.project') }}
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
              v-for="project in projectStats.slice(0, 5)"
              :key="project.project_path"
              class="border-b border-border-subtle/50 transition-colors hover:bg-accent-primary/5"
            >
              <td
                class="max-w-[200px] truncate p-3 font-medium text-text-primary"
                :title="project.project_path"
              >
                {{ shortenPath(project.project_path) }}
              </td>
              <td class="p-3 text-right text-text-secondary">
                {{ formatTokens(project.total_tokens) }}
              </td>
              <td class="p-3 text-right text-text-secondary">
                {{ formatCost(project.total_cost) }}
              </td>
            </tr>
          </tbody>
        </table>
        <div
          v-if="!projectStats.length"
          class="p-6 text-center text-sm text-text-muted"
        >
          {{ $t('usage.dashboard.table.noData') }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'

type SummaryCard = {
  label: string
  value: string
}

type TrendSeriesItem = {
  name: string
  data: number[]
}

type ModelStat = {
  model: string
  request_count: number
  total_cost: number
}

type ProjectStat = {
  project_path: string
  total_tokens: number
  total_cost: number
}

interface Props {
  chartComponent: Component
  shouldLoadCharts: boolean
  summaryCards: SummaryCard[]
  trendSeries: TrendSeriesItem[]
  trendOptions: object
  pieSeries: number[]
  pieOptions: object
  modelStats: ModelStat[]
  projectStats: ProjectStat[]
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
  shortenPath: (path: string) => string
}

defineProps<Props>()
</script>
