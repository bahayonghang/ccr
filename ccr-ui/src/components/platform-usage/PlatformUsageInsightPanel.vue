<template>
  <section
    class="platform-usage-panel"
    :class="`platform-usage-panel--${spec.tone}`"
    :aria-label="spec.title"
    data-testid="platform-usage-insight"
  >
    <div class="platform-usage-panel__header">
      <div class="platform-usage-panel__copy">
        <p class="platform-usage-panel__eyebrow">
          {{ spec.eyebrow }}
        </p>
        <h2>{{ spec.title }}</h2>
        <p>{{ spec.description }}</p>
      </div>

      <div class="platform-usage-panel__actions">
        <div class="platform-usage-panel__chips">
          <span>{{ spec.windowLabel }}</span>
          <span>{{ spec.sourceLabel }}</span>
          <span v-if="state.generatedAt">{{ generatedLabel }}</span>
        </div>
        <div class="platform-usage-panel__buttons">
          <button
            type="button"
            class="platform-usage-panel__button"
            :disabled="loading"
            @click="$emit('refresh')"
          >
            <SIcon
              name="RefreshCw"
              size="w-4 h-4"
              :class="{ 'animate-spin': loading }"
            />
            {{ spec.retryLabel }}
          </button>
          <RouterLink
            class="platform-usage-panel__button platform-usage-panel__button--primary"
            :to="spec.primaryActionTo"
          >
            {{ spec.primaryActionLabel }}
            <SIcon
              name="ArrowUpRight"
              size="w-4 h-4"
            />
          </RouterLink>
        </div>
      </div>
    </div>

    <div
      v-if="isInitialLoading"
      class="platform-usage-panel__skeleton-grid"
      aria-hidden="true"
    >
      <div
        v-for="index in 3"
        :key="index"
        class="platform-usage-panel__skeleton"
      />
    </div>

    <div
      v-else-if="error && state.empty"
      class="platform-usage-panel__notice platform-usage-panel__notice--error"
      role="status"
    >
      <SIcon
        name="CircleAlert"
        size="w-5 h-5"
      />
      <div>
        <strong>{{ spec.errorTitle }}</strong>
        <span>{{ error }}</span>
      </div>
    </div>

    <div
      v-else-if="state.empty"
      class="platform-usage-panel__notice"
      role="status"
    >
      <SIcon
        name="Database"
        size="w-5 h-5"
      />
      <div>
        <strong>{{ spec.emptyTitle }}</strong>
        <span>{{ spec.emptyDescription }}</span>
      </div>
    </div>

    <template v-else>
      <div
        v-if="error"
        class="platform-usage-panel__notice platform-usage-panel__notice--inline"
        role="status"
      >
        <SIcon
          name="WifiOff"
          size="w-4 h-4"
        />
        <span>{{ error }}</span>
      </div>

      <div class="platform-usage-panel__kpis">
        <article
          v-for="card in state.cards"
          :key="card.id"
          class="platform-usage-panel__kpi"
          :class="[
            `platform-usage-panel__kpi--${card.id}`,
            { 'platform-usage-panel__kpi--token-only': card.pricingState === 'token_only' },
          ]"
        >
          <div class="platform-usage-panel__kpi-icon">
            <SIcon
              :name="card.icon"
              size="w-5 h-5"
            />
          </div>
          <div>
            <span>{{ card.label }}</span>
            <strong>{{ card.value }}</strong>
            <p>{{ card.detail }}</p>
            <small>{{ card.meta }}</small>
          </div>
        </article>
      </div>

      <div
        class="platform-usage-panel__tabs"
        role="tablist"
        :aria-label="`${spec.label} usage metrics`"
      >
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          role="tab"
          class="platform-usage-panel__tab"
          :class="{ 'platform-usage-panel__tab--active': activeTab === tab.id }"
          :aria-selected="activeTab === tab.id"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </div>

      <div
        v-if="activeTab !== 'breakdown'"
        class="platform-usage-panel__chart-row"
      >
        <PlatformUsageTrendChart
          :metric="activeMetric"
          :trends="state.trends"
          :title="activeChartLabel"
          :eyebrow="spec.label"
          :window-label="spec.windowLabel"
          :empty-label="spec.emptyDescription"
        />
      </div>

      <div class="platform-usage-panel__rank-grid">
        <PlatformUsageRankList
          :title="spec.modelRankTitle"
          :eyebrow="state.topModelLabel"
          :rows="state.modelRows"
          :empty-label="spec.emptyDescription"
        />
        <PlatformUsageRankList
          :title="spec.projectRankTitle"
          :eyebrow="state.topProjectLabel"
          :rows="state.projectRows"
          :empty-label="spec.emptyDescription"
        />
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import SIcon from '@/components/ui/SIcon.vue'
import PlatformUsageRankList from './PlatformUsageRankList.vue'
import PlatformUsageTrendChart from './PlatformUsageTrendChart.vue'
import type {
  PlatformUsageInsightPresentation,
  PlatformUsageInsightSpec,
  PlatformUsageMetric,
} from '@/types/platformUsageInsight'

