<template>
  <div class="opencode-view stage-page">
    <div class="opencode-shell">
      <section class="opencode-grid opencode-grid--hero">
        <Card
          variant="glass"
          class="opencode-hero-card"
        >
          <div class="opencode-hero-card__glow" />

          <div class="opencode-hero-card__content">
            <div class="opencode-hero-head">
              <div class="opencode-hero-copy">
                <div class="opencode-hero-title-row">
                  <div class="opencode-hero-icon">
                    <SIcon
                      name="TerminalSquare"
                      size="w-6 h-6"
                      class="text-lime-300"
                    />
                  </div>
                  <div>
                    <div class="opencode-hero-eyebrow">
                      OpenCode operator deck
                    </div>
                    <h1 class="opencode-hero-title">
                      OpenCode
                    </h1>
                    <p class="opencode-hero-subtitle">
                      把 provider、MCP、agents、commands、skills、plugins 与 runtime 配置收敛到一个高密度控制台。
                    </p>
                  </div>
                </div>

                <div class="opencode-pill-row">
                  <span class="opencode-pill opencode-pill--lime">
                    config: {{ configPathLabel }}
                  </span>
                  <span class="opencode-pill opencode-pill--neutral">
                    tui: {{ tuiPathLabel }}
                  </span>
                  <span class="opencode-pill opencode-pill--violet">
                    default agent: {{ defaultAgentLabel }}
                  </span>
                </div>
              </div>

              <div class="opencode-action-row">
                <RouterLink to="/opencode/providers">
                  <Button
                    variant="glass"
                    size="sm"
                  >
                    <SIcon
                      name="Layers"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    Providers
                  </Button>
                </RouterLink>
                <RouterLink to="/opencode/mcp">
                  <Button
                    variant="glass"
                    size="sm"
                  >
                    <SIcon
                      name="Server"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    MCP
                  </Button>
                </RouterLink>
                <RouterLink to="/opencode/settings">
                  <Button
                    variant="glass"
                    size="sm"
                  >
                    <SIcon
                      name="SlidersHorizontal"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    Settings
                  </Button>
                </RouterLink>
                <Button
                  variant="ghost"
                  size="sm"
                  :disabled="loading"
                  @click="loadOverview()"
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

            <div class="opencode-hero-stats">
              <div class="opencode-stat-card">
                <p class="opencode-stat-label">
                  Providers
                </p>
                <p class="opencode-stat-value">
                  {{ providers.length }}
                </p>
                <p class="opencode-stat-detail">
                  active model: {{ activeModelLabel }}
                </p>
              </div>
              <div class="opencode-stat-card">
                <p class="opencode-stat-label">
                  MCP / Agents / Commands
                </p>
                <p class="opencode-stat-value">
                  {{ mcpServers.length }} / {{ agents.length }} / {{ commands.length }}
                </p>
                <p class="opencode-stat-detail">
                  npm plugins: {{ plugins.length }} · local files: {{ localPlugins.length }}
                </p>
              </div>
              <div class="opencode-stat-card">
                <p class="opencode-stat-label">
                  Runtime
                </p>
                <p class="opencode-stat-value">
                  {{ runtimeStatusLabel }}
                </p>
                <p class="opencode-stat-detail">
                  theme: {{ themeLabel }}
                </p>
              </div>
            </div>
          </div>
        </Card>

        <Card
          variant="glass"
          class="opencode-side-panel"
        >
          <div class="opencode-side-panel__header">
            <div class="opencode-side-panel__icon">
              <SIcon
                name="Route"
                size="w-5 h-5"
                class="text-cyan-300"
              />
            </div>
            <div>
              <h2 class="opencode-side-panel__title">
                Runtime strip
              </h2>
              <p class="opencode-side-panel__description">
                先看 server / web / ACP，再决定下一个入口。
              </p>
            </div>
          </div>

          <div class="opencode-runtime-strip">
            <div class="opencode-runtime-chip">
              <span class="opencode-runtime-chip__label">serve</span>
              <strong>{{ serverLabel }}</strong>
            </div>
            <div class="opencode-runtime-chip">
              <span class="opencode-runtime-chip__label">web</span>
              <strong>{{ webLabel }}</strong>
            </div>
            <div class="opencode-runtime-chip">
              <span class="opencode-runtime-chip__label">acp</span>
              <strong>available</strong>
            </div>
            <div class="opencode-runtime-chip">
              <span class="opencode-runtime-chip__label">share</span>
              <strong>{{ shareLabel }}</strong>
            </div>
          </div>

          <div class="opencode-side-panel__footer">
            <div>
              <p class="opencode-side-panel__footer-label">
                skills discovery
              </p>
              <p class="opencode-side-panel__footer-value">
                {{ skillLocationSummary }}
              </p>
            </div>
            <RouterLink
              to="/opencode/skills"
              class="opencode-text-link"
            >
              打开 Skills
            </RouterLink>
          </div>
        </Card>
      </section>

      <section class="opencode-grid opencode-grid--capabilities">
        <RouterLink
          v-for="card in opencodeCapabilityCards"
          :key="card.id"
          :to="card.href"
          class="group"
        >
          <Card
            variant="elevated"
            hover
            class="opencode-capability-card"
          >
            <div class="opencode-capability-card__header">
              <div
                class="opencode-capability-card__icon"
                :class="`opencode-capability-card__icon--${card.tone}`"
              >
                <SIcon
                  :name="card.icon"
                  size="w-5 h-5"
                />
              </div>
              <SIcon
                name="ArrowRight"
                size="w-4 h-4"
                class="opencode-capability-card__arrow"
              />
            </div>
            <h3 class="opencode-capability-card__title">
              {{ card.title }}
            </h3>
            <p class="opencode-capability-card__description">
              {{ card.description }}
            </p>
          </Card>
        </RouterLink>
      </section>

      <section class="opencode-grid opencode-grid--detail">
        <Card
          variant="glass"
          class="opencode-panel"
        >
          <div class="opencode-panel__header">
            <h2 class="opencode-panel__title">
              CLI runtime surface
            </h2>
            <p class="opencode-panel__description">
              这是 OpenCode 在 CLI 层暴露的核心运行面，适合作为页面信息架构骨架。
            </p>
          </div>
          <div class="opencode-command-list">
            <div
              v-for="item in opencodeCliCommands"
              :key="item.command"
              class="opencode-command-row"
            >
              <div>
                <p class="opencode-command-row__command">
                  {{ item.command }}
                </p>
                <p class="opencode-command-row__description">
                  {{ item.description }}
                </p>
              </div>
              <span
                v-if="item.note"
                class="opencode-command-row__note"
              >
                {{ item.note }}
              </span>
            </div>
          </div>
        </Card>

        <Card
          variant="glass"
          class="opencode-panel"
        >
          <div class="opencode-panel__header">
            <h2 class="opencode-panel__title">
              Built-in tools
            </h2>
            <p class="opencode-panel__description">
              页面展示使用 built-in tool 与 permission key 的对应关系，帮助理解 `permission` 配置。
            </p>
          </div>
          <div class="opencode-tool-grid">
            <div
              v-for="tool in opencodeBuiltInTools"
              :key="tool.id"
              class="opencode-tool-card"
            >
              <div class="opencode-tool-card__meta">
                <strong>{{ tool.id }}</strong>
                <span>{{ tool.permissionKey }}</span>
              </div>
              <p class="opencode-tool-card__description">
                {{ tool.description }}
              </p>
              <p
                v-if="tool.availability"
                class="opencode-tool-card__availability"
              >
                {{ tool.availability }}
              </p>
            </div>
          </div>
        </Card>
      </section>

      <section class="opencode-grid opencode-grid--detail">
        <Card
          variant="glass"
          class="opencode-panel"
        >
          <div class="opencode-panel__header">
            <h2 class="opencode-panel__title">
              Config topology
            </h2>
            <p class="opencode-panel__description">
              按官方 precedence 和目录约定整理出 UI 需要可视化的路径图。
            </p>
          </div>
          <div class="opencode-topology-list">
            <div
              v-for="item in opencodeConfigTopology"
              :key="item.path"
              class="opencode-topology-item"
            >
              <span class="opencode-topology-item__title">{{ item.title }}</span>
              <code class="opencode-topology-item__path">{{ item.path }}</code>
              <p class="opencode-topology-item__description">
                {{ item.description }}
              </p>
            </div>
          </div>
        </Card>

        <Card
          variant="glass"
          class="opencode-panel"
        >
          <div class="opencode-panel__header">
            <h2 class="opencode-panel__title">
              Local discovery
            </h2>
            <p class="opencode-panel__description">
              这里汇总本地插件文件和 skills 目录发现结果，判断 global / project 面是否已经落地。
            </p>
          </div>
          <div class="opencode-discovery-grid">
            <div class="opencode-discovery-card">
              <span class="opencode-discovery-card__label">Local plugins</span>
              <strong class="opencode-discovery-card__value">{{ localPlugins.length }}</strong>
              <p class="opencode-discovery-card__detail">
                {{ localPlugins.slice(0, 2).map((item) => item.name).join(', ') || 'No plugin files detected' }}
              </p>
            </div>
            <div class="opencode-discovery-card">
              <span class="opencode-discovery-card__label">Skill locations</span>
              <strong class="opencode-discovery-card__value">{{ skillLocations.length }}</strong>
              <p class="opencode-discovery-card__detail">
                {{ skillLocationSummary }}
              </p>
            </div>
            <div class="opencode-discovery-card">
              <span class="opencode-discovery-card__label">Agents</span>
              <strong class="opencode-discovery-card__value">{{ agents.length }}</strong>
              <p class="opencode-discovery-card__detail">
                custom + built-in split should happen in the dedicated page
              </p>
            </div>
          </div>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import {
  getOpenCodeConfig,
  getOpenCodeTuiSettings,
  listOpenCodeAgents,
  listOpenCodeCommands,
  listOpenCodeLocalPlugins,
  listOpenCodeMcpServers,
  listOpenCodePlugins,
  listOpenCodeProviders,
  listOpenCodeSkillLocations,
} from '@/api'
import {
  opencodeBuiltInTools,
  opencodeCapabilityCards,
  opencodeCliCommands,
  opencodeConfigTopology,
} from '@/config/opencodeMeta'
import type {
  OpenCodeAgent,
  OpenCodeCommand,
  OpenCodeConfig,
  OpenCodeLocalPluginFile,
  OpenCodeMcpServer,
  OpenCodePluginPackage,
  OpenCodeProviderConfig,
  OpenCodeSkillLocation,
  OpenCodeTuiConfig,
} from '@/types'

