<template>
  <div class="usage-insight-panel">
    <!-- 步骤1：标题 + 订阅 banner -->
    <header class="usage-insight-panel__head">
      <div class="usage-insight-panel__copy">
        <p class="usage-insight-panel__eyebrow">
          {{ $t('claudeCode.observer.eyebrow') }}
        </p>
        <h2 class="usage-insight-panel__title">
          {{ $t('claudeCode.observer.title') }}
        </h2>
        <p class="usage-insight-panel__subtitle">
          {{ pricingNote }}
        </p>
      </div>

      <div class="usage-insight-panel__head-actions">
        <RouterLink
          to="/usage"
          class="usage-insight-panel__full-link"
        >
          {{ $t('claudeCode.observer.fullDashboardLink') }}
        </RouterLink>
      </div>
    </header>

    <Card
      v-if="showSubscriptionBanner"
      variant="outline"
      padding="sm"
      class="usage-insight-panel__banner"
    >
      <div class="usage-insight-panel__banner-row">
        <div class="usage-insight-panel__banner-text">
          <SIcon
            name="Sparkles"
            size="w-4 h-4"
            class="usage-insight-panel__banner-icon"
          />
          <span>{{ subscriptionBannerText }}</span>
        </div>
        <button
          type="button"
          class="usage-insight-panel__icon-btn"
          :title="$t('claudeCode.observer.subscription.openDialog')"
          @click="dialogOpen = true"
        >
          <SIcon
            name="Settings"
            size="w-4 h-4"
          />
        </button>
      </div>
    </Card>

    <!-- 步骤2：紧凑订阅切换入口（非订阅模式下也能进对话框） -->
    <div
      v-else
      class="usage-insight-panel__compact-toggle"
    >
      <button
        type="button"
        class="usage-insight-panel__icon-btn usage-insight-panel__icon-btn--inline"
        :title="$t('claudeCode.observer.subscription.openDialog')"
        @click="dialogOpen = true"
      >
        <SIcon
          name="Settings"
          size="w-4 h-4"
        />
        <span>{{ $t('claudeCode.observer.subscription.openDialog') }}</span>
      </button>
    </div>

    <!-- 步骤3：状态切换 -->
    <AsyncStatePanel
      v-if="state === 'loading'"
      state="loading"
      :title="$t('claudeCode.observer.loading')"
      compact
    />
    <AsyncStatePanel
      v-else-if="state === 'error'"
      state="error"
      :title="$t('claudeCode.observer.errorTitle')"
      :description="loadError ?? ''"
      :action-label="$t('common.retry')"
      action-icon="RefreshCw"
      @action="refresh"
    />
    <AsyncStatePanel
      v-else-if="state === 'empty'"
      state="empty"
      :title="$t('claudeCode.observer.empty.noUsage')"
      :description="emptyDescription"
      icon="Database"
      :action-label="$t('claudeCode.observer.empty.openFullDashboard')"
      action-icon="ArrowUpRight"
      @action="goToUsage"
    />

    <template v-else>
      <!-- 步骤4：3 hero stat card -->
      <section class="usage-insight-panel__hero-grid">
        <article class="usage-insight-panel__hero-card">
          <p class="usage-insight-panel__hero-label">
            {{ $t('claudeCode.observer.metric.todayValue') }}
          </p>
          <p class="usage-insight-panel__hero-value">
            {{ formatUsd(insight?.today_value_usd ?? 0) }}
          </p>
          <p class="usage-insight-panel__hero-detail">
            {{ formatTokens(insight?.today_tokens ?? 0) }}
            {{ $t('claudeCode.observer.metric.tokensUnit') }}
          </p>
        </article>

        <article class="usage-insight-panel__hero-card usage-insight-panel__hero-card--accent">
          <p class="usage-insight-panel__hero-label">
            {{ $t('claudeCode.observer.metric.monthValue') }}
          </p>
          <p class="usage-insight-panel__hero-value">
            {{ formatUsd(insight?.month_value_usd ?? 0) }}
          </p>
          <p class="usage-insight-panel__hero-detail">
            <template v-if="hasRoi">
              {{ $t('claudeCode.observer.metric.monthValueDetail', {
                monthly: formatUsd(subscription?.monthly_usd ?? 0),
                roi: formatRoi(insight?.roi ?? null),
              }) }}
            </template>
            <template v-else>
              {{ formatTokens(insight?.month_tokens ?? 0) }}
              {{ $t('claudeCode.observer.metric.tokensUnit') }}
            </template>
          </p>
        </article>

        <article class="usage-insight-panel__hero-card">
          <p class="usage-insight-panel__hero-label">
            {{ $t('claudeCode.observer.metric.totalValue') }}
          </p>
          <p class="usage-insight-panel__hero-value">
            {{ formatUsd(insight?.total_value_usd ?? 0) }}
          </p>
          <p class="usage-insight-panel__hero-detail">
            {{ $t('claudeCode.observer.metric.totalValueDetail', {
              sessions: insight?.total_sessions ?? 0,
              projects: insight?.total_projects ?? 0,
            }) }}
          </p>
        </article>
      </section>

      <!-- 步骤5：Tab 切换 -->
      <div class="usage-insight-panel__tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="usage-insight-panel__tab"
          :class="{ 'usage-insight-panel__tab--active': activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </div>

      <div class="usage-insight-panel__tab-content">
        <CostAttributionTab
          v-if="activeTab === 'cost'"
          :daily="store.daily.data ?? []"
          :by-project="store.costByProject.data ?? []"
          :by-model="store.costByModel.data ?? []"
          :animations-enabled="animationsEnabled"
          :should-render-chart="shouldRenderChart"
        />
        <TokenDetailTab
          v-else-if="activeTab === 'token'"
          :stats="store.cacheStats.data"
          :daily="store.daily.data ?? []"
          :animations-enabled="animationsEnabled"
          :should-render-chart="shouldRenderChart"
        />
        <BehaviorAnalysisTab
          v-else
          :heatmap="store.heatmap.data ?? []"
          :top-tools="store.topTools.data ?? []"
          :sessions="store.topSessions.data ?? []"
          :animations-enabled="animationsEnabled"
          :should-render-chart="shouldRenderChart"
        />
      </div>
    </template>

    <SubscriptionDialog
      v-model="dialogOpen"
      :current="subscription"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import CostAttributionTab from './CostAttributionTab.vue'
