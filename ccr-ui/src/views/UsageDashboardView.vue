<!-- eslint-disable vue/no-template-shadow -->
<template>
  <div class="usage-page">
    <div class="usage-page__ambient" />
    <div class="usage-shell">
      <UsageDashboardToolbar
        :selected-platform="selectedPlatform"
        :selected-range="selectedRange"
        :import-button-label="importButtonLabel"
        :importing="store.importing"
        :runtime-unavailable="runtimeUnavailable"
        @update:selected-platform="updateSelectedPlatform"
        @update:selected-range="updateSelectedRange"
        @import="doImport"
      />

      <div
        v-if="!runtimeUnavailable && dashboardReady && dashboardMetaItems.length > 0"
        class="usage-header-meta"
      >
        <span
          v-for="item in dashboardMetaItems"
          :key="item.id"
          class="usage-header-meta__chip"
        >
          <span class="usage-header-meta__label">{{ item.label }}</span>
          <strong class="usage-header-meta__value">{{ item.value }}</strong>
        </span>
      </div>

      <AsyncStatePanel
        v-if="runtimeUnavailable"
        state="runtime-unavailable"
        :title="runtimeCopy.title"
        :description="runtimeCopy.description"
        :action-label="runtimeCopy.actionLabel"
        action-icon="ArrowLeft"
        @action="$router.push('/')"
      />

      <template v-else>
        <UsageOpsCockpit
          v-if="dashboardReady"
          :presentation="opsCockpit"
          @primary-action="handleOpsPrimaryAction"
          @secondary-action="openDiagnostics"
        />

        <section
          v-if="dashboardReady && summaryCards.length > 0"
          class="usage-summary-grid"
        >
          <UsageMetricCard
            v-for="card in summaryCards"
            :key="card.id"
            :card="card"
          />
        </section>

        <UsageTokenBreakdownStrip
          v-if="dashboardReady && store.summary"
          :summary="store.summary"
          :cache-creation-tokens="cacheCreationTokens"
        />

        <div class="usage-workspace-switcher">
          <div class="usage-tabs">
            <button
              v-for="tab in tabKeys"
              :key="tab"
              class="usage-tab"
              :class="{ 'usage-tab--active': activeTab === tab }"
              @click="activeTab = tab"
            >
              {{ $t(`usage.dashboard.tabs.${tab}`) }}
            </button>
          </div>
          <p class="usage-workspace-switcher__summary">
            {{ selectedPlatformLabel }} · {{ selectedWindowLabel }}
          </p>
        </div>

        <AsyncStatePanel
          v-if="store.loading"
          state="loading"
          :title="$t('usage.states.loading')"
          compact
        />

        <AsyncStatePanel
          v-else-if="store.error"
          state="error"
          :title="$t('usage.states.loadFailed')"
          :description="store.error"
          :action-label="$t('common.retry')"
          action-icon="RefreshCw"
          @action="onFilterChange"
        />

        <AsyncStatePanel
          v-else-if="store.dashboardUnsupported"
          state="empty"
          :title="unsupportedStateTitle"
          :description="unsupportedStateDescription"
          icon="Database"
          compact
        />

        <AsyncStatePanel
          v-else-if="!dashboardReady"
          state="loading"
          :title="$t('common.loading')"
          compact
        />

        <div
          v-else
          class="usage-content"
        >
          <AsyncStatePanel
            v-if="showEmptyState"
            state="empty"
            :title="emptyStateTitle"
            :description="emptyStateDescription"
            compact
          />

          <template v-else-if="activeTab === 'overview'">
            <UsageSourceSummaryCard
              v-if="sourceStats.length > 0"
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :selected-platform="selectedPlatform"
              :source-stats="sourceStats"
              @select-source="updateSelectedPlatform"
            />

            <UsageOverviewTab
              :chart-component="apexchart"
              :distribution-subtitle="distributionSubtitle"
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :has-renderable-trend-data="hasRenderableTrendData"
              :model-stats="store.modelStats"
              :model-distribution="modelDistribution"
              :overview-highlights="overviewHighlights"
              :pie-colors="pieColors"
              :pie-options="pieOptions"
              :pie-series="pieSeries"
              :project-stats="store.projectStats"
              :shorten-path="shortenPath"
              :should-render-distribution-chart="shouldRenderDistributionChart"
              :should-render-trend-chart="shouldRenderTrendChart"
              :top-model-rankings="topModelRankings"
              :top-project-rankings="topProjectRankings"
              :trend-granularity-label="trendGranularityLabel"
              :trend-options="trendOptions"
              :trend-series="trendSeries"
              :trend-subtitle="trendSubtitle"
            />
          </template>

          <template v-else-if="activeTab === 'tokens'">
            <UsageTokensTab
              :chart-component="apexchart"
              :format-tokens="formatTokens"
              :trends="store.trends"
            />
          </template>

          <template v-else-if="activeTab === 'cost'">
            <UsageCostTab
              :chart-component="apexchart"
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :model-stats="store.modelStats"
              :source-stats="sourceStats"
              :summary="store.summary"
              :trends="store.trends"
            />
          </template>

          <template v-else-if="activeTab === 'providers'">
            <UsageProvidersTab
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :provider-capability="providerCapability"
              :provider-stats="providerStats"
              :selected-window-label="selectedWindowLabel"
            />
          </template>

          <template v-else-if="activeTab === 'models'">
            <UsageModelsTab
              :chart-component="apexchart"
              :distribution-subtitle="distributionSubtitle"
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :model-stats="store.modelStats"
              :model-distribution="modelTokenDistribution"
              :pie-colors="pieColors"
              :pie-options="modelTokenPieOptions"
              :pie-series="modelTokenPieSeries"
              :should-render-chart="shouldRenderDistributionChart"
            />
          </template>

          <template v-else-if="activeTab === 'projects'">
            <UsageProjectsTab
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :project-stats="store.projectStats"
              :shorten-path="shortenPath"
            />
          </template>

          <template v-else-if="activeTab === 'logs'">
            <UsageLogsTab
              :can-next-logs="store.canNextLogs"
              :can-prev-logs="store.canPrevLogs"
              :diagnostics-empty-detail="diagnosticsEmptyDetail"
              :diagnostics-empty-message="diagnosticsEmptyMessage"
              :diagnostics-summary="diagnosticsSummary"
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :has-logs-total="store.hasLogsTotal"
              :load-logs="loadLogs"
              :logs-loading="store.logsLoading"
              :logs-page="store.logsPage"
              :logs-records="logsRecords"
              :logs-total-pages="store.logsTotalPages"
              :log-model-filter="logModelFilter"
              :repair-button-label="repairCodexButtonLabel"
              :repair-codex-logs="repairCodexLogs"
              :show-pager="store.showLogsPager"
              :update-log-model-filter="updateLogModelFilter"
            />
          </template>
        </div>
      </template>
    </div>

    <!-- llmusage Install Dialog -->
    <LlmusageInstallDialog
      v-model:is-open="showInstallDialog"
      @retry-import="doImportAfterInstall"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import UsageDashboardToolbar from '@/components/usage/UsageDashboardToolbar.vue'
