<template>
  <div class="codex-view">
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
                      class="text-pink-400"
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
                  <span class="codex-pill codex-pill--pink"> workflow first </span>
                  <span class="codex-pill codex-pill--neutral">
                    {{ versionLabel }}
                  </span>
                  <span class="codex-pill codex-pill--emerald">
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
                  {{ summary?.usage.all_time.total_requests ?? 0 }}
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

          <div class="codex-stack">
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
                    <h3 class="text-sm font-semibold text-white">
                      {{ action.title }}
                    </h3>
                    <SIcon
                      name="ArrowRight"
                      size="w-4 h-4"
                      class="text-white/35 shrink-0"
                    />
                  </div>
                  <p class="mt-1 text-sm leading-6 text-white/65">
                    {{ action.description }}
                  </p>
                </div>
              </div>
            </RouterLink>
          </div>
        </Card>
      </section>

      <section class="codex-grid codex-grid--health">
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

          <div class="codex-grid codex-grid--links">
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
            v-if="error"
            class="rounded-2xl border border-rose-500/20 bg-rose-500/10 p-4 text-sm text-rose-200"
          >
            <p class="font-medium mb-2">
              仪表盘数据加载失败
            </p>
            <p class="text-rose-100/80 break-words">
              {{ error }}
            </p>
          </div>

          <div
            v-else-if="!summary && loading"
            class="codex-stack"
          >
            <div class="codex-skeleton" />
            <div class="codex-skeleton" />
            <div class="codex-skeleton" />
          </div>

          <EmptyState
            v-else-if="!summary"
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
                {{ summary.usage.top_model?.model || summary.config.model || '未识别' }}
              </p>
              <p class="codex-summary-description">
                {{
                  summary.usage.top_model
                    ? `近阶段请求 ${summary.usage.top_model.total_requests} 次，输出 ${formatTokens(summary.usage.top_model.total_output_tokens)} tokens`
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
                    {{ summary.inventory.mcp_servers_total }}
                  </p>
                </div>
                <div>
                  <p class="codex-inventory-key">
                    Config Profiles
                  </p>
                  <p class="codex-inventory-value">
                    {{ summary.inventory.config_profiles_total }}
                  </p>
                </div>
                <div>
                  <p class="codex-inventory-key">
                    Agents
                  </p>
                  <p class="codex-inventory-value">
                    {{ summary.inventory.agents_total }}
                  </p>
                </div>
                <div>
                  <p class="codex-inventory-key">
                    Sessions
                  </p>
                  <p class="codex-inventory-value">
                    {{ summary.inventory.sessions_total }}
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
                  summary.usage.last_activity_at
                    ? `最近活动时间 ${formatDateTime(summary.usage.last_activity_at)}`
                    : '尚未发现最近活动记录'
                }}
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
  summary,
  loading,
  error,
  versionLabel,
  currentAccountLabel,
  currentProfileLabel,
  usageTotalTokens,
  healthItems,
  nextActions,
  managementLinks,
  formatTokens,
  formatDateTime,
  refresh,
} = useCodexDashboard()

const toneClassMap = {
  success: 'bg-emerald-500/10 border-emerald-500/20 text-emerald-300',
  warning: 'bg-amber-500/10 border-amber-500/20 text-amber-300',
  danger: 'bg-rose-500/10 border-rose-500/20 text-rose-300',
  neutral: 'bg-white/5 border-white/10 text-white/75',
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

  background: linear-gradient(270deg, rgb(236 72 153 / 10%), rgb(168 85 247 / 5%), transparent);
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
  @apply flex h-12 w-12 items-center justify-center rounded-2xl border border-pink-500/20 bg-pink-500/10 shadow-lg backdrop-blur-md;
}

.codex-hero-title {
  @apply text-3xl font-bold tracking-tight text-white;

  font-family: MapleBright, 'Microsoft YaHei UI', system-ui, sans-serif;
}

.codex-hero-subtitle {
  @apply text-sm text-white/70;
}