const loading = ref(false)
const config = ref<OpenCodeConfig>({})
const tui = ref<OpenCodeTuiConfig>({})
const providers = ref<OpenCodeProviderConfig[]>([])
const mcpServers = ref<OpenCodeMcpServer[]>([])
const agents = ref<OpenCodeAgent[]>([])
const commands = ref<OpenCodeCommand[]>([])
const plugins = ref<OpenCodePluginPackage[]>([])
const localPlugins = ref<OpenCodeLocalPluginFile[]>([])
const skillLocations = ref<OpenCodeSkillLocation[]>([])

const configPathLabel = '~/.config/opencode/opencode.json'
const tuiPathLabel = '~/.config/opencode/tui.json'

const activeModelLabel = computed(() => config.value.model || 'not configured')
const defaultAgentLabel = computed(() => config.value.default_agent || 'build')
const themeLabel = computed(() => tui.value.theme || 'system')
const shareLabel = computed(() => config.value.share || 'manual')
const runtimeStatusLabel = computed(() => {
  if (config.value.server?.port) return `:${config.value.server.port}`
  return 'default port'
})
const serverLabel = computed(() => {
  const port = config.value.server?.port ?? 4096
  const host = config.value.server?.hostname || 'localhost'
  return `${host}:${port}`
})
const webLabel = computed(() => config.value.server?.cors?.length ? 'cors configured' : 'same host')
const skillLocationSummary = computed(() => {
  const active = skillLocations.value.filter((item) => item.exists && item.skillCount > 0)
  if (active.length === 0) return 'No active OpenCode-compatible skill directories'
  return active
    .slice(0, 3)
    .map((item) => `${item.kind}:${item.skillCount}`)
    .join(' · ')
})