import UsageLogsTab from '@/components/usage/UsageLogsTab.vue'
import UsageMetricCard from '@/components/usage/UsageMetricCard.vue'
import UsageModelsTab from '@/components/usage/UsageModelsTab.vue'
import UsageOpsCockpit from '@/components/usage/UsageOpsCockpit.vue'
import UsageOverviewTab from '@/components/usage/UsageOverviewTab.vue'
import UsageProjectsTab from '@/components/usage/UsageProjectsTab.vue'
import UsageProvidersTab from '@/components/usage/UsageProvidersTab.vue'
import UsageSourceSummaryCard from '@/components/usage/UsageSourceSummaryCard.vue'
import UsageTokenBreakdownStrip from '@/components/usage/UsageTokenBreakdownStrip.vue'
import type { UsageRangePreset } from './usage/dateWindow'
import { perfMark, perfMeasure } from '@/utils/perfTelemetry'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { useUsageDashboardState } from './usage/useUsageDashboardState'

const apexchart = defineAsyncComponent(async () => {
  perfMark('usage_chart_import_start')
  const module = await import('vue3-apexcharts')
  perfMark('usage_chart_import_end')
  perfMeasure('usage_chart_import_ms', 'usage_chart_import_start', 'usage_chart_import_end')
  return module.default
})

const LlmusageInstallDialog = defineAsyncComponent({
  loader: () => import('@/components/usage/LlmusageInstallDialog.vue'),
  suspensible: false,
})

const UsageTokensTab = defineAsyncComponent({
  loader: () => import('@/components/usage/UsageTokensTab.vue'),
  suspensible: false,
})

const UsageCostTab = defineAsyncComponent({
  loader: () => import('@/components/usage/UsageCostTab.vue'),
  suspensible: false,
})

