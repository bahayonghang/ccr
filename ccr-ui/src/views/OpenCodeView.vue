<template>
  <div class="opencode-view stage-page">
    <div class="opencode-shell">
      <Card
        variant="glass"
        class="opencode-ops-board"
      >
        <div class="opencode-ops-board__glow opencode-ops-board__glow--lime" />
        <div class="opencode-ops-board__glow opencode-ops-board__glow--cyan" />

        <div class="opencode-ops-board__content">
          <section class="opencode-identity-panel">
            <div class="opencode-identity-panel__head">
              <div class="opencode-hero-icon">
                <SIcon
                  name="TerminalSquare"
                  size="w-5 h-5"
                  class="text-lime-300"
                />
              </div>
              <div>
                <div class="opencode-eyebrow">
                  OpenCode operator deck
                </div>
                <h1 class="opencode-title">
                  Operational console
                </h1>
              </div>
            </div>

            <p class="opencode-subtitle">
              高密度收敛 provider、MCP、agents、commands、plugins 与 runtime 配置；首屏直接进入可操作状态。
            </p>

            <div class="opencode-path-stack">
              <div
                v-for="item in identityItems"
                :key="item.label"
                class="opencode-path-chip"
              >
                <span>{{ item.label }}</span>
                <strong>{{ item.value }}</strong>
              </div>
            </div>
          </section>

          <section
            class="opencode-metrics-panel"
            aria-label="OpenCode live metrics"
          >
            <div class="opencode-panel-kicker">
              Live metrics
            </div>
            <div class="opencode-live-grid">
              <div
                v-for="metric in liveMetrics"
                :key="metric.label"
                class="opencode-live-metric"
              >
                <span>{{ metric.label }}</span>
                <strong>{{ metric.value }}</strong>
                <small>{{ metric.detail }}</small>
              </div>
            </div>

            <div class="opencode-runtime-strip">
              <div
                v-for="chip in runtimeChips"
                :key="chip.label"
                class="opencode-runtime-chip"
              >
                <span
                  class="opencode-status-dot"
                  :class="`opencode-status-dot--${chip.tone}`"
                />
                <span>{{ chip.label }}</span>
                <strong>{{ chip.value }}</strong>
              </div>
            </div>
          </section>

          <aside class="opencode-actions-panel">
            <div class="opencode-actions-panel__head">
              <div>
                <div class="opencode-panel-kicker">
                  Next actions
                </div>
                <p>{{ overviewStatusLabel }}</p>
              </div>
              <button
                type="button"
                class="opencode-refresh-button"
                :disabled="loading"
                @click="loadOverview()"
              >
                <SIcon
                  name="RefreshCw"
                  size="w-4 h-4"
                  :class="{ 'animate-spin': loading }"
                />
                <span>{{ loading ? 'Loading' : 'Refresh' }}</span>
              </button>
            </div>

            <div class="opencode-action-stack">
              <RouterLink
                v-for="action in nextActions"
                :key="action.href"
                :to="action.href"
                class="opencode-action-link"
              >
                <span>{{ action.label }}</span>
                <strong>{{ action.detail }}</strong>
              </RouterLink>
            </div>

            <div
              v-if="warningItems.length > 0"
              class="opencode-warning-strip"
              role="status"
            >
              <span
                v-for="warning in warningItems"
                :key="warning.key"
                class="opencode-warning-chip"
              >
                {{ warning.label }} warning
              </span>
            </div>
          </aside>
        </div>
      </Card>

      <section
        class="opencode-capability-grid"
        aria-label="OpenCode capability entries"
      >
        <RouterLink
          v-for="card in capabilityDeck"
          :key="card.id"
          :to="card.href"
          class="opencode-capability-link group"
        >
          <Card
            variant="elevated"
            hover
            class="opencode-capability-card"
          >
            <div class="opencode-capability-card__topline">
              <div
                class="opencode-capability-card__icon"
                :class="`opencode-capability-card__icon--${card.tone}`"
              >
                <SIcon
                  :name="card.icon"
                  size="w-5 h-5"
                />
              </div>
              <span
                class="opencode-capability-card__badge"
                :class="{ 'opencode-capability-card__badge--warn': card.status === 'warning' }"
              >
                {{ card.badge }}
              </span>
            </div>
            <div>
              <h2 class="opencode-capability-card__title">
                {{ card.title }}
              </h2>
              <p class="opencode-capability-card__description">
                {{ card.description }}
              </p>
            </div>
            <div class="opencode-capability-card__footer">
              <span>{{ card.cta }}</span>
              <SIcon
                name="ArrowRight"
                size="w-4 h-4"
                class="opencode-capability-card__arrow"
              />
            </div>
          </Card>
        </RouterLink>
      </section>

      <Card
        variant="glass"
        class="opencode-inspector"
      >
        <div class="opencode-inspector__head">
          <div>
            <div class="opencode-panel-kicker">
              Compact inspector
            </div>
            <h2>Runtime intelligence</h2>
          </div>
          <div
            class="opencode-inspector-tabs"
            role="tablist"
            aria-label="OpenCode inspector sections"
          >
            <button
              v-for="tab in inspectorTabs"
              :key="tab.id"
              type="button"
              class="opencode-inspector-tab"
              :class="{ 'opencode-inspector-tab--active': activeInspector === tab.id }"
              :aria-selected="activeInspector === tab.id"
              role="tab"
              @click="activeInspector = tab.id"
            >
              {{ tab.label }}
            </button>
          </div>
        </div>

        <div
          v-if="activeInspector === 'runtime'"
          class="opencode-inspector-grid opencode-inspector-grid--commands"
          role="tabpanel"
        >
          <div
            v-for="item in opencodeCliCommands"
            :key="item.command"
            class="opencode-command-row"
          >
            <code>{{ item.command }}</code>
            <span>{{ item.description }}</span>
            <strong v-if="item.note">{{ item.note }}</strong>
          </div>
        </div>

        <div
          v-else-if="activeInspector === 'tools'"
          class="opencode-inspector-grid opencode-inspector-grid--tools"
          role="tabpanel"
        >
          <div
            v-for="tool in opencodeBuiltInTools"
            :key="tool.id"
            class="opencode-tool-card"
          >
            <div>
              <strong>{{ tool.id }}</strong>
              <span>{{ tool.permissionKey }}</span>
            </div>
            <p>{{ tool.description }}</p>
            <small v-if="tool.availability">{{ tool.availability }}</small>
          </div>
        </div>

        <div
          v-else-if="activeInspector === 'topology'"
          class="opencode-inspector-grid opencode-inspector-grid--topology"
          role="tabpanel"
        >
          <div
            v-for="item in opencodeConfigTopology"
            :key="item.path"
            class="opencode-topology-item"
          >
            <span>{{ item.title }}</span>
            <code>{{ item.path }}</code>
            <p>{{ item.description }}</p>
          </div>
        </div>

        <div
          v-else
          class="opencode-inspector-grid opencode-inspector-grid--discovery"
          role="tabpanel"
        >
          <div class="opencode-discovery-card">
            <span>Local plugins</span>
            <strong>{{ localPlugins.length }}</strong>
            <p>{{ localPluginPreview }}</p>
          </div>
          <div class="opencode-discovery-card">
            <span>Agents</span>
            <strong>{{ agents.length }}</strong>
            <p>{{ agentModePreview }}</p>
          </div>
          <div class="opencode-discovery-card">
            <span>Commands</span>
            <strong>{{ commands.length }}</strong>
            <p>{{ commandScopePreview }}</p>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import Card from '@/components/ui/Card.vue'
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
  OpenCodeTuiConfig,
} from '@/types'

