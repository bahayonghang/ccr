<!-- eslint-disable vue/no-template-shadow -->
<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <AnimatedBackground
      contained
      variant="aurora"
    />

    <div class="max-w-[1600px] mx-auto flex flex-col gap-6 relative z-10">
      <!-- 顶部工具栏 -->
      <div class="flex flex-wrap items-center gap-3 justify-between">
        <div>
          <h1 class="text-2xl font-bold text-text-primary">
            {{ $t('usage.title') }}
          </h1>
          <p class="text-sm text-text-muted mt-1">
            {{ $t('usage.subtitle') }}
          </p>
        </div>
        <div class="flex items-center gap-2">
          <select
            v-model="selectedPlatform"
            class="toolbar-select"
            @change="onFilterChange"
          >
            <option value="">
              {{ $t('usage.dashboard.allPlatforms') }}
            </option>
            <option value="claude">
              Claude
            </option>
            <option value="codex">
              Codex
            </option>
            <option value="gemini">
              Gemini
            </option>
          </select>
          <select
            v-model="selectedDays"
            class="toolbar-select"
            @change="onFilterChange"
          >
            <option :value="7">
              {{ $t('usage.dashboard.days7') }}
            </option>
            <option :value="30">
              {{ $t('usage.dashboard.days30') }}
            </option>
            <option :value="90">
              {{ $t('usage.dashboard.days90') }}
            </option>
            <option :value="365">
              {{ $t('usage.dashboard.days365') }}
            </option>
          </select>
          <button
            class="px-3 py-1.5 rounded-lg text-xs font-medium bg-accent-primary/20 text-accent-primary hover:bg-accent-primary/30 transition-colors"
            :disabled="store.importing"
            @click="doImport"
          >
            {{ importButtonLabel }}
          </button>
          <span
            v-if="store.lastUpdated"
            class="text-[10px] text-text-muted"
          >
            {{ store.lastUpdated.toLocaleTimeString() }}
          </span>
        </div>
      </div>

      <!-- 标签页 -->
      <div class="flex gap-1 border-b border-border-subtle">
        <button
          v-for="t in tabKeys"
          :key="t"
          class="px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px"
          :class="activeTab === t ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'"
          @click="activeTab = t"
        >
          {{ $t(`usage.dashboard.tabs.${t}`) }}
        </button>
      </div>

      <!-- 加载/错误 -->
      <div
        v-if="store.loading"
        class="text-center py-12 text-text-muted"
      >
        {{ $t('usage.states.loading') }}
      </div>
      <div
        v-else-if="store.error"
        class="text-center py-12 text-accent-danger"
      >
        {{ store.error }}
      </div>
      <div
        v-else
        class="space-y-4"
      >
        <div
          v-if="warningMessage"
          class="rounded-2xl border border-accent-warning/30 bg-accent-warning/10 px-4 py-3 text-sm text-text-primary"
        >
          <div class="font-medium">
            {{ warningMessage }}
          </div>
          <div
            v-for="detail in importDetails"
            :key="detail"
            class="mt-1 text-xs text-text-secondary"
          >
            {{ detail }}
          </div>
        </div>

        <div
          v-if="showEmptyState"
          class="glass-panel rounded-2xl p-8 text-center"
        >
          <div class="text-lg font-semibold text-text-primary">
            {{ emptyStateTitle }}
          </div>
          <div class="mt-2 text-sm text-text-secondary">
            {{ emptyStateDescription }}
          </div>
        </div>

        <!-- Overview -->
        <template v-else-if="activeTab === 'overview'">
          <UsageOverviewTab
            :chart-component="apexchart"
            :format-cost="formatCost"
            :format-tokens="formatTokens"
            :model-stats="store.modelStats"
            :pie-options="pieOptions"
            :pie-series="pieSeries"
            :project-stats="store.projectStats"
            :shorten-path="shortenPath"
            :should-load-charts="shouldLoadCharts"
            :summary-cards="summaryCards"
            :trend-options="trendOptions"
            :trend-series="trendSeries"
          />
        </template>

        <!-- Models -->
        <template v-else-if="activeTab === 'models'">
          <UsageModelsTab
            :chart-component="apexchart"
            :format-cost="formatCost"
            :format-tokens="formatTokens"
            :model-stats="store.modelStats"
            :pie-options="pieOptions"
            :pie-series="pieSeries"
            :should-load-charts="shouldLoadCharts"
          />
        </template>

        <!-- Projects -->
        <template v-else-if="activeTab === 'projects'">
          <UsageProjectsTab
            :format-cost="formatCost"
            :format-tokens="formatTokens"
            :project-stats="store.projectStats"
            :shorten-path="shortenPath"
          />
        </template>

        <!-- Logs -->
        <template v-else-if="activeTab === 'logs'">
          <UsageLogsTab
            :can-next-logs="store.canNextLogs"
            :can-prev-logs="store.canPrevLogs"
            :format-cost="formatCost"
            :format-tokens="formatTokens"
            :has-logs-total="Boolean(store.logs?.total)"
            :load-logs="loadLogs"
            :logs-page="store.logsPage"
            :logs-records="logsRecords"
            :logs-total-pages="store.logsTotalPages"
            :logs-virtualizer="logsVirtualizer"
            :log-model-filter="logModelFilter"
            :set-logs-scroll-ref="setLogsScrollRef"
            :show-pager="Boolean(store.logs && (store.canPrevLogs || store.canNextLogs))"
            :update-log-model-filter="updateLogModelFilter"
          />
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import UsageLogsTab from '@/components/usage/UsageLogsTab.vue'
import UsageModelsTab from '@/components/usage/UsageModelsTab.vue'
import UsageOverviewTab from '@/components/usage/UsageOverviewTab.vue'
import UsageProjectsTab from '@/components/usage/UsageProjectsTab.vue'
import { useUsageDashboardState } from './usage/useUsageDashboardState'

// 图表仅在需要时按需加载，避免把 apexcharts 打进初始路由 chunk
const apexchart = defineAsyncComponent(async () => {
  const module = await import('vue3-apexcharts')
  return module.default
})

const {
  activeTab,
  doImport,
  emptyStateDescription,
  emptyStateTitle,
  formatCost,
  formatTokens,
  importButtonLabel,
  importDetails,
  loadLogs,
  logsRecords,
  setLogsScrollRef,
  logsVirtualizer,
  logModelFilter,
  onFilterChange,
  pieOptions,
  pieSeries,
  selectedDays,
  selectedPlatform,
  shortenPath,
  shouldLoadCharts,
  showEmptyState,
  store,
  summaryCards,
  tabKeys,
  trendOptions,
  trendSeries,
  updateLogModelFilter,
  warningMessage,
} = useUsageDashboardState()
</script>

<style scoped>
/* 表格数值等宽对齐 */
td {
  font-variant-numeric: tabular-nums;
}
</style>
