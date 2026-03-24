<template>
  <div class="open-code-view">
    <AnimatedBackground
      contained
      variant="complex"
    />

    <div class="open-code-shell">
      <!-- HEADER -->
      <section class="open-code-header animate-slide-up">
        <!-- Hero Card -->
        <Card
          variant="glass"
          class="open-code-hero-card"
        >
          <div class="open-code-hero-card__glow" />

          <div class="open-code-hero-card__content">
            <div class="open-code-brand">
              <div class="open-code-brand__icon">
                <SIcon
                  name="TerminalSquare"
                  size="w-6 h-6"
                />
              </div>
              <div>
                <h1 class="open-code-brand__title">
                  OpenCode
                </h1>
                <p class="open-code-brand__subtitle">
                  基于 npm AI SDK 的叠加式 Provider 配置管理
                </p>
              </div>
            </div>

            <div class="open-code-tag-row">
              <span class="open-code-tag open-code-tag--violet">
                <SIcon
                  name="Layers"
                  size="w-3 h-3"
                /> npm AI SDK
              </span>
              <span class="open-code-tag open-code-tag--secondary">
                opencode.json
              </span>
            </div>
          </div>
        </Card>

        <!-- Status Grid -->
        <div class="open-code-status-grid">
          <!-- Provider Count -->
          <Card
            variant="elevated"
            class="open-code-status-card open-code-status-card--violet"
          >
            <div class="open-code-status-card__icon open-code-status-card__icon--violet">
              <SIcon
                name="Layers"
                size="w-5 h-5"
              />
            </div>
            <div>
              <p class="open-code-status-card__label">
                Providers
              </p>
              <p class="open-code-status-card__value">
                {{ providersCount }}
              </p>
            </div>
          </Card>

          <!-- MCP Count -->
          <Card
            variant="elevated"
            class="open-code-status-card open-code-status-card--blue"
          >
            <div class="open-code-status-card__icon open-code-status-card__icon--blue">
              <SIcon
                name="Server"
                size="w-5 h-5"
              />
            </div>
            <div>
              <p class="open-code-status-card__label">
                MCP 服务器
              </p>
              <p class="open-code-status-card__value">
                {{ mcpCount }}
              </p>
            </div>
          </Card>

          <!-- Plugin Count -->
          <Card
            variant="elevated"
            class="open-code-status-card open-code-status-card--emerald"
          >
            <div class="open-code-status-card__icon open-code-status-card__icon--emerald">
              <SIcon
                name="Puzzle"
                size="w-5 h-5"
              />
            </div>
            <div>
              <p class="open-code-status-card__label">
                插件
              </p>
              <p class="open-code-status-card__value">
                {{ pluginsCount }}
              </p>
            </div>
          </Card>
        </div>
      </section>

      <!-- MODULE NAVIGATION -->
      <section class="animate-slide-up">
        <h2 class="open-code-section-title">
          功能模块
        </h2>
        <div class="open-code-module-grid">
          <RouterLink
            v-for="mod in modules"
            :key="mod.href"
            :to="mod.href"
            class="open-code-module-link"
          >
            <Card
              variant="glass"
              class="open-code-module-card"
            >
              <div
                :class="[
                  'open-code-module-card__icon',
                  `open-code-module-card__icon--${mod.tone}`,
                ]"
              >
                <SIcon
                  :name="mod.icon"
                  size="w-5 h-5"
                />
              </div>
              <div>
                <h3 class="open-code-module-card__title">
                  {{ mod.title }}
                </h3>
                <p class="open-code-module-card__desc">
                  {{ mod.description }}
                </p>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- CONFIG PATH INFO -->
      <section
        v-if="configPath"
        class="animate-slide-up"
        style="animation-delay: 0.2s"
      >
        <Card
          variant="elevated"
          class="open-code-config-card"
        >
          <SIcon
            name="FileJson"
            size="w-5 h-5"
            class="open-code-config-card__icon"
          />
          <div>
            <p class="open-code-config-card__label">
              配置文件路径
            </p>
            <p class="open-code-config-card__value">
              ~/.config/opencode/opencode.json
            </p>
          </div>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import {
  listOpenCodeProviders,
  listOpenCodeMcpServers,
  listOpenCodePlugins,
} from '@/api'
import type { OpenCodeProvider, OpenCodeMcpServer, OpenCodePlugin } from '@/types'

const providersCount = ref(0)
const mcpCount = ref(0)
const pluginsCount = ref(0)
const configPath = ref(true)

const modules = [
  {
    title: 'Provider 管理',
    description: '管理 npm AI SDK Provider，配置 API Key 和模型列表',
    href: '/opencode/providers',
    icon: 'Layers',
    tone: 'violet',
  },
  {
    title: 'MCP 服务器',
    description: '管理本地（local）和远程（remote）MCP 服务器',
    href: '/opencode/mcp',
    icon: 'Server',
    tone: 'blue',
  },
  {
    title: 'Skills',
    description: '管理 AI 技能库，跨平台共享 Skills 配置',
    href: '/skills',
    icon: 'Puzzle',
    tone: 'amber',
  },
  {
    title: '插件管理',
    description: '管理 npm 插件包，扩展 OpenCode 功能',
    href: '/opencode/plugins',
    icon: 'Puzzle',
    tone: 'emerald',
  },
]