async function loadOverview() {
  loading.value = true
  try {
    const [
      configValue,
      tuiValue,
      providerList,
      mcpList,
      agentList,
      commandList,
      pluginList,
      localPluginList,
      skillLocationList,
    ] = await Promise.all([
      getOpenCodeConfig<OpenCodeConfig>(),
      getOpenCodeTuiSettings<OpenCodeTuiConfig>(),
      listOpenCodeProviders<OpenCodeProviderConfig[]>(),
      listOpenCodeMcpServers<OpenCodeMcpServer[]>(),
      listOpenCodeAgents<OpenCodeAgent[]>(),
      listOpenCodeCommands<OpenCodeCommand[]>(),
      listOpenCodePlugins<string[]>(),
      listOpenCodeLocalPlugins<OpenCodeLocalPluginFile[]>(),
      listOpenCodeSkillLocations<OpenCodeSkillLocation[]>(),
    ])

    config.value = configValue
    tui.value = tuiValue
    providers.value = providerList
    mcpServers.value = mcpList
    agents.value = agentList
    commands.value = commandList
    plugins.value = pluginList.map((name) => ({ name }))
    localPlugins.value = localPluginList
    skillLocations.value = skillLocationList
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void loadOverview()
})
</script>

<style scoped>
.opencode-view {
  @apply relative min-h-full px-4 py-4 sm:px-6 sm:py-6;
}

