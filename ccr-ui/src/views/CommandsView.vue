<template>
  <div class="commands-page">
    <div class="commands-shell">
      <PageHeaderCard
        :title="$t('commands.title')"
        :description="$t('commands.description')"
        badge="Command Center"
        icon="Terminal"
        tone="secondary"
      >
        <div class="commands-header-meta">
          <span class="commands-chip">
            <SIcon
              name="Cpu"
              size="w-3.5 h-3.5"
            />
            {{ selectedClient }}
          </span>
          <span
            class="commands-chip"
            :class="runtimeUnavailable ? 'commands-chip--warning' : ''"
          >
            <SIcon
              :name="runtimeUnavailable ? 'MonitorOff' : 'CheckCircle2'"
              size="w-3.5 h-3.5"
            />
            {{ runtimeUnavailable ? 'web preview' : 'desktop runtime' }}
          </span>
        </div>
      </PageHeaderCard>

      <div class="commands-layout">
        <aside class="commands-sidebar">
          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            class="commands-panel"
          >
            <div class="commands-panel__header">
              <h2 class="commands-panel__title">
                {{ $t('commands.selectClient') }}
              </h2>
            </div>

            <div class="commands-list">
              <button
                v-for="client in CLI_CLIENTS"
                :key="client.id"
                class="client-row"
                :class="{ 'client-row--active': selectedClient === client.id }"
                @click="setSelectedClient(client.id)"
              >
                <div
                  class="client-row__icon"
                  :class="selectedClient === client.id ? client.surfaceClass : 'bg-bg-surface text-text-secondary'"
                >
                  <SIcon
                    :name="client.icon || ''"
                    size="w-4 h-4"
                    :class="selectedClient === client.id ? client.textClass : 'text-text-secondary'"
                  />
                </div>
                <span class="client-row__label">{{ client.name }}</span>
                <SIcon
                  v-if="selectedClient === client.id"
                  name="ChevronRight"
                  size="w-4 h-4"
                  class="text-accent-secondary"
                />
              </button>
            </div>
          </Card>

          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            class="commands-panel commands-panel--fill"
          >
            <div class="commands-panel__header">
              <h2 class="commands-panel__title">
                {{ $t('commands.availableCommands') }}
              </h2>
            </div>

            <div class="commands-list commands-list--scroll">
              <button
                v-for="cmd in commands"
                :key="cmd.name"
                class="command-row"
                :class="{ 'command-row--active': selectedCommand === cmd.name }"
                @click="setSelectedCommand(cmd.name)"
              >
                <div class="command-row__head">
                  <strong>{{ cmd.name }}</strong>
                  <SIcon
                    v-if="selectedCommand === cmd.name"
                    name="ChevronRight"
                    size="w-4 h-4"
                    class="text-accent-secondary"
                  />
                </div>
                <p>{{ cmd.description }}</p>
              </button>
            </div>
          </Card>
        </aside>

        <main class="commands-main">
          <Card
            surface="card"
            :elevation="3"
            motion="standard"
            class="commands-panel"
          >
            <div class="commands-panel__header commands-panel__header--wide">
              <div>
                <h2 class="commands-panel__title">
                  {{ selectedCommandInfo?.name || 'Select a command' }}
                </h2>
                <p class="commands-panel__subtitle">
                  {{ selectedCommandInfo?.description || 'Choose a command to inspect its arguments and run it.' }}
                </p>
              </div>
              <Button
                variant="primary"
                density="compact"
                surface="card"
                motion="standard"
                :disabled="loading || runtimeUnavailable || !selectedCommand"
                @click="handleExecute"
              >
                {{ loading ? $t('common.loading') : 'Run command' }}
              </Button>
            </div>

            <div class="command-input-shell">
              <div class="command-input-shell__label">
                command input
              </div>
              <div class="command-input-shell__body">
                <span class="command-input-shell__prompt">➜</span>
                <span class="command-input-shell__home">~</span>
                <span class="command-input-shell__binary">{{ selectedClient }}</span>
                <span class="command-input-shell__binary">{{ selectedCommand }}</span>

                <div
                  v-if="selectedCommand === 'switch'"
                  class="flex-1 min-w-[220px]"
                >
                  <label
                    for="commands-switch-select"
                    class="sr-only"
                  >{{ $t('commands.argsPlaceholder') }}</label>
                  <select
                    id="commands-switch-select"
                    v-model="args"
                    class="command-input-shell__field"
                    :disabled="runtimeUnavailable"
                    @keydown.enter="!loading && handleExecute()"
                  >
                    <option
                      value=""
                      disabled
                    >
                      Select a configuration
                    </option>
                    <option
                      v-for="config in configs"
                      :key="config.name"
                      :value="config.name"
                    >
                      {{ config.name }}
                    </option>
                  </select>
                </div>

                <label
                  v-else
                  for="commands-args"
                  class="sr-only"
                >{{ $t('commands.argsPlaceholder') }}</label>
                <input
                  v-if="selectedCommand !== 'switch'"
                  id="commands-args"
                  v-model="args"
                  type="text"
                  :disabled="runtimeUnavailable"
                  :placeholder="runtimeUnavailable ? 'Desktop mode required for command execution' : $t('commands.argsPlaceholder')"
                  class="command-input-shell__field"
                  @keydown.enter="!loading && handleExecute()"
                >
              </div>
            </div>
          </Card>

          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            class="commands-panel commands-panel--fill"
          >
            <div class="commands-panel__header commands-panel__header--wide">
              <div>
                <h2 class="commands-panel__title">
                  {{ $t('commands.output') }}
                </h2>
                <p class="commands-panel__subtitle">
                  Command output, execution metadata, and runtime feedback.
                </p>
              </div>

              <div class="commands-panel__actions">
                <Button
                  variant="ghost"
                  density="compact"
                  surface="status"
                  motion="subtle"
                  :disabled="!output"
                  @click="handleCopyOutput"
                >
                  {{ $t('commands.copy') }}
                </Button>
                <Button
                  variant="ghost"
                  density="compact"
                  surface="status"
                  motion="subtle"
                  :disabled="!output"
                  @click="handleClearOutput"
                >
                  {{ $t('commands.clear') }}
                </Button>
              </div>
            </div>

            <AsyncStatePanel
              v-if="runtimeUnavailable && !output"
              state="runtime-unavailable"
              :title="runtimeCopy.title"
              :description="runtimeCopy.description"
              compact
            />

            <div
              v-else-if="loading"
              class="commands-output commands-output--loading"
            >
              <SIcon
                name="Loader2"
                size="w-8 h-8"
                class="animate-spin text-accent-secondary"
              />
              <span>Processing command…</span>
            </div>

            <div
              v-else-if="output"
              class="commands-output"
            >
              <div class="commands-output__echo">
                <span class="command-input-shell__prompt">➜</span>
                <span class="command-input-shell__home">~</span>
                <span>{{ selectedClient }} {{ selectedCommand }} {{ args }}</span>
              </div>

              <pre
                class="commands-output__body"
                :class="output.success ? 'text-text-primary' : 'text-accent-danger'"
              >{{ commandOutputText }}</pre>

              <div class="commands-output__meta">
                <span>Status: <strong :class="output.success ? 'text-accent-success' : 'text-accent-danger'">{{ output.success ? 'SUCCESS' : 'FAILED' }}</strong></span>
                <span>Code: <strong>{{ output.exit_code }}</strong></span>
                <span>Time: <strong>{{ output.duration_ms }}ms</strong></span>
              </div>
            </div>

            <AsyncStatePanel
              v-else
              state="empty"
              title="Ready to execute commands"
              description="Select a client, choose a command, and provide optional arguments."
              compact
            />
          </Card>
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { executeCommand, listCommands, listConfigs } from '@/api'
import type { CommandInfo, CommandResponse, ConfigItem } from '@/types'
import { normalizeCliClient, type CliClient } from '@/types/router'
import { logger } from '@/utils/logger'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { isTauriRuntime } from '@/utils/tauriRuntime'

