<!-- eslint-disable vue/no-template-shadow -->
<template>
  <div class="usage-page">
    <div class="usage-shell">
      <PageHeaderCard
        :title="$t('usage.title')"
        :description="$t('usage.subtitle')"
        badge="Dashboard"
        icon="Activity"
        tone="info"
      >
        <template
          v-if="!runtimeUnavailable"
          #actions
        >
          <div class="usage-toolbar">
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
            <Button
              variant="primary"
              density="compact"
              surface="card"
              motion="standard"
              :disabled="store.importing"
              @click="doImport"
            >
              {{ importButtonLabel }}
            </Button>
          </div>
        </template>

        <div
          v-if="!runtimeUnavailable && summaryCards.length > 0"
          class="usage-summary-grid"
        >
          <div
            v-for="card in summaryCards"
            :key="card.label"
            class="usage-summary-card"
          >
            <span class="usage-summary-card__label">{{ card.label }}</span>
            <strong class="usage-summary-card__value">{{ card.value }}</strong>
          </div>
        </div>
      </PageHeaderCard>

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

        <AsyncStatePanel
          v-if="store.loading"
          state="loading"
          :title="$t('usage.states.loading')"
          compact
        />

        <AsyncStatePanel
          v-else-if="store.error"
          state="error"
          title="Unable to load usage data"
          :description="store.error"
          action-label="Retry"
          action-icon="RefreshCw"
          @action="onFilterChange"
        />

        <div
          v-else
          class="usage-content"
        >
          <div
            v-if="importJobBanner"
            class="usage-warning"
          >
            <div class="font-medium">
              {{ importJobBanner }}
            </div>
            <div
              v-for="detail in importJobWarnings"
              :key="detail"
              class="usage-warning__detail"
            >
              {{ detail }}
            </div>
          </div>

          <div
            v-if="warningMessage"
            class="usage-warning"
          >
            <div class="font-medium">
              {{ warningMessage }}
            </div>
            <div
              v-for="detail in importDetails"
              :key="detail"
              class="usage-warning__detail"
            >
              {{ detail }}
            </div>
          </div>

          <AsyncStatePanel
            v-if="showEmptyState"
            state="empty"
            :title="emptyStateTitle"
            :description="emptyStateDescription"
            compact
          />

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
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import Button from '@/components/ui/Button.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import UsageLogsTab from '@/components/usage/UsageLogsTab.vue'
import UsageModelsTab from '@/components/usage/UsageModelsTab.vue'
import UsageOverviewTab from '@/components/usage/UsageOverviewTab.vue'
import UsageProjectsTab from '@/components/usage/UsageProjectsTab.vue'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { useUsageDashboardState } from './usage/useUsageDashboardState'

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
  importJobBanner,
  importJobWarnings,
  loadLogs,
  logsRecords,
  setLogsScrollRef,
  logsVirtualizer,
  logModelFilter,
  onFilterChange,
  pieOptions,
  pieSeries,
  runtimeUnavailable,
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

const runtimeCopy = computed(() => getRuntimeUnavailableCopy('usage'))
</script>

<style scoped>
.usage-page {
  @apply px-4 py-4 sm:px-6 sm:py-6;
}

.usage-shell {
  @apply mx-auto flex max-w-[1440px] flex-col gap-5;
}

.usage-toolbar {
  @apply flex flex-wrap items-center gap-2;
}

.toolbar-select {
  @apply min-h-[44px] rounded-xl border px-3 py-2 text-sm text-text-primary outline-none;

  border-color: var(--surface-status-border);
  background: var(--surface-status-bg);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--elevation-1);
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.toolbar-select:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
}

.toolbar-select:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.usage-summary-grid {
  @apply grid gap-3 md:grid-cols-2 xl:grid-cols-4;
}

.usage-summary-card {
  @apply rounded-2xl border border-border-default/55 px-4 py-3;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--elevation-1);
}

.usage-summary-card__label {
  @apply block text-[11px] uppercase tracking-[0.12em] text-text-muted;
}

.usage-summary-card__value {
  @apply mt-2 block text-lg font-semibold tracking-tight text-text-primary;

  font-variant-numeric: tabular-nums;
}

.usage-tabs {
  @apply flex flex-wrap gap-2 rounded-2xl border border-border-default/55 p-2;

  background: var(--surface-workspace-bg);
  border-color: var(--surface-workspace-border);
  backdrop-filter: var(--surface-workspace-blur);
  box-shadow: var(--elevation-2);
}

.usage-tab {
  @apply rounded-xl border border-transparent px-4 py-2 text-sm font-medium text-text-secondary;

  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-tab:hover {
  @apply text-text-primary;

  background: var(--surface-status-bg);
  transform: translateY(-1px);
}

.usage-tab--active {
  @apply text-text-primary;

  background: linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 14%), rgb(var(--color-accent-secondary-rgb) / 10%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
}

.usage-content {
  @apply flex flex-col gap-4;
}

.usage-warning {
  @apply rounded-2xl border border-accent-warning/25 bg-accent-warning/10 px-4 py-3 text-sm text-text-primary;
}

.usage-warning__detail {
  @apply mt-1 text-xs text-text-secondary;
}

td {
  font-variant-numeric: tabular-nums;
}
</style>
