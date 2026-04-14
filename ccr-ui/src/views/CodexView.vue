<template>
  <div class="codex-view stage-page">
    <div class="codex-shell">
      <section class="codex-grid codex-grid--hero">
        <Card
          variant="glass"
          class="codex-hero-card"
        >
          <div class="codex-hero-card__glow" />

          <div class="codex-hero-content">
            <div class="codex-hero-header">
              <div class="codex-hero-copy">
                <div class="codex-hero-title-row">
                  <div class="codex-hero-icon">
                    <SIcon
                      name="Code2"
                      size="w-6 h-6"
                      class="text-accent-primary"
                    />
                  </div>
                  <div>
                    <h1 class="codex-hero-title">
                      Codex
                    </h1>
                    <p class="codex-hero-subtitle">
                      先看当前账号、配置健康度和下一步，再进入细项管理。
                    </p>
                  </div>
                </div>

                <div class="codex-pill-row">
                  <span class="codex-pill codex-pill--primary"> workflow first </span>
                  <span class="codex-pill codex-pill--neutral">
                    {{ versionLabel }}
                  </span>
                  <span class="codex-pill codex-pill--info">
                    {{ currentProfileLabel }}
                  </span>
                </div>
              </div>

              <div class="codex-action-row">
                <RouterLink to="/codex/auth">
                  <Button
                    variant="glass"
                    size="sm"
                  >
                    <SIcon
                      name="KeyRound"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    账号
                  </Button>
                </RouterLink>
                <RouterLink to="/codex/sessions">
                  <Button
                    variant="glass"
                    size="sm"
                  >
                    <SIcon
                      name="MessagesSquare"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    会话
                  </Button>
                </RouterLink>
                <RouterLink to="/codex/profiles">
                  <Button
                    variant="glass"
                    size="sm"
                  >
                    <SIcon
                      name="Folders"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    Profiles
                  </Button>
                </RouterLink>
                <RouterLink to="/codex/agents">
                  <Button
                    variant="glass"
                    size="sm"
                  >
                    <SIcon
                      name="Bot"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    Agents
                  </Button>
                </RouterLink>
                <Button
                  variant="ghost"
                  size="sm"
                  :disabled="loading"
                  @click="refresh(true)"
                >
                  <SIcon
                    name="RefreshCw"
                    size="w-4 h-4"
                    class="mr-2"
                    :class="{ 'animate-spin': loading }"
                  />
                  刷新
                </Button>
              </div>
            </div>

            <div class="codex-hero-stats">
              <div class="codex-stat-card">
                <p class="codex-stat-label">
                  当前账号
                </p>
                <p
                  class="codex-stat-value truncate"
                  :title="currentAccountLabel"
                >
                  {{ currentAccountLabel }}
                </p>
              </div>
              <div class="codex-stat-card">
                <p class="codex-stat-label">
                  累计请求
                </p>
                <p class="codex-stat-value">
                  {{ usageTotalRequests }}
                </p>
              </div>
              <div class="codex-stat-card">
                <p class="codex-stat-label">
                  累计 Tokens
                </p>
                <p class="codex-stat-value">
                  {{ usageTotalTokens }}
                </p>
              </div>
            </div>
          </div>
        </Card>

        <Card
          variant="glass"
          class="codex-panel"
        >
          <div class="codex-panel-header">
            <div class="codex-panel-icon codex-panel-icon--amber">
              <SIcon
                name="Route"
                size="w-5 h-5"
                class="text-amber-300"
              />
            </div>
            <div>
              <h2 class="codex-panel-title">
                下一步
              </h2>
              <p class="codex-panel-subtitle">
                只保留最该先做的动作
              </p>
            </div>
          </div>

          <div
            v-if="nextActions.length"
            class="codex-stack"
          >
            <RouterLink
              v-for="action in nextActions"
              :key="action.title"
              :to="action.to"
              class="codex-action-card"
            >
              <div class="flex items-start gap-3">
                <div
                  class="codex-tone-icon"
                  :class="toneClassMap[action.tone]"
                >
                  <SIcon
                    :name="action.icon"
                    size="w-5 h-5"
                  />
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center justify-between gap-3">
                    <h3 class="codex-action-title">
                      {{ action.title }}
                    </h3>
                    <SIcon
                      name="ArrowRight"
                      size="w-4 h-4"
                      class="codex-action-arrow shrink-0"
                    />
                  </div>
                  <p class="codex-action-description">
                    {{ action.description }}
                  </p>
                </div>
              </div>
            </RouterLink>
          </div>

          <div
            v-else-if="overviewLoading"
            class="codex-stack"
          >
            <div class="codex-skeleton codex-skeleton--panel" />
            <div class="codex-skeleton codex-skeleton--panel" />
          </div>

          <EmptyState
            v-else
            icon="Route"
            title="正在准备下一步动作"
            description="概览数据尚未可用，可以先刷新一次重试。"
            action-text="立即刷新"
            action-icon="RefreshCw"
            :on-action="() => refresh(true)"
          />
        </Card>
      </section>

      <section class="codex-grid codex-grid--health">
        <template v-if="healthItems.length">
          <RouterLink
            v-for="item in healthItems"
            :key="item.key"
            :to="item.to"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="codex-health-card"
            >
              <div class="flex items-start justify-between gap-3">
                <div
                  class="codex-tone-icon codex-tone-icon--large"
                  :class="toneClassMap[item.tone]"
                >
                  <SIcon
                    :name="item.icon"
                    size="w-5 h-5"
                  />
                </div>
                <span class="codex-health-eyebrow"> 状态 </span>
              </div>
              <p class="codex-health-label">
                {{ item.title }}
              </p>
              <p class="codex-health-value">
                {{ item.value }}
              </p>
              <p class="codex-health-detail">
                {{ item.detail }}
              </p>
            </Card>
          </RouterLink>
        </template>

        <template v-else-if="overviewLoading">
          <Card
            v-for="n in 4"
            :key="`health-skeleton-${n}`"
            variant="elevated"
            class="codex-health-card"
          >
            <div class="codex-skeleton codex-skeleton--icon mb-4" />
            <div class="codex-skeleton codex-skeleton--line mb-3" />
            <div class="codex-skeleton codex-skeleton--line mb-2" />
            <div class="codex-skeleton codex-skeleton--detail" />
          </Card>
        </template>
      </section>

      <section class="codex-grid codex-grid--manage">
        <Card
          variant="glass"
          class="codex-panel codex-panel--wide"
        >
          <div class="codex-section-header">
            <div>
              <h2 class="codex-panel-title">
                管理入口
              </h2>
              <p class="codex-panel-description">
                把细项管理降级成次级入口，需要时再深入。
              </p>
            </div>
            <RouterLink
              to="/codex/settings"
              class="codex-text-link"
            >
              打开设置
            </RouterLink>
          </div>

          <div
            v-if="managementLinks.length"
            class="codex-grid codex-grid--links"
          >
            <RouterLink
              v-for="link in managementLinks"
              :key="link.to"
              :to="link.to"
              class="group"
            >
              <Card
                variant="glass"
                hover
                class="codex-link-card"
              >
                <div class="codex-link-card__header">
                  <div
                    class="codex-tone-icon"
                    :class="toneClassMap[link.tone]"
                  >
                    <SIcon
                      :name="link.icon"
                      size="w-5 h-5"
                    />
                  </div>
                  <span class="codex-link-badge">
                    {{ link.badge }}
                  </span>
                </div>
                <h3 class="codex-link-title">
                  {{ link.title }}
                </h3>
                <p class="codex-link-description">
                  {{ link.description }}
                </p>
              </Card>
            </RouterLink>
          </div>

          <div
            v-else-if="overviewLoading"
            class="codex-grid codex-grid--links"
          >
            <Card
              v-for="n in 6"
              :key="`link-skeleton-${n}`"
              variant="glass"
              class="codex-link-card"
            >
              <div class="codex-skeleton codex-skeleton--icon mb-4" />
              <div class="codex-skeleton codex-skeleton--line mb-3" />
              <div class="codex-skeleton codex-skeleton--detail" />
            </Card>
          </div>

          <EmptyState
            v-else
            icon="Folders"
            title="暂时还没有管理入口数据"
            description="可以先刷新一次，或者进入 Auth / Profiles 页面补齐基础配置。"
            action-text="立即刷新"
            action-icon="RefreshCw"
            :on-action="() => refresh(true)"
          />
        </Card>

        <Card
          variant="glass"
          class="codex-panel"
        >
          <div class="codex-panel-header">
            <div class="codex-panel-icon codex-panel-icon--indigo">
              <SIcon
                name="Sparkles"
                size="w-5 h-5"
                class="text-indigo-300"
              />
            </div>
            <div>
              <h2 class="codex-panel-title">
                工作流摘要
              </h2>
              <p class="codex-panel-subtitle">
                一眼确认是否可以直接开工
              </p>
            </div>
          </div>

          <div
            v-if="error && !overview"
            class="codex-alert codex-alert--danger"
          >
            <p class="mb-2 font-medium">
              仪表盘概览加载失败
            </p>
            <p class="break-words">
              {{ error }}
            </p>
          </div>

          <div
            v-else-if="!overview && overviewLoading"
            class="codex-stack"
          >
            <div class="codex-skeleton" />
            <div class="codex-skeleton" />
            <div class="codex-skeleton" />
          </div>

          <EmptyState
            v-else-if="!overview"
            icon="Inbox"
            title="暂时还没有仪表盘数据"
            description="可以先刷新一次，或者进入 Auth / Profiles 页面补齐基础配置。"
            action-text="立即刷新"
            action-icon="RefreshCw"
            :on-action="() => refresh(true)"
          />

          <div
            v-else
            class="codex-stack"
          >
            <div class="codex-summary-card">
              <p class="codex-summary-label">
                活跃模型
              </p>
              <p class="codex-summary-value">
                {{ usageSummary?.top_model?.model || overview.config.model || (usageLoading ? '分析中' : '未识别') }}
              </p>
              <p class="codex-summary-description">
                {{
                  usageSummary?.top_model
                    ? `近阶段请求 ${usageSummary.top_model.total_requests} 次，输出 ${formatTokens(usageSummary.top_model.total_output_tokens)} tokens`
                    : usageLoading
                      ? '正在计算模型活跃度'
                      : '暂无按模型维度的活跃数据'
                }}
              </p>
            </div>

            <div class="codex-summary-card">
              <p class="codex-summary-label">
                扩展能力库存
              </p>
              <div class="codex-inventory-grid">
                <div>
                  <p class="codex-inventory-key">
                    MCP
                  </p>
                  <p class="codex-inventory-value">
                    {{ overview.inventory.mcp_servers_total }}
                  </p>
                </div>
                <div>
                  <p class="codex-inventory-key">
                    Config Profiles
                  </p>
                  <p class="codex-inventory-value">
                    {{ overview.inventory.config_profiles_total }}
                  </p>
                </div>
                <div>
                  <p class="codex-inventory-key">
                    Agents
                  </p>
                  <p class="codex-inventory-value">
                    {{ overview.inventory.agents_total }}
                  </p>
                </div>
                <div>
                  <p class="codex-inventory-key">
                    Sessions
                  </p>
                  <p class="codex-inventory-value">
                    {{ overview.inventory.sessions_total }}
                  </p>
                </div>
              </div>
            </div>

            <div class="codex-summary-card">
              <p class="codex-summary-label">
                最近活动
              </p>
              <p class="codex-summary-description codex-summary-description--compact">
                {{
                  usageSummary?.last_activity_at
                    ? `最近活动时间 ${formatDateTime(usageSummary.last_activity_at)}`
                    : usageLoading
                      ? '正在读取最近活动记录'
                      : '尚未发现最近活动记录'
                }}
              </p>
            </div>

            <div
              v-if="usageError && !usageSummary"
              class="codex-alert codex-alert--warning"
            >
              <p class="mb-1 font-medium">
                用量摘要加载失败
              </p>
              <p class="break-words">
                {{ usageError }}
              </p>
            </div>
          </div>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onActivated, onMounted } from 'vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useCodexDashboard } from '@/composables/useCodexDashboard'

