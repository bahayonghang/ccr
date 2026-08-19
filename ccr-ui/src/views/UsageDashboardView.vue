<!-- eslint-disable vue/no-template-shadow -->
<template>
  <PageShell class="usage-page">
    <template #header>
      <PageHeader
        :title="$t('usage.title')"
        :description="$t('usage.subtitle')"
      >
        <template
          v-if="costSummaryCard"
          #status
        >
          <StatTile
            :label="costSummaryCard.label"
            :value="costSummaryCard.value"
            :hint="costSummaryCard.detail"
          />
        </template>
      </PageHeader>
    </template>

    <div class="usage-shell">
      <UsageDashboardToolbar
        :selected-platform="selectedPlatform"
        :selected-range="selectedRange"
        :import-button-label="importButtonLabel"
        :importing="store.importing"
        :runtime-unavailable="runtimeUnavailable"
        :meta-items="!runtimeUnavailable && dashboardReady ? dashboardMetaItems : []"
        @update:selected-platform="updateSelectedPlatform"
        @update:selected-range="updateSelectedRange"
        @import="doImport"
      />

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
        <UsageStaleBanner
          v-if="dashboardReady"
          :presentation="opsCockpit"
          @primary-action="handleOpsPrimaryAction"
          @secondary-action="diagnosticsOpen = true"
        />

        <section
          v-if="dashboardReady && costSummaryCard"
          class="usage-hero-row"
        >
          <UsageCostConclusionCard :card="costSummaryCard">
            <UsageTokenBreakdownStrip
              v-if="store.summary"
              :summary="store.summary"
              :cache-creation-tokens="cacheCreationTokens"
            />
          </UsageCostConclusionCard>

          <div
            v-if="otherSummaryCards.length > 0"
            class="usage-metric-grid"
          >
            <UsageMetricCard
              v-for="card in otherSummaryCards"
              :key="card.id"
              :card="card"
            />
          </div>
        </section>

        <div class="usage-workspace-switcher">
          <PillToggleGroup
            :options="tabToggleOptions"
            :model-value="activeTab"
            @update:model-value="activeTab = $event"
          />
          <p class="usage-workspace-switcher__summary">
            {{ selectedPlatformLabel }} · {{ selectedWindowLabel }}
          </p>
        </div>

        <AsyncStatePanel
          v-if="store.loading && !hasDashboardData"
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
          :class="{ 'usage-content--busy': store.loading }"
          :aria-busy="store.loading"
        >
          <AsyncStatePanel
            v-if="showEmptyState"
            state="empty"
            :title="emptyStateTitle"
            :description="emptyStateDescription"
            compact
          />

          <KeepAlive
            v-else
            :max="4"
          >
            <component :is="activeTabComponent" />
          </KeepAlive>
        </div>
      </template>
    </div>

    <!-- llmusage Install Dialog -->
    <LlmusageInstallDialog
      v-model:is-open="showInstallDialog"
      @retry-import="doImportAfterInstall"
    />

    <UsageDiagnosticsDrawer
      v-model="diagnosticsOpen"
      :presentation="opsCockpit"
      @refresh="doImport"
    />
  </PageShell>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, ref, type Component } from 'vue'
import { useI18n } from 'vue-i18n'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import PillToggleGroup from '@/components/ui/PillToggleGroup.vue'
import StatTile from '@/components/ui/StatTile.vue'
import UsageCostConclusionCard from '@/components/usage/UsageCostConclusionCard.vue'
import UsageDashboardToolbar from '@/components/usage/UsageDashboardToolbar.vue'
import UsageDiagnosticsDrawer from '@/components/usage/UsageDiagnosticsDrawer.vue'
import UsageLogsTab from '@/components/usage/UsageLogsTab.vue'
import UsageMetricCard from '@/components/usage/UsageMetricCard.vue'
import UsageModelsTab from '@/components/usage/UsageModelsTab.vue'
import UsageOverviewTab from '@/components/usage/UsageOverviewTab.vue'
import UsageProjectsTab from '@/components/usage/UsageProjectsTab.vue'
import UsageProvidersTab from '@/components/usage/UsageProvidersTab.vue'
import UsageStaleBanner from '@/components/usage/UsageStaleBanner.vue'
import UsageTokenBreakdownStrip from '@/components/usage/UsageTokenBreakdownStrip.vue'
import type { UsageRangePreset } from './usage/dateWindow'
import { perfMark, perfMeasure } from '@/utils/perfTelemetry'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { useUsageDashboardState } from './usage/useUsageDashboardState'
import {
  createUsageDashboardContext,
  provideUsageDashboardContext,
} from './usage/usageDashboardContext'