onMounted(async () => {
  try {
    const [providers, mcpServers, plugins] = await Promise.all([
      listOpenCodeProviders<OpenCodeProvider[]>(),
      listOpenCodeMcpServers<OpenCodeMcpServer[]>(),
      listOpenCodePlugins<OpenCodePlugin[]>(),
    ])
    providersCount.value = providers.length
    mcpCount.value = mcpServers.length
    pluginsCount.value = plugins.length
  } catch {
    // 静默失败，OpenCode 可能未安装
  }
})
</script>

<style scoped>
.open-code-view {
  @apply relative min-h-full overflow-hidden p-6 lg:p-10;
}

.open-code-shell {
  @apply relative z-10 mx-auto max-w-7xl space-y-5;
}

.open-code-header {
  @apply grid grid-cols-1 gap-4 lg:grid-cols-3;
}

.open-code-hero-card {
  @apply relative flex flex-col overflow-hidden p-5 lg:col-span-2;
}

.open-code-hero-card__glow {
  @apply pointer-events-none absolute h-48 w-48 rounded-bl-full;

  top: 0;
  right: 0;
  margin-top: -3rem;
  margin-right: -3rem;
  background: linear-gradient(to bottom left, rgb(139 92 246 / 10%), transparent);
}

.open-code-hero-card__content {
  @apply relative z-10;
}

.open-code-brand {
  @apply mb-3 flex items-center gap-3;
}

.open-code-brand__icon {
  @apply flex h-12 w-12 items-center justify-center rounded-xl border shadow-lg backdrop-blur-md;

  color: rgb(139 92 246);
  background: rgb(139 92 246 / 10%);
  border-color: rgb(139 92 246 / 20%);
}

.open-code-brand__title {
  @apply text-3xl font-bold tracking-tight text-white;

  font-family: MapleBright, 'Microsoft YaHei UI', system-ui, sans-serif;
}

.open-code-brand__subtitle {
  @apply max-w-md text-base text-white/80;
}

.open-code-tag-row {
  @apply flex flex-wrap gap-2;
}

.open-code-tag {
  @apply flex items-center gap-2 rounded-full border px-3 py-1 text-xs font-bold uppercase;

  letter-spacing: 0.1em;
}

.open-code-tag--violet {
  color: rgb(139 92 246);
  background: rgb(139 92 246 / 10%);
  border-color: rgb(139 92 246 / 20%);
}

.open-code-tag--secondary {
  @apply border-accent-secondary/20 bg-accent-secondary/10 text-accent-secondary;
}

.open-code-status-grid {
  @apply grid grid-cols-1 gap-3;
}

.open-code-status-card {
  @apply flex items-center gap-3 border-l-4 p-3;
}

.open-code-status-card--violet {
  border-left-color: rgb(139 92 246);
}

.open-code-status-card--blue {
  border-left-color: rgb(59 130 246);
}

.open-code-status-card--emerald {
  border-left-color: rgb(16 185 129);
}

.open-code-status-card__icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-lg;
}

.open-code-status-card__icon--violet {
  color: rgb(139 92 246);
  background: rgb(139 92 246 / 10%);
}

.open-code-status-card__icon--blue {
  color: rgb(59 130 246);
  background: rgb(59 130 246 / 10%);
}

.open-code-status-card__icon--emerald {
  color: rgb(16 185 129);
  background: rgb(16 185 129 / 10%);
}

.open-code-status-card__label {
  @apply mb-0.5 text-xs font-bold uppercase text-white/50;

  letter-spacing: 0.1em;
}

.open-code-status-card__value {
  @apply text-base font-bold text-white;
}

.open-code-section-title {
  @apply mb-3 text-lg font-bold text-white;
}

.open-code-module-grid {
  @apply grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4;
}

.open-code-module-link {
  @apply block;
}

.open-code-module-card {
  @apply flex cursor-pointer flex-col gap-3 border border-transparent p-4 duration-300;

  transition-property: box-shadow, border-color, transform;
}

.open-code-module-link:hover .open-code-module-card {
  @apply shadow-lg;

  transform: scale(1.02);
  border-color: rgb(139 92 246 / 30%);
}

.open-code-module-card__icon {
  @apply flex h-10 w-10 items-center justify-center rounded-lg;
}

.open-code-module-card__icon--violet {
  color: rgb(139 92 246);
  background: rgb(139 92 246 / 10%);
}

.open-code-module-card__icon--blue {
  color: rgb(59 130 246);
  background: rgb(59 130 246 / 10%);
}

.open-code-module-card__icon--amber {
  color: rgb(245 158 11);
  background: rgb(245 158 11 / 10%);
}

.open-code-module-card__icon--emerald {
  color: rgb(16 185 129);
  background: rgb(16 185 129 / 10%);
}

.open-code-module-card__title {
  @apply mb-1 text-sm font-bold text-white;
}

.open-code-module-card__desc {
  @apply text-xs leading-relaxed text-white/50;
}

.open-code-config-card {
  @apply flex items-center gap-3 p-4;
}

.open-code-config-card__icon {
  @apply shrink-0 text-white/50;
}

.open-code-config-card__label {
  @apply mb-0.5 text-xs font-bold uppercase text-white/50;

  letter-spacing: 0.1em;
}

.open-code-config-card__value {
  @apply text-sm font-mono text-white/80;
}
</style>