defineOptions({ name: 'CodexView' })

const {
  overview,
  usageSummary,
  loading,
  error,
  overviewLoading,
  usageLoading,
  usageError,
  versionLabel,
  currentAccountLabel,
  currentProfileLabel,
  usageTotalRequests,
  usageTotalTokens,
  healthItems,
  nextActions,
  managementLinks,
  formatTokens,
  formatDateTime,
  refresh,
} = useCodexDashboard()

const toneClassMap = {
  success: 'codex-tone-icon--success',
  warning: 'codex-tone-icon--warning',
  danger: 'codex-tone-icon--danger',
  neutral: 'codex-tone-icon--neutral',
} as const

onMounted(() => {
  void refresh(false)
})

onActivated(() => {
  void refresh(false)
})
</script>

<style scoped>
.codex-view {
  @apply relative min-h-full overflow-hidden p-6 lg:p-10;
}

.codex-shell {
  @apply mx-auto max-w-7xl space-y-5;
}

.codex-grid {
  @apply grid gap-4;
}

.codex-grid--hero {
  @apply grid-cols-1 xl:grid-cols-3;
}

.codex-grid--health {
  @apply grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4;
}

.codex-grid--manage {
  @apply grid-cols-1 xl:grid-cols-3;
}

.codex-grid--links {
  @apply grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3;
}