const props = defineProps<{
  spec: PlatformUsageInsightSpec
  state: PlatformUsageInsightPresentation
  loading?: boolean
  error?: string | null
}>()

defineEmits<{
  refresh: []
}>()

type UsagePanelTab = PlatformUsageMetric | 'breakdown'

const activeTab = ref<UsagePanelTab>('cost')
const tabs = computed(() => [
  { id: 'cost' as const, label: props.spec.tabs.cost },
  { id: 'tokens' as const, label: props.spec.tabs.tokens },
  { id: 'requests' as const, label: props.spec.tabs.requests },
  { id: 'breakdown' as const, label: props.spec.tabs.breakdown },
])
const tabsById = computed(() =>
  Object.fromEntries(tabs.value.map((tab) => [tab.id, tab])) as Record<UsagePanelTab, { id: UsagePanelTab, label: string }>,
)
const activeMetric = computed<PlatformUsageMetric>(() =>
  activeTab.value === 'breakdown' ? 'cost' : activeTab.value,
)
const activeChartLabel = computed(() => tabsById.value[activeMetric.value].label)
const isInitialLoading = computed(() => props.loading && props.state.cards.length === 0)
const generatedLabel = computed(() =>
  props.state.generatedAt
    ? new Intl.DateTimeFormat(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    }).format(new Date(props.state.generatedAt))
    : '',
)
</script>

<style scoped>
.platform-usage-panel {
  position: relative;
  overflow: hidden;
  display: grid;
  gap: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  border-radius: 1.75rem;
  background: var(--color-bg-surface);
  padding: clamp(1rem, 2vw, 1.45rem);
  box-shadow: var(--elevation-1);
  backdrop-filter: var(--surface-workspace-blur);
}

.platform-usage-panel--codex {
  --platform-usage-accent-rgb: var(--color-success-rgb);
}

.platform-usage-panel--antigravity {
  --platform-usage-accent-rgb: var(--color-info-rgb);
}

.platform-usage-panel--opencode {
  --platform-usage-accent-rgb: var(--color-warning-rgb);
}

.platform-usage-panel__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.platform-usage-panel__copy {
  display: grid;
  gap: 0.35rem;
  max-width: 52rem;
}

.platform-usage-panel__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 780;
  letter-spacing: 0.13em;
  text-transform: uppercase;
}

.platform-usage-panel__copy h2 {
  color: var(--color-text-primary);
  font-size: clamp(1.2rem, 1.2vw + 0.95rem, 1.62rem);
  font-weight: 760;
  letter-spacing: -0.035em;
  line-height: 1.05;
}

.platform-usage-panel__copy p:not(.platform-usage-panel__eyebrow) {
  color: var(--color-text-secondary);
  font-size: 0.88rem;
  line-height: 1.5;
}

.platform-usage-panel__actions {
  display: grid;
  gap: 0.66rem;
  justify-items: end;
  flex: none;
}

.platform-usage-panel__chips,
.platform-usage-panel__buttons {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.45rem;
}

.platform-usage-panel__chips span {
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 999px;
  background: rgb(var(--color-bg-elevated-rgb) / 44%);
  padding: 0.26rem 0.58rem;
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 700;
}

.platform-usage-panel__button {
  display: inline-flex;
  min-height: 2.25rem;
  align-items: center;
  justify-content: center;
  gap: 0.42rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 16%);
  border-radius: 999px;
  background: rgb(var(--color-bg-elevated-rgb) / 58%);
  padding: 0 0.8rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  font-weight: 720;
  transition:
    transform var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.platform-usage-panel__button:hover:not(:disabled) {
  border-color: rgb(var(--platform-usage-accent-rgb, var(--color-accent-primary-rgb)) / 26%);
  color: var(--color-text-primary);
  transform: translateY(-1px);
}

.platform-usage-panel__button:disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.platform-usage-panel__button--primary {
  background: rgb(var(--platform-usage-accent-rgb, var(--color-accent-primary-rgb)) / 12%);
  color: var(--color-text-primary);
}

.platform-usage-panel__skeleton-grid,
.platform-usage-panel__kpis {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.8rem;
}

.platform-usage-panel__skeleton {
  min-height: 8.5rem;
  border-radius: 1.15rem;
  background: rgb(var(--color-border-default-rgb) / 10%);
}