.codex-pill-row {
  @apply flex flex-wrap gap-2;
}

.codex-pill {
  @apply rounded-full border px-3 py-1 text-xs font-semibold uppercase text-white/75;

  letter-spacing: 0.12em;
}

.codex-pill--pink {
  @apply border-pink-500/20 bg-pink-500/10 text-pink-300;
}

.codex-pill--neutral {
  @apply border-white/10 bg-white/5 text-white/75;
}

.codex-pill--emerald {
  @apply border-emerald-500/20 bg-emerald-500/10 text-emerald-300;
}

.codex-action-row {
  @apply flex flex-wrap gap-2 lg:justify-end;
}

.codex-hero-stats {
  @apply grid grid-cols-1 gap-3 sm:grid-cols-3;
}

.codex-stat-card {
  @apply rounded-2xl border border-white/10 bg-white/5 px-4 py-3;
}

.codex-stat-label {
  @apply mb-1 text-xs uppercase text-white/45;

  letter-spacing: 0.2em;
}

.codex-stat-value {
  @apply text-lg font-semibold text-white;
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
  @apply text-base font-semibold text-white;
}

.codex-panel-subtitle {
  @apply text-xs text-white/55;
}

.codex-panel-description {
  @apply text-sm text-white/55;
}

.codex-panel-icon {
  @apply flex h-10 w-10 items-center justify-center rounded-xl border;
}

.codex-panel-icon--amber {
  @apply border-amber-500/20 bg-amber-500/10;
}

.codex-panel-icon--indigo {
  @apply border-indigo-500/20 bg-indigo-500/10;
}

.codex-stack {
  @apply space-y-3;
}

.codex-action-card {
  @apply block rounded-2xl border border-white/10 bg-white/5 p-4 transition-all duration-200 hover:border-pink-500/30 hover:bg-white/10;
}

.codex-tone-icon {
  @apply mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-xl border;
}

.codex-tone-icon--large {
  @apply h-11 w-11;
}

.codex-health-card {
  @apply h-full border border-white/10 p-4;
}

.codex-health-eyebrow {
  @apply text-white/35;

  font-size: 11px;
  letter-spacing: 0.18em;
}

.codex-health-label {
  @apply mt-4 text-xs uppercase text-white/45;

  letter-spacing: 0.2em;
}

.codex-health-value {
  @apply mt-1 break-words text-lg font-semibold text-white;
}

.codex-health-detail {
  @apply mt-2 text-sm leading-6 text-white/60;
}

.codex-section-header {
  @apply mb-4 flex items-center justify-between gap-3;
}

.codex-text-link {
  @apply text-sm text-pink-300 transition-colors hover:text-pink-200;
}

.codex-link-card {
  @apply h-full border border-white/10 p-4;
}

.codex-link-card__header {
  @apply mb-3 flex items-start justify-between gap-3;
}

.codex-link-badge {
  @apply rounded-full border border-white/10 bg-white/5 px-2.5 py-1 text-white/55;

  font-size: 11px;
}

.codex-link-title {
  @apply text-sm font-semibold text-white transition-colors group-hover:text-pink-200;
}

.codex-link-description {
  @apply mt-2 text-sm leading-6 text-white/60;
}

.codex-skeleton {
  @apply h-20 animate-pulse rounded-2xl bg-white/5;
}

.codex-summary-card {
  @apply rounded-2xl border border-white/10 bg-white/5 p-4;
}

.codex-summary-label {
  @apply mb-1 text-xs uppercase text-white/45;

  letter-spacing: 0.2em;
}

.codex-summary-value {
  @apply text-lg font-semibold text-white;
}

.codex-summary-description {
  @apply mt-1 text-sm leading-6 text-white/60;
}

.codex-summary-description--compact {
  @apply text-white/70;
}

.codex-inventory-grid {
  @apply mt-3 grid grid-cols-2 gap-3 text-sm;
}

.codex-inventory-key {
  @apply text-white/40;
}

.codex-inventory-value {
  @apply font-semibold text-white;
}
</style>