const loading = ref(false)
const loadedOnce = ref(false)
const loadErrors = ref<Record<string, string>>({})
const activeInspector = ref('runtime')
const config = ref<OpenCodeConfig>({})
const tui = ref<OpenCodeTuiConfig>({})
const providers = ref<OpenCodeProviderConfig[]>([])
const mcpServers = ref<OpenCodeMcpServer[]>([])
const agents = ref<OpenCodeAgent[]>([])
const commands = ref<OpenCodeCommand[]>([])
const plugins = ref<OpenCodePluginPackage[]>([])
const localPlugins = ref<OpenCodeLocalPluginFile[]>([])

const configPathLabel = '~/.config/opencode/opencode.json'
const tuiPathLabel = '~/.config/opencode/tui.json'

const inspectorTabs = [
  { id: 'runtime', label: 'CLI runtime' },
  { id: 'tools', label: 'Built-in tools' },
  { id: 'topology', label: 'Config topology' },
  { id: 'discovery', label: 'Local discovery' },
]

const activeModelLabel = computed(() => config.value.model || 'not configured')
const defaultAgentLabel = computed(() => config.value.default_agent || 'build')
const themeLabel = computed(() => tui.value.theme || 'system')
const shareLabel = computed(() => config.value.share || 'manual')
const serverLabel = computed(() => {
  const port = config.value.server?.port ?? 4096
  const host = config.value.server?.hostname || 'localhost'
  return `${host}:${port}`
})
const webLabel = computed(() => config.value.server?.cors?.length ? 'cors configured' : 'same host')
const overviewStatusLabel = computed(() => {
  if (loading.value && !loadedOnce.value) return 'Loading local OpenCode surfaces…'
  if (warningItems.value.length > 0) return `${warningItems.value.length} degraded source(s), usable data kept visible.`
  if (loadedOnce.value) return 'All local surfaces are available.'
  return 'Ready to read local settings.'
})