import TokenDetailTab from './TokenDetailTab.vue'
import BehaviorAnalysisTab from './BehaviorAnalysisTab.vue'
import { useClaudeObserverStore } from '@/stores/claudeObserver'
import { formatTokens } from './formatters'

// 订阅对话框比较重，按需异步加载
const SubscriptionDialog = defineAsyncComponent({
  loader: () => import('./SubscriptionDialog.vue'),
  suspensible: false,
})

const router = useRouter()
const { t } = useI18n()
const store = useClaudeObserverStore()

/*
 * ========================================================================
 * 步骤1：本地状态
 * ========================================================================
 */
type TabId = 'cost' | 'token' | 'behavior'
const activeTab = ref<TabId>('cost')
const dialogOpen = ref(false)

const tabs = computed(() => [
  { id: 'cost' as TabId, label: t('claudeCode.observer.tab.cost') },
  { id: 'token' as TabId, label: t('claudeCode.observer.tab.token') },
  { id: 'behavior' as TabId, label: t('claudeCode.observer.tab.behavior') },
])

const insight = computed(() => store.insight.data)
const subscription = computed(() => store.subscription.data ?? insight.value?.subscription ?? null)

/*
 * ========================================================================
 * 步骤2：派生：状态机
 * ========================================================================
 * 1) 任一关键切片 loading -> 'loading'
 * 2) 关键切片 error -> 'error'
 * 3) 全部为 null 且 insight 已加载且 today/month/total 都是 0 -> 'empty'
 */
type PanelState = 'loading' | 'error' | 'empty' | 'ready'