const { t } = useI18n({ useScope: 'global' })
const route = useRoute()
const router = useRouter()

const runtimeUnavailable = computed(() => !isTauriRuntime())
const runtimeCopy = computed(() => getRuntimeUnavailableCopy('commands'))

const CLI_CLIENTS = [
  { id: 'ccr' as CliClient, name: 'CCR', icon: 'Zap', surfaceClass: 'bg-accent-primary/10', textClass: 'text-accent-primary' },
  { id: 'claude' as CliClient, name: 'Claude Code', icon: 'Code2', surfaceClass: 'bg-accent-secondary/10', textClass: 'text-accent-secondary' },
  { id: 'gemini' as CliClient, name: 'Gemini', icon: 'Gem', surfaceClass: 'bg-accent-info/10', textClass: 'text-accent-info' },
]

const selectedClient = ref<CliClient>('ccr')
const commands = ref<CommandInfo[]>([])
const selectedCommand = ref('')
const args = ref('')
const output = ref<CommandResponse | null>(null)
const loading = ref(false)
const configs = ref<ConfigItem[]>([])

const fallbackCommandRegistry: Record<CliClient, CommandInfo[]> = {
  ccr: [
    { name: 'help', description: 'Inspect the CCR command surface.', usage: 'ccr --help', examples: ['ccr --help'] },
    { name: 'switch', description: 'Switch to a saved CCR configuration.', usage: 'ccr switch <name>', examples: ['ccr switch default'] },
    { name: 'version', description: 'Inspect the installed CCR version.', usage: 'ccr --version', examples: ['ccr --version'] },
  ],
  claude: [
    { name: 'help', description: 'Inspect Claude Code CLI help.', usage: 'claude --help', examples: ['claude --help'] },
    { name: 'version', description: 'Inspect Claude Code CLI version.', usage: 'claude --version', examples: ['claude --version'] },
    { name: 'login', description: 'Authenticate Claude Code.', usage: 'claude login', examples: ['claude login'] },
  ],
  gemini: [
    { name: 'help', description: 'Inspect Gemini CLI help.', usage: 'gemini --help', examples: ['gemini --help'] },
    { name: 'version', description: 'Inspect Gemini CLI version.', usage: 'gemini --version', examples: ['gemini --version'] },
    { name: 'login', description: 'Authenticate Gemini CLI.', usage: 'gemini login', examples: ['gemini login'] },
  ],
}