const identityItems = computed(() => [
  { label: 'config', value: configPathLabel },
  { label: 'tui', value: tuiPathLabel },
  { label: 'default agent', value: defaultAgentLabel.value },
])

const liveMetrics = computed(() => [
  { label: 'Providers', value: providers.value.length, detail: activeModelLabel.value },
  { label: 'MCP', value: mcpServers.value.length, detail: `${agents.value.length} agents` },
  { label: 'Commands', value: commands.value.length, detail: `${plugins.value.length + localPlugins.value.length} plugins` },
  { label: 'Theme', value: themeLabel.value, detail: `share ${shareLabel.value}` },
])

const runtimeChips = computed(() => [
  { label: 'serve', value: serverLabel.value, tone: loadErrors.value.config ? 'warn' : 'ok' },
  { label: 'web', value: webLabel.value, tone: 'info' },
  { label: 'acp', value: 'available', tone: 'ok' },
  { label: 'share', value: shareLabel.value, tone: shareLabel.value === 'manual' ? 'idle' : 'ok' },
])

const nextActions = computed(() => [
  { label: 'Provider matrix', href: '/opencode/providers', detail: `${providers.value.length} configured` },
  { label: 'MCP wiring', href: '/opencode/mcp', detail: `${mcpServers.value.length} servers` },
  { label: 'Runtime settings', href: '/opencode/settings', detail: themeLabel.value },
])

const warningItems = computed(() => Object.entries(loadErrors.value).map(([key]) => ({
  key,
  label: key.replace(/([A-Z])/g, ' $1').toLowerCase(),
})))

const capabilityCounts = computed<Record<string, number>>(() => ({
  providers: providers.value.length,
  mcp: mcpServers.value.length,
  agents: agents.value.length,
  commands: commands.value.length,
  plugins: plugins.value.length + localPlugins.value.length,
  settings: Object.keys(config.value).length + Object.keys(tui.value).length,
}))

const capabilityErrors = computed<Record<string, boolean>>(() => ({
  providers: Boolean(loadErrors.value.providers || loadErrors.value.config),
  mcp: Boolean(loadErrors.value.mcp || loadErrors.value.config),
  agents: Boolean(loadErrors.value.agents),
  commands: Boolean(loadErrors.value.commands),
  plugins: Boolean(loadErrors.value.plugins || loadErrors.value.localPlugins || loadErrors.value.config),
  settings: Boolean(loadErrors.value.config || loadErrors.value.tui),
}))

const capabilityDeck = computed(() => opencodeCapabilityCards.map((card) => {
  const count = capabilityCounts.value[card.id] ?? 0
  const failed = capabilityErrors.value[card.id]
  return {
    ...card,
    badge: failed ? 'warning' : `${count} live`,
    cta: failed ? 'Retry or inspect' : 'Open surface',
    status: failed ? 'warning' : 'ok',
  }
}))

const localPluginPreview = computed(() => (
  localPlugins.value.slice(0, 3).map((item) => item.name).join(', ') || 'No plugin files detected'
))
const agentModePreview = computed(() => {
  const primary = agents.value.filter((agent) => agent.mode === 'primary').length
  const subagent = agents.value.filter((agent) => agent.mode === 'subagent').length
  return `${primary} primary · ${subagent} subagent · ${Math.max(agents.value.length - primary - subagent, 0)} mixed`
})
const commandScopePreview = computed(() => {
  const project = commands.value.filter((command) => command.scope === 'project').length
  return `${project} project · ${commands.value.length - project} global/builtin`
})

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}