const loadError = computed(() => {
  return (
    store.insight.error
    || store.cacheStats.error
    || store.daily.error
    || store.costByProject.error
    || store.costByModel.error
    || null
  )
})

const isLoading = computed(() =>
  store.insight.loading
  && !store.insight.data,
)

const isEmpty = computed(() => {
  const i = store.insight.data
  if (!i) return false
  return (
    (i.total_value_usd ?? 0) === 0
    && (i.month_value_usd ?? 0) === 0
    && (i.today_value_usd ?? 0) === 0
    && (i.total_sessions ?? 0) === 0
  )
})

const state = computed<PanelState>(() => {
  if (isLoading.value) return 'loading'
  if (loadError.value && !store.insight.data) return 'error'
  if (isEmpty.value) return 'empty'
  return 'ready'
})

/*
 * ========================================================================
 * 步骤3：订阅 banner
 * ========================================================================
 */
const showSubscriptionBanner = computed(() => subscription.value?.mode === 'subscription')

const hasRoi = computed(
  () => subscription.value?.mode === 'subscription'
    && Number.isFinite(insight.value?.roi ?? null)
    && (insight.value?.roi ?? 0) > 0,
)

const subscriptionBannerText = computed(() => {
  return t('claudeCode.observer.subscription.banner', {
    monthly: formatUsd(subscription.value?.monthly_usd ?? 0),
    roi: formatRoi(insight.value?.roi ?? null),
  })
})

const pricingNote = computed(() => {
  const version = insight.value?.pricing_version
  if (!version) {
    return t('claudeCode.observer.subtitle')
  }
  return t('claudeCode.observer.subtitleWithVersion', { version })
})

const emptyDescription = computed(() => {
  // 如果有错误，显示错误信息
  if (loadError.value) {
    return `${t('claudeCode.observer.empty.loadError')}: ${loadError.value}`
  }
  // 否则显示友好提示
  return t('claudeCode.observer.empty.noUsageDesc')
})

/*
 * ========================================================================
 * 步骤4：动画开关
 * ========================================================================
 * 1) 监听 prefers-reduced-motion
 * 2) Tab 是否可渲染图表（首次激活后才挂载）
 */
const animationsEnabled = ref(true)
const renderedTabs = ref(new Set<TabId>())

const updateMotionPreference = () => {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    animationsEnabled.value = true
    return
  }
  animationsEnabled.value = !window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

const shouldRenderChart = computed(() => renderedTabs.value.has(activeTab.value))

let motionMql: MediaQueryList | null = null
const handleMotionChange = () => updateMotionPreference()

onMounted(async () => {
  updateMotionPreference()
  if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
    motionMql = window.matchMedia('(prefers-reduced-motion: reduce)')
    motionMql.addEventListener?.('change', handleMotionChange)
  }
  renderedTabs.value.add('cost')
  // 4.1 启动事件订阅 + 拉一遍数据
  await store.startEventListener()
  await store.fetchAll()
})

onBeforeUnmount(async () => {
  if (motionMql) {
    motionMql.removeEventListener?.('change', handleMotionChange)
    motionMql = null
  }
  await store.disposeEventListener()
})

// 4.2 切 Tab 时记录是否已渲染过（首次进入才挂图表，避免不可见时白浪费动画）
watch(activeTab, (next) => {
  renderedTabs.value.add(next)
})

/*
 * ========================================================================
 * 步骤5：动作
 * ========================================================================
 */
const refresh = () => {
  void store.fetchAll()
}

const goToUsage = () => {
  void router.push('/usage')
}

/*
 * ========================================================================
 * 工具函数
 * ========================================================================
 */
const formatUsd = (value: number) => {
  if (!Number.isFinite(value)) return '$0.00'
  if (value >= 1000) return `$${value.toFixed(0)}`
  if (value >= 1) return `$${value.toFixed(2)}`
  return `$${value.toFixed(4)}`
}

const formatRoi = (roi: number | null) => {
  if (roi === null || !Number.isFinite(roi)) return '-'
  return `${roi.toFixed(1)}×`
}
</script>