.codex-hero-card {
  @apply relative overflow-hidden p-6 xl:col-span-2;
}

.codex-hero-card__glow {
  @apply pointer-events-none absolute inset-y-0 right-0 w-72;

  background: linear-gradient(270deg, rgb(var(--color-accent-primary-rgb) / 10%), rgb(var(--color-premium-blue-rgb) / 38%), transparent);
}

.codex-hero-content {
  @apply relative z-10 space-y-5;
}

.codex-hero-header {
  @apply flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between;
}

.codex-hero-copy {
  @apply space-y-3;
}

.codex-hero-title-row {
  @apply flex items-center gap-3;
}

.codex-hero-icon {
  @apply flex h-12 w-12 items-center justify-center rounded-2xl shadow-lg backdrop-blur-md;

  border: 1px solid rgb(var(--color-accent-primary-rgb) / 14%);
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 100%), rgb(var(--color-bg-surface-rgb) / 90%));
}

.codex-hero-title {
  color: var(--stage-text-primary);

  @apply text-[2.35rem] font-semibold tracking-[-0.04em];

  font-family: var(--font-brand);
}

.codex-hero-subtitle {
  color: var(--stage-text-secondary);

  @apply text-sm;
}

.codex-pill-row {
  @apply flex flex-wrap gap-2;
}