const {
  activeTab,
  cacheCreationTokens,
  dashboardReady,
  dashboardMetaItems,
  doImport,
  doImportAfterInstall,
  emptyStateDescription,
  emptyStateTitle,
  formatCost,
  formatTokens,
  importButtonLabel,
  diagnosticsEmptyDetail,
  diagnosticsEmptyMessage,
  diagnosticsSummary,
  hasRenderableTrendData,
  handleOpsPrimaryAction,
  loadLogs,
  logsRecords,
  logModelFilter,
  onFilterChange,
  openDiagnostics,
  opsCockpit,
  overviewHighlights,
  pieColors,
  pieOptions,
  pieSeries,
  providerCapability,
  providerStats,
  runtimeUnavailable,
  selectedPlatformLabel,
  selectedPlatform,
  selectedRange,
  selectedWindowLabel,
  repairCodexButtonLabel,
  repairCodexLogs,
  shortenPath,
  shouldRenderDistributionChart,
  shouldRenderTrendChart,
  showEmptyState,
  showInstallDialog,
  sourceStats,
  store,
  trendSubtitle,
  summaryCards,
  tabKeys,
  topModelRankings,
  topProjectRankings,
  trendGranularityLabel,
  trendOptions,
  trendSeries,
  distributionSubtitle,
  modelDistribution,
  modelTokenDistribution,
  modelTokenPieOptions,
  modelTokenPieSeries,
  updateLogModelFilter,
  unsupportedStateDescription,
  unsupportedStateTitle,
} = useUsageDashboardState()

const runtimeCopy = computed(() => getRuntimeUnavailableCopy('usage'))

const updateSelectedPlatform = (value: string) => {
  selectedPlatform.value = value
  onFilterChange()
}

const updateSelectedRange = (value: UsageRangePreset) => {
  selectedRange.value = value
  onFilterChange()
}
</script>

<style scoped>
.usage-page {
  position: relative;
  padding: 1rem 1rem 1.5rem;
}

.usage-page__ambient {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background:
    radial-gradient(circle at 12% 0%, rgb(var(--color-accent-primary-rgb) / 8%), transparent 28%),
    radial-gradient(circle at 100% 18%, rgb(var(--color-premium-blue-rgb) / 18%), transparent 34%);
  opacity: 0.7;
}

.usage-shell {
  position: relative;
  z-index: 1;
  margin: 0 auto;
  display: flex;
  max-width: 1520px;
  flex-direction: column;
  gap: 0.85rem;
}

.usage-header-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.usage-header-meta__chip {
  display: inline-flex;
  min-height: 1.8rem;
  align-items: center;
  gap: 0.45rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 28%);
  background: rgb(var(--color-bg-elevated-rgb) / 66%);
  padding: 0.28rem 0.7rem;
  color: var(--color-text-secondary);
}

.usage-header-meta__label {
  font-size: 0.68rem;
  letter-spacing: 0.04em;
}

.usage-header-meta__value {
  color: var(--color-text-primary);
  font-size: 0.78rem;
  font-weight: 600;
}

.usage-summary-grid {
  display: grid;
  gap: 0.8rem;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.usage-workspace-switcher {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  border-radius: 1.35rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  padding: 0.58rem 0.65rem;
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 86%), rgb(var(--color-bg-surface-rgb) / 72%));
  box-shadow: var(--elevation-1);
}

.usage-workspace-switcher__summary {
  color: var(--color-text-secondary);
  font-size: 0.84rem;
}

.usage-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.usage-tab {
  border-radius: 1rem;
  border: 1px solid transparent;
  padding: 0.62rem 0.95rem;
  font-size: 0.88rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-tab:hover {
  color: var(--color-text-primary);
  background: var(--surface-status-bg);
  transform: translateY(-1px);
}

.usage-tab--active {
  color: var(--color-text-primary);
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 96%), rgb(var(--color-bg-surface-rgb) / 88%));
  border-color: rgb(var(--color-accent-primary-rgb) / 16%);
  box-shadow: 0 10px 24px rgb(var(--color-accent-primary-rgb) / 8%);
}

.usage-content {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}

td {
  font-variant-numeric: tabular-nums;
}

.usage-page :deep(.glass-panel) {
  border-color: rgb(var(--color-border-default-rgb) / 14%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 90%), rgb(var(--color-bg-surface-rgb) / 72%));
  backdrop-filter: none;
  box-shadow: var(--elevation-1), inset 0 1px 0 rgb(255 255 255 / 6%);
}

@media (width < 1280px) {
  .usage-summary-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width < 900px) {
  .usage-summary-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .usage-workspace-switcher {
    align-items: flex-start;
  }
}
</style>