.opencode-shell {
  @apply mx-auto flex max-w-[1480px] flex-col gap-5;
}

.opencode-grid {
  @apply grid gap-5;
}

.opencode-grid--hero {
  @apply xl:grid-cols-[minmax(0,2fr)_380px];
}

.opencode-grid--capabilities {
  @apply md:grid-cols-2 xl:grid-cols-4;
}

.opencode-grid--detail {
  @apply xl:grid-cols-2;
}

.opencode-hero-card,
.opencode-panel,
.opencode-side-panel {
  @apply relative overflow-hidden p-5;
}

.opencode-hero-card__glow {
  @apply pointer-events-none absolute right-[-4rem] top-[-5rem] h-56 w-56 rounded-full;

  background: radial-gradient(circle, rgb(163 230 53 / 20%), transparent 70%);
}

.opencode-hero-card__content {
  @apply relative z-10 flex flex-col gap-5;
}

.opencode-hero-head {
  @apply flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between;
}

.opencode-hero-title-row {
  @apply flex items-start gap-4;
}

.opencode-hero-icon {
  @apply flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl border border-lime-300/25 bg-lime-300/10;
}

.opencode-hero-eyebrow {
  @apply mb-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-text-muted;
}

.opencode-hero-title {
  @apply text-3xl font-semibold tracking-[-0.04em] text-text-primary;
}

.opencode-hero-subtitle {
  @apply mt-2 max-w-3xl text-sm leading-7 text-text-secondary;
}

.opencode-pill-row,
.opencode-action-row,
.opencode-hero-stats,
.opencode-runtime-strip,
.opencode-discovery-grid {
  @apply flex flex-wrap gap-3;
}

.opencode-pill,
.opencode-runtime-chip,
.opencode-command-row__note,
.opencode-discovery-card {
  @apply inline-flex items-center rounded-2xl border px-3 py-2 text-sm;
}

.opencode-pill--lime {
  @apply border-lime-300/20 bg-lime-300/10 text-lime-200;
}

.opencode-pill--neutral {
  @apply border-border-default/55 bg-bg-base/35 text-text-secondary;
}

.opencode-pill--violet {
  @apply border-violet-300/25 bg-violet-300/10 text-violet-200;
}