.platform-usage-panel__kpi {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 0.8rem;
  min-width: 0;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 1.2rem;
  background: rgb(var(--color-bg-elevated-rgb) / 48%);
  padding: 0.95rem;
}

.platform-usage-panel__kpi--cost {
  --kpi-icon-rgb: var(--color-accent-primary-rgb);

  background:
    radial-gradient(circle at 12% 0%, rgb(var(--platform-usage-accent-rgb, var(--color-accent-primary-rgb)) / 12%), transparent 16rem),
    rgb(var(--color-bg-elevated-rgb) / 54%);
}

.platform-usage-panel__kpi--tokens {
  --kpi-icon-rgb: var(--color-info-rgb);
}

.platform-usage-panel__kpi--requests {
  --kpi-icon-rgb: var(--color-accent-secondary-rgb);
}

.platform-usage-panel__kpi--token-only {
  border-color: rgb(var(--color-warning-rgb) / 20%);
}

.platform-usage-panel__kpi-icon {
  display: grid;
  place-items: center;
  width: 2.15rem;
  height: 2.15rem;
  border-radius: 0.85rem;
  border: 1px solid rgb(var(--kpi-icon-rgb, var(--color-accent-primary-rgb)) / 14%);
  background: rgb(var(--kpi-icon-rgb, var(--color-accent-primary-rgb)) / 10%);
  color: rgb(var(--kpi-icon-rgb, var(--color-accent-primary-rgb)));
}

.platform-usage-panel__kpi span,
.platform-usage-panel__kpi small {
  display: block;
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 720;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.platform-usage-panel__kpi strong {
  display: block;
  margin-top: 0.34rem;
  color: var(--color-text-primary);
  font-size: clamp(1.34rem, 1.2vw + 1rem, 2rem);
  font-variant-numeric: tabular-nums;
  font-weight: 760;
  letter-spacing: -0.04em;
  line-height: 1;
}

.platform-usage-panel__kpi p {
  margin-top: 0.38rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
}

.platform-usage-panel__kpi small {
  margin-top: 0.38rem;
  letter-spacing: 0;
  text-transform: none;
}

.platform-usage-panel__tabs {
  display: inline-flex;
  width: fit-content;
  max-width: 100%;
  gap: 0.22rem;
  overflow-x: auto;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  border-radius: 999px;
  background: rgb(var(--color-bg-elevated-rgb) / 44%);
  padding: 0.22rem;
}

.platform-usage-panel__tab {
  flex: none;
  min-height: 2.05rem;
  border-radius: 999px;
  padding: 0 0.76rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  font-weight: 690;
}

.platform-usage-panel__tab--active {
  background: rgb(var(--platform-usage-accent-rgb, var(--color-accent-primary-rgb)) / 13%);
  color: var(--color-text-primary);
  box-shadow: inset 0 0 0 1px rgb(var(--platform-usage-accent-rgb, var(--color-accent-primary-rgb)) / 14%);
}

.platform-usage-panel__chart-row {
  min-width: 0;
}

.platform-usage-panel__rank-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.8rem;
}

.platform-usage-panel__notice {
  display: flex;
  align-items: flex-start;
  gap: 0.7rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  border-radius: 1.15rem;
  background: rgb(var(--color-bg-elevated-rgb) / 48%);
  padding: 0.95rem;
  color: var(--color-text-secondary);
}

.platform-usage-panel__notice strong,
.platform-usage-panel__notice span {
  display: block;
}

.platform-usage-panel__notice strong {
  color: var(--color-text-primary);
  font-size: 0.92rem;
  font-weight: 740;
}

.platform-usage-panel__notice span {
  margin-top: 0.16rem;
  font-size: 0.82rem;
}

.platform-usage-panel__notice--error {
  border-color: rgb(var(--color-danger-rgb) / 22%);
}

.platform-usage-panel__notice--inline {
  align-items: center;
  padding: 0.66rem 0.8rem;
}

@keyframes platform-usage-shimmer {
  from {
    background-position: 200% 0;
  }

  to {
    background-position: -200% 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .platform-usage-panel__skeleton {
    animation: none;
  }

  .platform-usage-panel__button:hover:not(:disabled) {
    transform: none;
  }
}

@media (width < 960px) {
  .platform-usage-panel__header {
    flex-direction: column;
  }

  .platform-usage-panel__actions {
    justify-items: start;
  }

  .platform-usage-panel__chips,
  .platform-usage-panel__buttons {
    justify-content: flex-start;
  }

  .platform-usage-panel__skeleton-grid,
  .platform-usage-panel__kpis,
  .platform-usage-panel__rank-grid {
    grid-template-columns: 1fr;
  }
}
</style>