const apexchart = defineAsyncComponent(async () => {
  perfMark('usage_chart_import_start')
  const module = await import('@/utils/apexChartsCore')
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

// KeepAlive 按组件标识缓存，映射表内的组件引用必须稳定（模块级 import / 模块级 async 包装），
// 不能在渲染期重建，否则每次切换都被判为新组件而丢失缓存。
const tabComponents: Record<string, Component> = {
  overview: UsageOverviewTab,
  tokens: UsageTokensTab,
  cost: UsageCostTab,
  providers: UsageProvidersTab,
  models: UsageModelsTab,
  projects: UsageProjectsTab,
  logs: UsageLogsTab,
}

const usage = useUsageDashboardState()
const {
  activeTab,
  cacheCreationTokens,
  dashboardReady,
  dashboardMetaItems,
  doImport,
  doImportAfterInstall,
  emptyStateDescription,
  emptyStateTitle,
  importButtonLabel,
  handleOpsPrimaryAction,
  onFilterChange,
  opsCockpit,
  runtimeUnavailable,
  selectedPlatform,
  selectedPlatformLabel,
  selectedRange,
  selectedWindowLabel,
  showEmptyState,
  showInstallDialog,
  store,
  summaryCards,
  tabKeys,
  unsupportedStateDescription,
  unsupportedStateTitle,
} = usage

const updateSelectedPlatform = (value: string) => {
  selectedPlatform.value = value
  onFilterChange()
}

const updateSelectedRange = (value: UsageRangePreset) => {
  selectedRange.value = value
  onFilterChange()
}

const diagnosticsOpen = ref(false)
const { t } = useI18n()

const tabToggleOptions = computed(() =>
  tabKeys.map((tab) => ({
    value: tab,
    label: t(`usage.dashboard.tabs.${tab}`),
  })),
)

// 费用结论卡拎出 cost 卡单独放大展示，其余 3 张沿用 UsageMetricCard 排进右侧 2 列指标格。
const costSummaryCard = computed(() => summaryCards.value.find((card) => card.id === 'cost') ?? null)
const otherSummaryCards = computed(() => summaryCards.value.filter((card) => card.id !== 'cost'))

// usage 域 provide/inject 上下文：7 个 tab 子组件 inject 取用，替代逐 tab 8~20 个 props 透传。
provideUsageDashboardContext(
  createUsageDashboardContext(usage, {
    chartComponent: apexchart,
    updateSelectedPlatform,
  })
)

// 刷新期间保持内容挂载：仅在“尚无可渲染数据”（首屏 / 切平台进空库）时让 loading 面板接管；
// 已有内容的刷新交给各图表的 memoized options → updateSeries 快路径，不再整树重挂。
const hasDashboardData = computed(() => store.summary != null || store.trends.length > 0)

const activeTabComponent = computed(() => tabComponents[activeTab.value] ?? UsageOverviewTab)

const runtimeCopy = computed(() => getRuntimeUnavailableCopy('usage'))
</script>

<style scoped>
.usage-page {
  min-width: 0;
}

.usage-shell {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}

.usage-hero-row {
  display: grid;
  gap: 0.8rem;
  grid-template-columns: minmax(0, 7fr) minmax(0, 5fr);
  align-items: stretch;
}

.usage-metric-grid {
  display: grid;
  gap: 0.8rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-content: start;
}

.usage-workspace-switcher {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
}

.usage-workspace-switcher__summary {
  color: var(--color-text-secondary);
  font-size: 0.84rem;
}

.usage-content {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
  transition: opacity var(--motion-subtle-duration) var(--motion-subtle-ease);
}

/* 刷新进行中（已有内容）时的轻量忙碌提示：仅淡出，不搭骨架屏（骨架屏属清单第 7 项）。 */
.usage-content--busy {
  opacity: 0.8;
}

td {
  font-variant-numeric: tabular-nums;
}

@media (width < 1280px) {
  .usage-hero-row {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (width < 900px) {
  .usage-metric-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .usage-workspace-switcher {
    align-items: flex-start;
  }
}
</style>