.codex-pill {
  color: var(--stage-chip-neutral-text);

  @apply rounded-full border px-3 py-1 text-xs font-semibold;

  letter-spacing: 0.04em;
}

.codex-pill--primary {
  border-color: rgb(var(--color-accent-primary-rgb) / 16%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
}

.codex-pill--neutral {
  background: var(--stage-chip-neutral-bg);
  border-color: var(--stage-chip-neutral-border);
  color: var(--stage-chip-neutral-text);
}

.codex-pill--info {
  border-color: rgb(var(--color-info-rgb) / 16%);
  background: rgb(var(--color-info-rgb) / 10%);
  color: var(--color-info);
}

.codex-action-row {
  @apply flex flex-wrap gap-2 lg:justify-end;
}

.codex-hero-stats {
  @apply grid grid-cols-1 gap-3 sm:grid-cols-3;
}

.codex-stat-card {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply rounded-2xl px-4 py-3;
}

.codex-stat-label {
  color: var(--stage-text-quiet);

  @apply mb-1 text-xs;

  letter-spacing: 0.04em;
}

.codex-stat-value {
  color: var(--stage-text-primary);

  @apply text-lg font-semibold;
}

.codex-panel {
  @apply p-5;
}

.codex-panel--wide {
  @apply xl:col-span-2;
}

.codex-panel-header {
  @apply mb-4 flex items-center gap-3;
}

.codex-panel-title {
  color: var(--stage-text-primary);

  @apply text-base font-semibold;
}

.codex-panel-subtitle {
  color: var(--stage-text-muted);

  @apply text-xs;
}

.codex-panel-description {
  color: var(--stage-text-muted);

  @apply text-sm;
}

.codex-panel-icon {
  @apply flex h-10 w-10 items-center justify-center rounded-xl border;
}

.codex-panel-icon--amber {
  border-color: rgb(var(--color-warning-rgb) / 16%);
  background: rgb(var(--color-warning-rgb) / 10%);
}

.codex-panel-icon--indigo {
  border-color: rgb(var(--color-accent-primary-rgb) / 16%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

.codex-stack {
  @apply space-y-3;
}

.codex-action-card {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply block rounded-2xl p-4 transition-all duration-200;
}

.codex-action-card:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 16%);
}

.codex-tone-icon {
  @apply mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border;
}

.codex-tone-icon--success {
  border-color: rgb(var(--color-success-rgb) / 16%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--color-success);
}

.codex-tone-icon--warning {
  border-color: rgb(var(--color-warning-rgb) / 16%);
  background: rgb(var(--color-warning-rgb) / 10%);
  color: var(--color-warning);
}

.codex-tone-icon--danger {
  border-color: rgb(var(--color-danger-rgb) / 16%);
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--color-danger);
}