const selectedCommandInfo = computed(() =>
  commands.value.find((command) => command.name === selectedCommand.value),
)

const commandOutputText = computed(() => output.value?.output || output.value?.error || '')

const isCommandResponse = (value: unknown): value is CommandResponse => {
  if (typeof value !== 'object' || value === null) {
    return false
  }
  const record = value as Record<string, unknown>
  return typeof record.success === 'boolean'
    && typeof record.output === 'string'
    && typeof record.error === 'string'
    && typeof record.exit_code === 'number'
    && typeof record.duration_ms === 'number'
}

const normalizeCommandResponse = (value: unknown): CommandResponse => {
  if (isCommandResponse(value)) {
    return value
  }
  return {
    success: false,
    output: '',
    error: t('commands.unknownError'),
    exit_code: -1,
    duration_ms: 0,
  }
}

const applyCommandList = (client: CliClient) => {
  commands.value = fallbackCommandRegistry[client]
  if (!selectedCommand.value || !commands.value.some((command) => command.name === selectedCommand.value)) {
    selectedCommand.value = commands.value[0]?.name ?? ''
  }
}

const loadConfigs = async () => {
  if (runtimeUnavailable.value) {
    configs.value = [
      { name: 'default' } as ConfigItem,
      { name: 'workspace' } as ConfigItem,
    ]
    return
  }

  try {
    const response = await listConfigs<{ configs: ConfigItem[] }>()
    configs.value = response.configs
  } catch (error) {
    logger.error('Failed to load configs:', error)
  }
}

