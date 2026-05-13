<!-- eslint-disable vue/no-template-shadow -->
<template>
  <div class="usage-page">
    <div class="usage-page__ambient" />
    <div class="usage-shell">
      <PageHeaderCard
        class="usage-page__header"
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
              <option value="opencode">
                OpenCode
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
          v-if="dashboardReady && summaryCards.length > 0"
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
            v-if="unsupportedSyncMessage || warningMessage"
            class="usage-warning"
          >
            <div class="font-medium">
              {{ unsupportedSyncMessage || warningMessage }}
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
              :has-renderable-trend-data="hasRenderableTrendData"
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
import Button from '@/components/ui/Button.vue'
import LlmusageInstallDialog from '@/components/usage/LlmusageInstallDialog.vue'
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
  dashboardReady,
  dashboardMetaItems,
  doImport,
  doImportAfterInstall,
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
  hasRenderableTrendData,
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
  showInstallDialog,
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
  unsupportedStateDescription,
  unsupportedStateTitle,
  unsupportedSyncMessage,
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
    radial-gradient(circle at 12% 0%, rgb(var(--color-accent-primary-rgb) / 8%), transparent 28%),
    radial-gradient(circle at 100% 18%, rgb(var(--color-premium-blue-rgb) / 80%), transparent 30%);
  opacity: 1;
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
  backdrop-filter: blur(10px);
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

.usage-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 0.55rem;
}

.toolbar-select {
  min-height: 40px;
  min-width: 8.5rem;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  padding: 0.52rem 0.82rem;
  font-size: 0.88rem;
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
  border-color: rgb(var(--color-accent-primary-rgb) / 16%);
}

.toolbar-select:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 22%);
  box-shadow: var(--elevation-2);
}

.usage-summary-grid {
  display: grid;
  gap: 0.8rem;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.usage-summary-card {
  position: relative;
  overflow: hidden;
  display: flex;
  min-height: 7.75rem;
  flex-direction: column;
  gap: 0.45rem;
  border-radius: 1.35rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  border-left: 3px solid rgb(var(--color-border-default-rgb) / 14%);
  padding: 0.82rem 0.95rem 0.88rem;
  background: var(--surface-status-bg);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--elevation-1), inset 0 1px 0 rgb(255 255 255 / 12%);
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);
}

/* ── Per-tone: rose (总请求数) ── */
.usage-summary-card--rose {
  border-left-color: rgb(var(--color-accent-primary-rgb) / 60%);
}

.usage-summary-card--rose:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  border-left-color: rgb(var(--color-accent-primary-rgb) / 70%);
  box-shadow: var(--elevation-2);
}

.usage-summary-card--rose .usage-summary-card__icon {
  background: rgb(var(--color-accent-primary-rgb) / 14%);
  color: rgb(var(--color-accent-primary-rgb));
}

.usage-summary-card--rose .usage-summary-card__value {
  color: rgb(var(--color-accent-primary-rgb));
}

/* ── Per-tone: violet (总 Tokens) ── */
.usage-summary-card--violet {
  border-left-color: rgb(var(--color-accent-secondary-rgb) / 60%);
}

.usage-summary-card--violet:hover {
  border-color: rgb(var(--color-accent-secondary-rgb) / 18%);
  border-left-color: rgb(var(--color-accent-secondary-rgb) / 70%);
  box-shadow: var(--elevation-2);
}

.usage-summary-card--violet .usage-summary-card__icon {
  background: rgb(var(--color-accent-secondary-rgb) / 14%);
  color: rgb(var(--color-accent-secondary-rgb));
}

.usage-summary-card--violet .usage-summary-card__value {
  color: rgb(var(--color-accent-secondary-rgb));
}

/* ── Per-tone: sky (总费用) ── */
.usage-summary-card--sky {
  border-left-color: rgb(var(--color-info-rgb) / 64%);
}

.usage-summary-card--sky:hover {
  border-color: rgb(var(--color-info-rgb) / 18%);
  border-left-color: rgb(var(--color-info-rgb) / 70%);
  box-shadow: var(--elevation-2);
}

.usage-summary-card--sky .usage-summary-card__icon {
  background: rgb(var(--color-info-rgb) / 14%);
  color: rgb(var(--color-info-rgb));
}

.usage-summary-card--sky .usage-summary-card__value {
  color: rgb(var(--color-info-rgb));
}

/* ── Per-tone: amber (缓存效率) ── */
.usage-summary-card--amber {
  border-left-color: rgb(var(--color-warning-rgb) / 64%);
}

.usage-summary-card--amber:hover {
  border-color: rgb(var(--color-warning-rgb) / 18%);
  border-left-color: rgb(var(--color-warning-rgb) / 70%);
  box-shadow: var(--elevation-2);
}

.usage-summary-card--amber .usage-summary-card__icon {
  background: rgb(var(--color-warning-rgb) / 14%);
  color: rgb(var(--color-warning-rgb));
}

.usage-summary-card--amber .usage-summary-card__value {
  color: rgb(var(--color-warning-rgb));
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
  gap: 0.55rem;
}

.usage-summary-card__icon {
  display: inline-flex;
  height: 1.8rem;
  width: 1.8rem;
  align-items: center;
  justify-content: center;
  border-radius: 0.7rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 10%);
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-text-primary);
}

.usage-summary-card__label {
  display: block;
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.1em;
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
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  line-height: 1.35;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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

.usage-warning {
  border-radius: 1.35rem;
  border: 1px solid rgb(var(--color-warning-rgb) / 18%);
  background: linear-gradient(180deg, rgb(var(--color-warning-rgb) / 12%), rgb(var(--color-bg-elevated-rgb) / 90%));
  padding: 0.9rem 1rem;
  color: var(--color-text-primary);
  box-shadow: var(--elevation-1), inset 0 1px 0 rgb(255 255 255 / 12%);
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

.usage-page__header :deep(.page-header-card__content) {
  padding: 1.1rem 1.25rem;
}

.usage-page__header :deep(.page-header-card__top) {
  gap: 0.9rem;
}

.usage-page__header :deep(.page-header-card__intro) {
  gap: 0.85rem;
}

.usage-page__header :deep(.page-header-card__icon) {
  width: 2.75rem;
  height: 2.75rem;
  border-radius: 0.9rem;
}

.usage-page__header :deep(.page-header-card__title) {
  font-size: 1.32rem;
}

.usage-page__header :deep(.page-header-card__description) {
  margin-top: 0.28rem;
  max-width: 42rem;
  font-size: 0.9rem;
  line-height: 1.45;
}

.usage-page__header :deep(.page-header-card__meta) {
  margin-top: 0.65rem;
  gap: 0.45rem;
}

.usage-page__header :deep(.page-header-card__actions) {
  gap: 0.55rem;
  align-items: flex-start;
}

@media (width >= 1024px) {
  .usage-page__header :deep(.page-header-card__top) {
    align-items: flex-start;
  }

  .usage-page__header :deep(.page-header-card__actions) {
    justify-content: flex-end;
  }
}
</style>
