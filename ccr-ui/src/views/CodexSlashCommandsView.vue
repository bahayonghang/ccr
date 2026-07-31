<template>
  <div class="codex-slash-view">
    <div class="codex-slash-shell">
      <ModuleSubnav module="codex" />

      <div class="codex-slash-stack">
        <Card
          variant="glass"
          class="codex-slash-hero"
        >
          <div class="codex-slash-hero__glow" />
          <div class="codex-slash-hero__content">
            <div class="codex-slash-hero__icon">
              <SIcon
                name="Command"
                size="w-6 h-6"
                class="text-accent-warning"
              />
            </div>

            <div class="codex-slash-hero__copy">
              <p class="codex-slash-kicker">
                {{ tt('仅兼容入口', 'Compatibility Only') }}
              </p>
              <h1 class="codex-slash-title">
                {{ tt('Codex 目前没有可管理的 Slash Commands', 'Codex currently has no manageable slash commands') }}
              </h1>
              <p class="codex-slash-subtitle">
                {{ tt('这个页面保留为兼容入口，用来解释为什么 Codex 模块没有接入 Slash Commands 管理。 当前工作流重点已经切到 Sessions、Agents、Profiles 和 MCP。', 'This page remains as a compatibility entry so it can explain why the Codex module does not expose slash-command management. The active workflow focus has shifted to Sessions, Agents, Profiles, and MCP.') }}
              </p>
            </div>

            <div class="codex-slash-actions">
              <RouterLink
                to="/codex/sessions"
                class="btn btn-primary"
              >
                <SIcon
                  name="MessagesSquare"
                  size="w-4 h-4"
                />
                <span>{{ tt('打开 Sessions', 'Open Sessions') }}</span>
              </RouterLink>
              <RouterLink
                to="/codex/agents"
                class="btn btn-secondary"
              >
                <SIcon
                  name="Bot"
                  size="w-4 h-4"
                />
                <span>{{ tt('管理 Agents', 'Manage Agents') }}</span>
              </RouterLink>
            </div>
          </div>
        </Card>

        <div class="codex-slash-grid">
          <Card
            variant="glass"
            class="codex-slash-panel"
          >
            <div class="codex-slash-panel__header">
              <div class="codex-slash-panel__icon codex-slash-panel__icon--rose">
                <SIcon
                  name="AlertTriangle"
                  size="w-5 h-5"
                />
              </div>
              <div>
                <h2 class="codex-slash-panel__title">
                  {{ tt('当前状态', 'Current state') }}
                </h2>
                <p class="codex-slash-panel__subtitle">
                  {{ tt('后端没有对应的 Slash Commands 命令集', 'The backend has no matching slash-command command set') }}
                </p>
              </div>
            </div>

            <div class="codex-slash-note">
              <p class="codex-slash-note__title">
                {{ tt('为什么不继续沿用通用页面', 'Why not keep the generic page') }}
              </p>
              <p class="codex-slash-note__body">
                {{ tt('Codex 在 CCR 中没有对应的 slash command CRUD 能力，之前的页面只是复用通用容器后返回“平台不支持”。现在把它降级成说明页，避免把一个不存在的能力放进主导航。', 'CCR does not expose slash-command CRUD for Codex. The old page only reused the generic container and returned “platform not supported”. It is now downgraded into an explainer page so a nonexistent capability does not stay in the main navigation.') }}
              </p>
            </div>
          </Card>

          <Card
            variant="glass"
            class="codex-slash-panel"
          >
            <div class="codex-slash-panel__header">
              <div class="codex-slash-panel__icon codex-slash-panel__icon--emerald">
                <SIcon
                  name="Workflow"
                  size="w-5 h-5"
                />
              </div>
              <div>
                <h2 class="codex-slash-panel__title">
                  {{ tt('推荐入口', 'Recommended entries') }}
                </h2>
                <p class="codex-slash-panel__subtitle">
                  {{ tt('真实可用的 Codex 工作面板', 'The Codex surfaces that actually work') }}
                </p>
              </div>
            </div>

            <div class="codex-slash-shortcuts">
              <RouterLink
                v-for="item in shortcuts"
                :key="item.to"
                :to="item.to"
                class="codex-slash-shortcut"
              >
                <div class="codex-slash-shortcut__icon">
                  <SIcon
                    :name="item.icon"
                    size="w-4 h-4"
                  />
                </div>
                <div class="min-w-0 flex-1">
                  <p class="codex-slash-shortcut__title">
                    {{ item.title }}
                  </p>
                  <p class="codex-slash-shortcut__desc">
                    {{ item.description }}
                  </p>
                </div>
                <SIcon
                  name="ArrowRight"
                  size="w-4 h-4"
                  class="text-text-ghost"
                />
              </RouterLink>
            </div>
          </Card>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import Card from '@/components/ui/Card.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import SIcon from '@/components/ui/SIcon.vue'

