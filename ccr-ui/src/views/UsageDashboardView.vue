<!-- eslint-disable vue/no-template-shadow -->
<template>
  <div class="usage-page">
    <div class="usage-page__ambient" />
    <div class="usage-shell">
      <PageHeaderCard
        :title="$t('usage.title')"
        :description="$t('usage.subtitle')"
        badge="Usage Ops"
        icon="Activity"
        tone="secondary"
      >
        <template #meta>
          <div class="usage-header-meta">
            <span
              v-for="item in dashboardMetaItems"
              :key="item.id"
              class="usage-header-meta__chip"
            >
              <span class="usage-header-meta__label">{{ item.label }}</span>
              <strong class="usage-header-meta__value">{{ item.value }}</strong>
            </span>
          </div>
        </template>

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
        <section
          v-if="summaryCards.length > 0"
          class="usage-summary-grid"
        >
          <article
            v-for="card in summaryCards"
            :key="card.id"
            class="usage-summary-card"
            :class="`usage-summary-card--${card.tone}`"
          >
            <div class="usage-summary-card__head">
              <span class="usage-summary-card__icon">
                <SIcon
                  :name="card.icon"
                  size="w-4 h-4"
                />
              </span>
              <span class="usage-summary-card__label">{{ card.label }}</span>
            </div>
            <strong class="usage-summary-card__value">{{ card.value }}</strong>
            <p class="usage-summary-card__detail">
              {{ card.detail }}
            </p>
          </article>
        </section>

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
              :distribution-subtitle="distributionSubtitle"
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :model-stats="store.modelStats"
              :model-distribution="modelDistribution"
              :overview-highlights="overviewHighlights"
              :pie-colors="pieColors"
              :pie-options="pieOptions"
              :pie-series="pieSeries"
              :project-stats="store.projectStats"
              :shorten-path="shortenPath"
              :should-load-charts="shouldLoadCharts"
              :top-model-rankings="topModelRankings"
              :top-project-rankings="topProjectRankings"
              :trend-granularity-label="trendGranularityLabel"
              :trend-options="trendOptions"
              :trend-series="trendSeries"
              :trend-subtitle="trendSubtitle"
            />
          </template>

          <template v-else-if="activeTab === 'models'">
            <UsageModelsTab
              :chart-component="apexchart"
              :distribution-subtitle="distributionSubtitle"
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :model-stats="store.modelStats"
              :model-distribution="modelDistribution"
              :pie-colors="pieColors"
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
              :diagnostics-empty-detail="diagnosticsEmptyDetail"
              :diagnostics-empty-message="diagnosticsEmptyMessage"
              :diagnostics-summary="diagnosticsSummary"
              :format-cost="formatCost"
              :format-tokens="formatTokens"
              :has-logs-total="Boolean(store.logs?.total)"
              :load-logs="loadLogs"
              :logs-loading="store.logsLoading"
              :logs-page="store.logsPage"
              :logs-records="logsRecords"
              :logs-total-pages="store.logsTotalPages"
              :log-model-filter="logModelFilter"
              :repair-button-label="repairCodexButtonLabel"
              :repair-codex-logs="repairCodexLogs"
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
import SIcon from '@/components/ui/SIcon.vue'
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
  dashboardMetaItems,
  doImport,
  emptyStateDescription,
  emptyStateTitle,
  formatCost,
  formatTokens,
  importButtonLabel,
  importDetails,
  importJobBanner,
  importJobWarnings,
  diagnosticsEmptyDetail,
  diagnosticsEmptyMessage,
  diagnosticsSummary,
  loadLogs,
  logsRecords,
  logModelFilter,
  onFilterChange,
  overviewHighlights,
  pieColors,
  pieOptions,
  pieSeries,
  runtimeUnavailable,
  selectedDays,
  selectedPlatformLabel,
  selectedPlatform,
  selectedWindowLabel,
  repairCodexButtonLabel,
  repairCodexLogs,
  shortenPath,
  shouldLoadCharts,
  showEmptyState,
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
  updateLogModelFilter,
  warningMessage,
} = useUsageDashboardState()

const runtimeCopy = computed(() => getRuntimeUnavailableCopy('usage'))
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
    radial-gradient(circle at 12% 0%, rgb(var(--color-accent-primary-rgb) / 10%), transparent 30%),
    radial-gradient(circle at 100% 18%, rgb(var(--color-accent-secondary-rgb) / 10%), transparent 26%);
  opacity: 0.9;
}