async function loadOverview() {
  loading.value = true

  const tasks = {
    config: getOpenCodeConfig<OpenCodeConfig>(),
    tui: getOpenCodeTuiSettings<OpenCodeTuiConfig>(),
    providers: listOpenCodeProviders<OpenCodeProviderConfig[]>(),
    mcp: listOpenCodeMcpServers<OpenCodeMcpServer[]>(),
    agents: listOpenCodeAgents<OpenCodeAgent[]>(),
    commands: listOpenCodeCommands<OpenCodeCommand[]>(),
    plugins: listOpenCodePlugins<string[]>(),
    localPlugins: listOpenCodeLocalPlugins<OpenCodeLocalPluginFile[]>(),
  }

  const entries = Object.entries(tasks)
  const results = await Promise.allSettled(entries.map(([, task]) => task))
  const nextErrors: Record<string, string> = {}

  results.forEach((result, index) => {
    const key = entries[index]?.[0]
    if (!key) return

    if (result.status === 'rejected') {
      nextErrors[key] = errorMessage(result.reason)
      return
    }

    switch (key) {
      case 'config':
        config.value = result.value as OpenCodeConfig
        break
      case 'tui':
        tui.value = result.value as OpenCodeTuiConfig
        break
      case 'providers':
        providers.value = result.value as OpenCodeProviderConfig[]
        break
      case 'mcp':
        mcpServers.value = result.value as OpenCodeMcpServer[]
        break
      case 'agents':
        agents.value = result.value as OpenCodeAgent[]
        break
      case 'commands':
        commands.value = result.value as OpenCodeCommand[]
        break
      case 'plugins':
        plugins.value = (result.value as string[]).map((name) => ({ name }))
        break
      case 'localPlugins':
        localPlugins.value = result.value as OpenCodeLocalPluginFile[]
        break
    }
  })

  loadErrors.value = nextErrors
  loadedOnce.value = true
  loading.value = false
}

onMounted(() => {
  void loadOverview()
})
</script>

<style scoped>
.opencode-view {
  @apply relative min-h-full px-4 py-4 sm:px-6;
}

.opencode-shell {
  @apply mx-auto flex max-w-[1480px] flex-col gap-4;
}

.opencode-ops-board,
.opencode-inspector {
  @apply relative overflow-hidden p-4 sm:p-5;
}

.opencode-ops-board__glow {
  @apply pointer-events-none absolute rounded-full blur-3xl;
}

.opencode-ops-board__glow--lime {
  @apply right-[-5rem] top-[-6rem] h-56 w-56 bg-lime-300/20;
}

.opencode-ops-board__glow--cyan {
  @apply bottom-[-7rem] left-[30%] h-52 w-52 bg-cyan-300/10;
}

.opencode-ops-board__content {
  @apply relative z-10 grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(420px,1fr)_340px];
}

.opencode-identity-panel,
.opencode-metrics-panel,
.opencode-actions-panel {
  @apply rounded-[1.75rem] border border-border-default/55 bg-bg-base/35 p-4;
}

.opencode-identity-panel__head,
.opencode-actions-panel__head,
.opencode-capability-card__topline,
.opencode-capability-card__footer,
.opencode-inspector__head {
  @apply flex items-center justify-between gap-3;
}

.opencode-hero-icon {
  @apply flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border border-lime-300/25 bg-lime-300/10;
}