.codex-action-title {
  color: var(--stage-text-primary);

  @apply text-sm font-semibold;
}

.codex-action-arrow {
  color: var(--stage-text-quiet);
}

.codex-action-description {
  color: var(--stage-text-secondary);

  @apply mt-1 text-sm leading-6;
}

.codex-tone-icon--neutral {
  background: var(--stage-chip-neutral-bg);
  border-color: var(--stage-chip-neutral-border);
  color: var(--stage-chip-neutral-text);
}

.codex-tone-icon--large {
  @apply h-11 w-11;
}

.codex-health-card {
  background: var(--stage-surface-medium);
  border: 1px solid var(--stage-border-soft);

  @apply h-full p-4;
}

.codex-health-eyebrow {
  color: var(--stage-text-quiet);
  font-size: 11px;
  letter-spacing: 0.18em;
}

.codex-health-label {
  color: var(--stage-text-quiet);

  @apply mt-4 text-xs;

  letter-spacing: 0.04em;
}

.codex-health-value {
  color: var(--stage-text-primary);

  @apply mt-1 break-words text-lg font-semibold;
}

.codex-health-detail {
  color: var(--stage-text-secondary);

  @apply mt-2 text-sm leading-6;
}

.codex-section-header {
  @apply mb-4 flex items-center justify-between gap-3;
}

.codex-text-link {
  @apply text-sm text-accent-primary transition-colors hover:text-accent-secondary;
}

.codex-link-card {
  background: var(--stage-surface-medium);
  border: 1px solid var(--stage-border-soft);

  @apply h-full p-4;
}

.codex-link-card__header {
  @apply mb-3 flex items-start justify-between gap-3;
}

.codex-link-badge {
  background: var(--stage-chip-neutral-bg);
  border: 1px solid var(--stage-chip-neutral-border);
  color: var(--stage-text-muted);

  @apply rounded-full px-2.5 py-1;

  font-size: 11px;
}

.codex-link-title {
  color: var(--stage-text-primary);

  @apply text-sm font-semibold transition-colors group-hover:text-accent-primary;
}

.codex-link-description {
  color: var(--stage-text-secondary);

  @apply mt-2 text-sm leading-6;
}

.codex-skeleton {
  background: var(--stage-surface-soft);

  @apply h-20 animate-pulse rounded-2xl;
}

.codex-skeleton--panel {
  @apply h-24;
}

.codex-skeleton--icon {
  @apply h-10 w-10 rounded-xl;
}

.codex-skeleton--line {
  @apply h-5 w-3/4;
}

.codex-skeleton--detail {
  @apply h-4 w-full;
}

.codex-summary-card {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply rounded-2xl p-4;
}

.codex-summary-label {
  color: var(--stage-text-quiet);

  @apply mb-1 text-xs;

  letter-spacing: 0.04em;
}

.codex-summary-value {
  color: var(--stage-text-primary);

  @apply text-lg font-semibold;
}

.codex-summary-description {
  color: var(--stage-text-secondary);

  @apply mt-1 text-sm leading-6;
}

.codex-summary-description--compact {
  color: var(--stage-text-secondary);
}

.codex-inventory-grid {
  @apply mt-3 grid grid-cols-2 gap-3 text-sm;
}

.codex-inventory-key {
  color: var(--stage-text-quiet);
}

.codex-inventory-value {
  color: var(--stage-text-primary);

  @apply font-semibold;
}

.codex-alert {
  @apply rounded-2xl p-4 text-sm;
}

.codex-alert--danger {
  color: rgb(136 33 57 / 96%);
  background: rgb(255 231 239 / 76%);
  border: 1px solid rgb(212 111 136 / 28%);
}

.codex-alert--warning {
  color: rgb(126 78 22 / 96%);
  background: rgb(255 243 218 / 78%);
  border: 1px solid rgb(214 161 67 / 30%);
}

[data-theme='dark'] .codex-alert--danger,
:root[class~='dark'] .codex-alert--danger {
  color: rgb(255 225 233 / 92%);
  background: rgb(120 27 56 / 22%);
  border-color: rgb(234 143 170 / 28%);
}

[data-theme='dark'] .codex-alert--warning,
:root[class~='dark'] .codex-alert--warning {
  color: rgb(255 242 213 / 90%);
  background: rgb(131 90 19 / 22%);
  border-color: rgb(225 180 91 / 28%);
}
</style>