const loadCommands = async () => {
  if (runtimeUnavailable.value || selectedClient.value !== 'ccr') {
    applyCommandList(selectedClient.value)
    return
  }

  try {
    const data = await listCommands<CommandInfo[]>()
    commands.value = data
    if (data.length > 0 && !selectedCommand.value) {
      selectedCommand.value = data[0].name
    }
  } catch (error) {
    logger.error('Failed to load commands:', error)
    applyCommandList(selectedClient.value)
  }
}

onMounted(() => {
  const initialClient = normalizeCliClient(route.params.client)
  if (initialClient) {
    selectedClient.value = initialClient
  }
  void loadCommands()
  void loadConfigs()
})

watch(
  () => route.params.client,
  (clientParam) => {
    const client = normalizeCliClient(clientParam)
    if (client && client !== selectedClient.value) {
      selectedClient.value = client
    }
  },
)

watch(selectedClient, () => {
  selectedCommand.value = ''
  args.value = ''
  output.value = null
  void loadCommands()

  const current = normalizeCliClient(route.params.client) || 'ccr'
  if (current !== selectedClient.value) {
    void router.replace({ name: 'commands', params: { client: selectedClient.value } })
  }
})

const setSelectedClient = (client: CliClient) => {
  selectedClient.value = client
}

const setSelectedCommand = (command: string) => {
  selectedCommand.value = command
  args.value = ''
}

const resolveClientBinary = (client: CliClient): string => {
  return client
}

const handleExecute = async () => {
  if (!selectedCommand.value || runtimeUnavailable.value) return

  loading.value = true
  try {
    const argsArray = args.value
      .split(' ')
      .map((arg) => arg.trim())
      .filter((arg) => arg.length > 0)

    if (selectedClient.value === 'ccr') {
      const result = await executeCommand({
        command: selectedCommand.value,
        args: argsArray,
      })
      output.value = normalizeCommandResponse(result)
    } else {
      const finalArgs = [...argsArray]

      if (selectedCommand.value === 'help' && !finalArgs.includes('--help')) {
        finalArgs.unshift('--help')
      } else if (selectedCommand.value === 'version' && !finalArgs.includes('--version')) {
        finalArgs.unshift('--version')
      } else if (selectedCommand.value === 'login' && !finalArgs.includes('login')) {
        finalArgs.unshift('login')
      }

      const result = await executeCommand({
        command: resolveClientBinary(selectedClient.value),
        args: finalArgs,
      })
      output.value = normalizeCommandResponse(result)
    }
  } catch (error) {
    output.value = {
      success: false,
      output: '',
      error: error instanceof Error ? error.message : t('commands.unknownError'),
      exit_code: -1,
      duration_ms: 0,
    }
  } finally {
    loading.value = false
  }
}

const handleCopyOutput = async () => {
  if (!output.value) return
  const text = output.value.output + (output.value.error ? `\n${output.value.error}` : '')
  try {
    await navigator.clipboard.writeText(text)
  } catch (error) {
    logger.error('Failed to copy:', error)
  }
}

const handleClearOutput = () => {
  output.value = null
}
</script>

<style scoped>
.commands-page {
  @apply px-4 py-4 sm:px-6 sm:py-6;
}

.commands-shell {
  @apply mx-auto flex max-w-[1440px] flex-col gap-5;
}

.commands-header-meta {
  @apply flex flex-wrap gap-2;
}

.commands-chip {
  @apply inline-flex items-center gap-1.5 rounded-full border border-border-default/55 px-3 py-1 text-xs font-medium text-text-secondary;

  background-color: rgb(var(--color-bg-elevated-rgb) / 72%);
}