.opencode-eyebrow,
.opencode-panel-kicker,
.opencode-live-metric span,
.opencode-path-chip span,
.opencode-discovery-card span {
  @apply text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.opencode-title {
  @apply text-2xl font-semibold tracking-[-0.04em] text-text-primary sm:text-3xl;
}

.opencode-subtitle {
  @apply mt-3 max-w-2xl text-sm leading-6 text-text-secondary;
}

.opencode-path-stack,
.opencode-runtime-strip,
.opencode-action-stack,
.opencode-warning-strip {
  @apply mt-4 flex flex-wrap gap-2;
}

.opencode-path-chip,
.opencode-runtime-chip,
.opencode-warning-chip,
.opencode-capability-card__badge,
.opencode-refresh-button,
.opencode-inspector-tab {
  @apply inline-flex items-center rounded-full border px-3 py-1.5 text-xs;
}

.opencode-path-chip {
  @apply max-w-full gap-2 border-border-default/55 bg-bg-base/45 text-text-secondary;
}

.opencode-path-chip strong {
  @apply truncate font-mono text-text-primary;
}

.opencode-live-grid {
  @apply mt-3 grid grid-cols-2 gap-2 lg:grid-cols-4;
}

.opencode-live-metric {
  @apply rounded-2xl border border-border-default/50 bg-bg-base/40 p-3;
}

.opencode-live-metric strong {
  @apply mt-1 block truncate text-2xl font-semibold tracking-[-0.04em] text-text-primary;
}

.opencode-live-metric small {
  @apply mt-1 block truncate text-xs text-text-secondary;
}

.opencode-runtime-chip {
  @apply gap-2 border-border-default/55 bg-bg-base/45 text-text-secondary;
}

.opencode-runtime-chip strong {
  @apply font-mono text-text-primary;
}

.opencode-status-dot {
  @apply h-2 w-2 rounded-full;
}

.opencode-status-dot--ok {
  @apply bg-lime-300 shadow-[0_0_14px_rgb(190_242_100_/_0.7)];
}

.opencode-status-dot--info {
  @apply bg-cyan-300 shadow-[0_0_14px_rgb(103_232_249_/_0.6)];
}

.opencode-status-dot--idle {
  @apply bg-text-muted;
}

.opencode-status-dot--warn {
  @apply bg-amber-300 shadow-[0_0_14px_rgb(252_211_77_/_0.65)];
}

.opencode-actions-panel p {
  @apply mt-1 text-sm leading-5 text-text-secondary;
}

.opencode-refresh-button {
  @apply gap-2 border-border-default/60 bg-bg-elevated/60 text-text-primary transition hover:border-lime-300/35 hover:text-lime-100 disabled:cursor-not-allowed disabled:opacity-60;
}

.opencode-action-link {
  @apply flex flex-1 basis-full items-center justify-between gap-3 rounded-2xl border border-border-default/55 bg-bg-base/40 px-3 py-2 text-sm text-text-secondary transition hover:border-lime-300/30 hover:bg-lime-300/10 hover:text-text-primary;
}

.opencode-action-link strong {
  @apply font-mono text-xs text-text-primary;
}

.opencode-warning-chip,
.opencode-capability-card__badge--warn {
  @apply border-amber-300/30 bg-amber-300/10 text-amber-100;
}

.opencode-capability-grid {
  @apply grid gap-3 md:grid-cols-2 xl:grid-cols-6;
}

.opencode-capability-link {
  @apply min-w-0 rounded-[1.75rem] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-lime-300/60;
}

.opencode-capability-card {
  @apply flex h-full min-h-[168px] flex-col justify-between gap-3 p-4 transition duration-200 group-hover:border-lime-300/35 group-hover:shadow-[0_18px_60px_rgb(132_204_22_/_0.12)];
}

.opencode-capability-card__icon {
  @apply flex h-10 w-10 items-center justify-center rounded-2xl border;
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

.opencode-capability-card__badge {
  @apply border-border-default/55 bg-bg-base/45 font-mono text-text-secondary;
}

.opencode-capability-card__title,
.opencode-inspector__head h2 {
  @apply text-base font-semibold text-text-primary;
}

.opencode-capability-card__description {
  @apply mt-1 line-clamp-2 text-sm leading-5 text-text-secondary;
}

.opencode-capability-card__footer {
  @apply text-sm font-medium text-lime-100;
}

.opencode-capability-card__arrow {
  @apply text-text-muted transition-transform duration-200 group-hover:translate-x-1 group-hover:text-lime-200;
}

.opencode-inspector__head {
  @apply mb-4 flex-col items-start sm:flex-row sm:items-center;
}

.opencode-inspector-tabs {
  @apply flex flex-wrap gap-2;
}

.opencode-inspector-tab {
  @apply border-border-default/55 bg-bg-base/35 text-text-secondary transition hover:border-cyan-300/30 hover:text-text-primary;
}

.opencode-inspector-tab--active {
  @apply border-cyan-300/40 bg-cyan-300/10 text-cyan-100 shadow-[0_0_22px_rgb(103_232_249_/_0.12)];
}

.opencode-inspector-grid {
  @apply grid gap-3;
}

.opencode-inspector-grid--commands,
.opencode-inspector-grid--tools,
.opencode-inspector-grid--topology,
.opencode-inspector-grid--discovery {
  @apply md:grid-cols-2 xl:grid-cols-3;
}

.opencode-command-row,
.opencode-tool-card,
.opencode-topology-item,
.opencode-discovery-card {
  @apply rounded-2xl border border-border-default/55 bg-bg-base/35 p-3;
}

.opencode-command-row code,
.opencode-topology-item code {
  @apply font-mono text-sm text-text-primary;
}

.opencode-command-row span,
.opencode-topology-item p,
.opencode-tool-card p,
.opencode-discovery-card p {
  @apply mt-1 block text-sm leading-5 text-text-secondary;
}

.opencode-command-row strong,
.opencode-tool-card small {
  @apply mt-2 block text-xs text-amber-100;
}

.opencode-tool-card div {
  @apply flex items-center justify-between gap-3;
}

.opencode-tool-card strong,
.opencode-topology-item span,
.opencode-discovery-card strong {
  @apply text-sm font-semibold text-text-primary;
}

.opencode-tool-card div span {
  @apply rounded-full bg-bg-base/55 px-2 py-1 font-mono text-[11px] uppercase tracking-[0.14em] text-text-muted;
}

.opencode-discovery-card strong {
  @apply mt-1 block text-2xl tracking-[-0.04em];
}
</style>