<style scoped>
.usage-insight-panel {
  display: grid;
  gap: 0.85rem;
}

.usage-insight-panel__head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 0.9rem;
  flex-wrap: wrap;
}

.usage-insight-panel__copy {
  display: grid;
  gap: 0.18rem;
  min-width: 0;
}

.usage-insight-panel__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.usage-insight-panel__title {
  color: var(--color-text-primary);
  font-size: 1.4rem;
  font-weight: 650;
}

.usage-insight-panel__subtitle {
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  line-height: 1.5;
}

.usage-insight-panel__head-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.usage-insight-panel__full-link {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  border-radius: 0.65rem;
  border: 1px solid var(--color-border-default);
  padding: 0.45rem 0.85rem;
  color: var(--color-text-secondary);
  font-size: 0.8rem;
  font-weight: 600;
  text-decoration: none;
  transition: color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-insight-panel__full-link:hover {
  color: var(--color-text-primary);
  border-color: rgb(var(--color-accent-primary-rgb) / 30%);
  background: rgb(var(--color-accent-primary-rgb) / 6%);
}

.usage-insight-panel__banner {
  width: 100%;
}

.usage-insight-panel__banner-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.7rem;
}

.usage-insight-panel__banner-text {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  color: var(--color-text-primary);
  font-size: 0.82rem;
  font-weight: 600;
}

.usage-insight-panel__banner-icon {
  color: var(--color-accent-primary);
  flex-shrink: 0;
}

.usage-insight-panel__compact-toggle {
  display: flex;
  justify-content: flex-end;
}

.usage-insight-panel__icon-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border-radius: 0.6rem;
  border: 1px solid var(--color-border-default);
  background: transparent;
  color: var(--color-text-secondary);
  padding: 0.35rem 0.55rem;
  font-size: 0.78rem;
  transition: color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-insight-panel__icon-btn:hover {
  color: var(--color-text-primary);
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
}

.usage-insight-panel__icon-btn--inline {
  padding: 0.35rem 0.7rem;
}

.usage-insight-panel__hero-grid {
  display: grid;
  gap: 0.7rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.usage-insight-panel__hero-card {
  position: relative;
  border-radius: 1.1rem;
  border: 1px solid var(--surface-card-border);
  background: var(--surface-card-bg);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--surface-card-shadow);
  padding: 0.95rem 1.1rem;
  display: grid;
  gap: 0.25rem;
}

.usage-insight-panel__hero-card--accent {
  border-color: rgb(var(--color-accent-primary-rgb) / 22%);
  background: linear-gradient(
    160deg,
    rgb(var(--color-accent-primary-rgb) / 8%),
    var(--surface-card-bg) 65%
  );
}

.usage-insight-panel__hero-label {
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.usage-insight-panel__hero-value {
  color: var(--color-text-primary);
  font-size: 1.85rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
  line-height: 1.15;
}

.usage-insight-panel__hero-detail {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
}

.usage-insight-panel__tabs {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}

.usage-insight-panel__tab {
  border-radius: 0.95rem;
  border: 1px solid transparent;
  background: transparent;
  padding: 0.5rem 0.85rem;
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-insight-panel__tab:hover {
  color: var(--color-text-primary);
  background: var(--surface-status-bg);
}

.usage-insight-panel__tab--active {
  color: var(--color-text-primary);
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  background: linear-gradient(
    180deg,
    rgb(var(--color-bg-elevated-rgb) / 92%),
    rgb(var(--color-bg-surface-rgb) / 84%)
  );
  box-shadow: 0 6px 18px rgb(var(--color-accent-primary-rgb) / 6%);
}

.usage-insight-panel__tab-content {
  min-width: 0;
}

@media (width < 900px) {
  .usage-insight-panel__hero-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .usage-insight-panel__head {
    flex-direction: column;
    align-items: flex-start;
  }
}

@media (prefers-reduced-motion: reduce) {
  .usage-insight-panel__tab,
  .usage-insight-panel__full-link,
  .usage-insight-panel__icon-btn {
    transition: none;
  }
}
</style>