.commands-chip--warning {
  @apply border-accent-warning/25 bg-accent-warning/10 text-accent-warning;
}

.commands-layout {
  @apply grid gap-5 xl:grid-cols-[300px_minmax(0,1fr)];
}

.commands-sidebar,
.commands-main {
  @apply flex flex-col gap-5;
}

.commands-panel {
  @apply p-5;
}

.commands-panel--fill {
  @apply min-h-[420px];
}

.commands-panel__header {
  @apply mb-4 flex items-start justify-between gap-4;
}

.commands-panel__header--wide {
  @apply flex-wrap;
}

.commands-panel__title {
  @apply text-base font-semibold text-text-primary;
}

.commands-panel__subtitle {
  @apply mt-1 max-w-2xl text-sm text-text-secondary;
}

.commands-panel__actions {
  @apply flex flex-wrap items-center gap-2;
}

.commands-list {
  @apply flex flex-col gap-2;
}

.commands-list--scroll {
  @apply max-h-[460px] overflow-y-auto pr-1;
}

.client-row,
.command-row {
  @apply flex w-full items-start gap-3 rounded-2xl border border-transparent px-3 py-3 text-left transition-colors duration-200;
}

.client-row:hover,
.command-row:hover {
  background-color: rgb(var(--color-bg-surface-rgb) / 62%);
  border-color: rgb(var(--color-border-default-rgb) / 60%);
}

.client-row--active,
.command-row--active {
  background: linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 12%), rgb(var(--color-accent-secondary-rgb) / 9%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
}

.client-row__icon {
  @apply flex h-9 w-9 items-center justify-center rounded-xl border border-border-default/35;
}

.client-row__label {
  @apply flex-1 text-sm font-medium text-text-primary;
}

.command-row {
  @apply flex-col gap-2;
}

.command-row__head {
  @apply flex w-full items-center justify-between gap-2;
}

.command-row__head strong {
  @apply font-mono text-sm font-semibold text-text-primary;
}

.command-row p {
  @apply text-sm leading-relaxed text-text-secondary;
}

.command-input-shell {
  @apply rounded-2xl border border-border-default/55 p-4;

  background-color: rgb(var(--color-bg-base-rgb) / 92%);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 6%);
}

.command-input-shell__label {
  @apply mb-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-text-muted;
}

.command-input-shell__body {
  @apply flex flex-wrap items-center gap-3 font-mono text-sm;
}

.command-input-shell__prompt {
  @apply font-semibold text-accent-success;
}

.command-input-shell__home {
  @apply font-semibold text-accent-info;
}

.command-input-shell__binary {
  @apply font-semibold text-text-primary;
}

.command-input-shell__field {
  @apply min-h-[44px] min-w-[220px] flex-1 rounded-xl border border-border-default/60 px-3 py-2 text-sm text-text-primary outline-none transition-colors duration-200 placeholder:text-text-muted;

  background-color: rgb(var(--color-bg-elevated-rgb) / 62%);
}

.command-input-shell__field:focus {
  @apply border-accent-secondary/35;
}

.command-input-shell__field:disabled {
  @apply cursor-not-allowed opacity-60;
}

.commands-output {
  @apply flex min-h-[260px] flex-col rounded-2xl border border-border-default/50 p-4 font-mono text-sm;

  background-color: rgb(var(--color-bg-base-rgb) / 72%);
}

.commands-output--loading {
  @apply items-center justify-center gap-3 text-text-secondary;
}

.commands-output__echo {
  @apply mb-4 flex flex-wrap items-center gap-2 text-xs text-text-muted;
}

.commands-output__body {
  @apply flex-1 whitespace-pre-wrap break-words leading-relaxed;
}

.commands-output__meta {
  @apply mt-5 flex flex-wrap gap-4 border-t border-border-default/50 pt-4 text-xs text-text-secondary;
}

.commands-output__meta strong {
  @apply font-semibold text-text-primary;
}
</style>