.opencode-stat-card,
.opencode-tool-card,
.opencode-topology-item,
.opencode-command-row,
.opencode-discovery-card {
  @apply rounded-3xl border border-border-default/55 bg-bg-base/35 p-4;
}

.opencode-stat-card {
  @apply min-w-[220px] flex-1;
}

.opencode-stat-label,
.opencode-runtime-chip__label,
.opencode-side-panel__footer-label,
.opencode-discovery-card__label {
  @apply text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.opencode-stat-value,
.opencode-discovery-card__value {
  @apply mt-2 text-2xl font-semibold tracking-[-0.03em] text-text-primary;
}

.opencode-stat-detail,
.opencode-discovery-card__detail,
.opencode-side-panel__footer-value {
  @apply mt-2 text-sm leading-6 text-text-secondary;
}

.opencode-side-panel__header,
.opencode-panel__header {
  @apply mb-4 flex items-start gap-3;
}

.opencode-side-panel__icon {
  @apply flex h-10 w-10 items-center justify-center rounded-2xl border border-cyan-300/20 bg-cyan-300/10;
}

.opencode-side-panel__title,
.opencode-panel__title {
  @apply text-lg font-semibold text-text-primary;
}

.opencode-side-panel__description,
.opencode-panel__description,
.opencode-capability-card__description,
.opencode-command-row__description,
.opencode-tool-card__description,
.opencode-topology-item__description {
  @apply mt-1 text-sm leading-6 text-text-secondary;
}

.opencode-runtime-chip {
  @apply flex min-w-[150px] flex-1 flex-col items-start gap-1 border-border-default/55 bg-bg-base/35;
}

.opencode-side-panel__footer {
  @apply mt-4 flex items-center justify-between gap-4 rounded-3xl border border-border-default/55 bg-bg-base/30 p-4;
}

.opencode-text-link {
  @apply text-sm font-medium text-lime-200 transition-colors hover:text-lime-100;
}

.opencode-capability-card {
  @apply h-full p-4;
}

.opencode-capability-card__header {
  @apply mb-4 flex items-center justify-between gap-3;
}

.opencode-capability-card__icon {
  @apply flex h-11 w-11 items-center justify-center rounded-2xl border;
}

.opencode-capability-card__icon--lime {
  @apply border-lime-300/20 bg-lime-300/10 text-lime-200;
}

.opencode-capability-card__icon--violet {
  @apply border-violet-300/20 bg-violet-300/10 text-violet-200;
}

.opencode-capability-card__icon--cyan {
  @apply border-cyan-300/20 bg-cyan-300/10 text-cyan-200;
}

.opencode-capability-card__icon--amber {
  @apply border-amber-300/20 bg-amber-300/10 text-amber-200;
}

.opencode-capability-card__icon--emerald {
  @apply border-emerald-300/20 bg-emerald-300/10 text-emerald-200;
}

.opencode-capability-card__title {
  @apply text-base font-semibold text-text-primary;
}

.opencode-capability-card__arrow {
  @apply text-text-muted transition-transform duration-200 group-hover:translate-x-1;
}

.opencode-command-list,
.opencode-topology-list,
.opencode-tool-grid {
  @apply flex flex-col gap-3;
}

.opencode-command-row {
  @apply flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between;
}

.opencode-command-row__command,
.opencode-topology-item__path,
.opencode-tool-card__meta strong {
  @apply font-mono text-sm text-text-primary;
}

.opencode-command-row__note {
  @apply border-border-default/55 bg-bg-base/45 text-text-secondary;
}

.opencode-tool-card__meta {
  @apply mb-2 flex items-center justify-between gap-3;
}

.opencode-tool-card__meta span {
  @apply rounded-full bg-bg-base/55 px-2 py-1 text-[11px] uppercase tracking-[0.14em] text-text-muted;
}

.opencode-tool-card__availability {
  @apply mt-3 text-xs text-amber-200;
}

.opencode-topology-item__title {
  @apply block text-sm font-semibold text-text-primary;
}

.opencode-topology-item__path {
  @apply mt-2 block break-all;
}
</style>