.usage-shell {
  position: relative;
  z-index: 1;
  margin: 0 auto;
  display: flex;
  max-width: 1520px;
  flex-direction: column;
  gap: 1rem;
}

.usage-header-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.625rem;
}

.usage-header-meta__chip {
  display: inline-flex;
  min-height: 2rem;
  align-items: center;
  gap: 0.55rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 28%);
  background: rgb(var(--color-bg-elevated-rgb) / 66%);
  padding: 0.35rem 0.8rem;
  color: var(--color-text-secondary);
  backdrop-filter: blur(10px);
}

.usage-header-meta__label {
  font-size: 0.7rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.usage-header-meta__value {
  color: var(--color-text-primary);
  font-size: 0.8rem;
  font-weight: 600;
}

.usage-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 0.625rem;
}

.toolbar-select {
  min-height: 44px;
  min-width: 9rem;
  border-radius: 1rem;
  border: 1px solid var(--surface-status-border);
  padding: 0.6rem 0.9rem;
  font-size: 0.92rem;
  color: var(--color-text-primary);
  outline: none;
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
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.usage-summary-card {
  position: relative;
  overflow: hidden;
  display: flex;
  min-height: 10rem;
  flex-direction: column;
  gap: 0.8rem;
  border-radius: 1.5rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 32%);
  padding: 1rem 1.05rem 1.1rem;
  background: var(--surface-status-bg);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--elevation-1);
}

.usage-summary-card::after {
  content: '';
  position: absolute;
  inset: auto -10% -35% auto;
  width: 8.5rem;
  height: 8.5rem;
  border-radius: 9999px;
  opacity: 0.4;
  filter: blur(40px);
  pointer-events: none;
}

.usage-summary-card--rose::after {
  background: rgb(var(--color-accent-primary-rgb) / 24%);
}

.usage-summary-card--violet::after {
  background: rgb(var(--color-accent-secondary-rgb) / 22%);
}

.usage-summary-card--sky::after {
  background: rgb(var(--color-info-rgb) / 20%);
}

.usage-summary-card--amber::after {
  background: rgb(var(--color-warning-rgb) / 18%);
}

.usage-summary-card__head {
  display: flex;
  align-items: center;
  gap: 0.7rem;
}

.usage-summary-card__icon {
  display: inline-flex;
  height: 2rem;
  width: 2rem;
  align-items: center;
  justify-content: center;
  border-radius: 0.85rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 20%);
  background: rgb(var(--color-bg-elevated-rgb) / 82%);
  color: var(--color-text-primary);
}

.usage-summary-card__label {
  display: block;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.usage-summary-card__value {
  display: block;
  color: var(--color-text-primary);
  font-size: clamp(1.5rem, 1.5vw + 1rem, 2rem);
  font-weight: 700;
  letter-spacing: -0.03em;
  font-variant-numeric: tabular-nums;
}

.usage-summary-card__detail {
  margin-top: auto;
  color: var(--color-text-secondary);
  font-size: 0.85rem;
  line-height: 1.55;
}

.usage-workspace-switcher {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  border-radius: 1.5rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 32%);
  padding: 0.7rem;
  background: var(--surface-workspace-bg);
  backdrop-filter: var(--surface-workspace-blur);
  box-shadow: var(--elevation-2);
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
  padding: 0.7rem 1rem;
  font-size: 0.92rem;
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
  background: linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 14%), rgb(var(--color-accent-secondary-rgb) / 10%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  box-shadow: 0 10px 24px rgb(var(--color-accent-primary-rgb) / 10%);
}

.usage-content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.usage-warning {
  border-radius: 1.35rem;
  border: 1px solid rgb(var(--color-warning-rgb) / 24%);
  background: linear-gradient(135deg, rgb(var(--color-warning-rgb) / 14%), rgb(var(--color-accent-primary-rgb) / 8%));
  padding: 0.9rem 1rem;
  color: var(--color-text-primary);
  box-shadow: var(--elevation-1);
}

.usage-warning__detail {
  margin-top: 0.3rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
}

td {
  font-variant-numeric: tabular-nums;
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