defineOptions({ name: 'CodexSlashCommandsView' })
const { locale } = useI18n()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)

const shortcuts = [
  {
    title: tt('Sessions', 'Sessions'),
    description: tt('查看最近会话、导出上下文、克隆或删除本地 session 记录。', 'Inspect recent sessions, export context, and clone or delete local session records.'),
    to: '/codex/sessions',
    icon: 'MessagesSquare',
  },
  {
    title: tt('Agents', 'Agents'),
    description: tt('管理 Codex 专用 agents，复用现有 agent 配置能力。', 'Manage Codex-specific agents while reusing the existing agent configuration flow.'),
    to: '/codex/agents',
    icon: 'Bot',
  },
  {
    title: 'MCP',
    description: tt('继续扩展本地工具链，把 Codex 接到更多外部能力上。', 'Keep extending the local toolchain and connect Codex to more external capabilities.'),
    to: '/codex/mcp',
    icon: 'Server',
  },
] as const
</script>

<style scoped>
.codex-slash-view {
  @apply min-h-full p-6 lg:p-8;
}

.codex-slash-shell {
  @apply mx-auto max-w-6xl space-y-6;
}

.codex-slash-stack {
  @apply space-y-4;
}

.codex-slash-hero {
  @apply relative overflow-hidden p-6;
}

.codex-slash-hero__glow {
  @apply pointer-events-none absolute inset-y-0 right-0 w-72;

  background: linear-gradient(270deg, rgb(245 158 11 / 16%), rgb(244 114 182 / 10%), transparent);
}

.codex-slash-hero__content {
  @apply relative z-10 flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between;
}

.codex-slash-hero__icon {
  @apply flex h-14 w-14 items-center justify-center rounded-2xl border border-accent-warning/20 bg-accent-warning/10;
}

.codex-slash-hero__copy {
  @apply min-w-0 flex-1 space-y-2;
}

.codex-slash-kicker {
  @apply text-xs font-semibold tracking-[0.08em] text-accent-warning;
}

.codex-slash-title {
  @apply text-2xl font-bold text-text-primary lg:text-3xl;

  font-family: var(--font-brand);
}

.codex-slash-subtitle {
  @apply max-w-3xl text-sm leading-7 text-text-secondary;
}

.codex-slash-actions {
  @apply flex flex-wrap gap-3;
}

.codex-slash-grid {
  @apply grid gap-4 xl:grid-cols-2;
}

.codex-slash-panel {
  @apply p-5;
}

.codex-slash-panel__header {
  @apply mb-4 flex items-center gap-3;
}

.codex-slash-panel__icon {
  @apply flex h-11 w-11 items-center justify-center rounded-2xl border;
}

.codex-slash-panel__icon--rose {
  @apply border-accent-danger/20 bg-accent-danger/10 text-accent-danger;
}

.codex-slash-panel__icon--emerald {
  @apply border-accent-success/20 bg-accent-success/10 text-accent-success;
}

.codex-slash-panel__title {
  @apply text-base font-semibold text-text-primary;
}

.codex-slash-panel__subtitle {
  @apply text-sm text-text-muted;
}

.codex-slash-note {
  @apply rounded-2xl border border-border-default/15 bg-bg-elevated p-4;
}

.codex-slash-note__title {
  @apply text-sm font-semibold text-text-primary;
}

.codex-slash-note__body {
  @apply mt-2 text-sm leading-7 text-text-secondary;
}

.codex-slash-shortcuts {
  @apply space-y-3;
}

.codex-slash-shortcut {
  @apply flex items-start gap-3 rounded-2xl border border-border-default/15 bg-bg-elevated p-4 transition-all duration-200 hover:border-accent-warning/25 hover:bg-bg-elevated/80;
}

.codex-slash-shortcut__icon {
  @apply mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border border-border-default/15 bg-bg-elevated text-text-secondary;
}

.codex-slash-shortcut__title {
  @apply text-sm font-semibold text-text-primary;
}

.codex-slash-shortcut__desc {
  @apply mt-1 text-sm leading-6 text-text-muted;
}
</style>

