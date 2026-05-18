<template>
  <div class="commands-page">
    <div class="commands-shell">
      <PageHeaderCard
        :title="t('commands.title')"
        :description="t('commands.description')"
        :badge="t('commands.operatorBadge')"
        icon="Terminal"
        tone="secondary"
      >
        <div class="commands-header-meta">
          <span
            class="commands-chip"
            :class="canRun ? 'commands-chip--success' : 'commands-chip--warning'"
          >
            <SIcon
              :name="canRun ? 'CheckCircle2' : 'AlertTriangle'"
              size="w-3.5 h-3.5"
            />
            {{ readinessLabel }}
          </span>
          <span class="commands-chip">
            <SIcon
              name="Cpu"
              size="w-3.5 h-3.5"
            />
            {{ selectedClientLabel }}
          </span>
          <span class="commands-chip">
            <SIcon
              name="ShieldCheck"
              size="w-3.5 h-3.5"
            />
            {{ t('commands.whitelistBadge', { count: executableCommandCount }) }}
          </span>
        </div>
      </PageHeaderCard>

      <section class="commands-status-grid">
        <Card
          v-for="item in readinessCards"
          :key="item.key"
          surface="workspace"
          :elevation="1"
          motion="subtle"
          class="commands-status-card"
        >
          <div
            class="commands-status-card__icon"
            :class="`commands-status-card__icon--${item.tone}`"
          >
            <SIcon
              :name="item.icon"
              size="w-4 h-4"
            />
          </div>
          <div>
            <p class="commands-status-card__label">
              {{ item.label }}
            </p>
            <strong>{{ item.value }}</strong>
            <span>{{ item.detail }}</span>
          </div>
        </Card>
      </section>

      <div class="commands-workbench">
        <aside class="commands-palette">
          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            class="commands-panel commands-panel--palette"
          >
            <div class="commands-panel__header">
              <div>
                <h2 class="commands-panel__title">
                  {{ t('commands.paletteTitle') }}
                </h2>
                <p class="commands-panel__subtitle">
                  {{ t('commands.paletteSubtitle') }}
                </p>
              </div>
            </div>

            <div class="commands-client-switcher">
              <button
                v-for="client in CLI_CLIENTS"
                :key="client.id"
                type="button"
                class="commands-client-pill"
                :class="{
                  'commands-client-pill--active': selectedClient === client.id,
                  'commands-client-pill--disabled': !client.executable,
                }"
                @click="setSelectedClient(client.id)"
              >
                <SIcon
                  :name="client.icon"
                  size="w-4 h-4"
                />
                <span>{{ client.name }}</span>
                <small v-if="!client.executable">{{ t('commands.clientPreview') }}</small>
              </button>
            </div>

            <label
              for="commands-search"
              class="commands-search"
            >
              <SIcon
                name="Search"
                size="w-4 h-4"
              />
              <input
                id="commands-search"
                v-model="searchQuery"
                type="search"
                :placeholder="t('commands.searchPlaceholder')"
              >
            </label>

            <div class="commands-category-tabs">
              <button
                v-for="category in categoryTabs"
                :key="category"
                type="button"
                :class="{ 'commands-category-tabs__item--active': activeCategory === category }"
                class="commands-category-tabs__item"
                @click="activeCategory = category"
              >
                {{ categoryLabel(category) }}
              </button>
            </div>

            <div class="commands-list">
              <button
                v-for="cmd in filteredCommands"
                :key="cmd.name"
                type="button"
                class="command-row"
                :class="{
                  'command-row--active': selectedCommand === cmd.name,
                  'command-row--disabled': !cmd.executable,
                }"
                @click="setSelectedCommand(cmd.name)"
              >
                <div class="command-row__topline">
                  <strong>{{ cmd.name }}</strong>
                  <span
                    v-for="badge in commandBadges(cmd)"
                    :key="badge"
                    class="command-badge"
                    :class="`command-badge--${badge}`"
                  >
                    {{ badgeLabel(badge) }}
                  </span>
                </div>
                <p>{{ cmd.description }}</p>
              </button>
            </div>
          </Card>
        </aside>

        <main class="commands-main-grid">
          <Card
            surface="card"
            :elevation="3"
            motion="standard"
            class="commands-panel commands-composer"
          >
            <div class="commands-panel__header commands-panel__header--wide">
              <div>
                <p class="commands-panel__eyebrow">
                  {{ t('commands.composerEyebrow') }}
                </p>
                <h2 class="commands-panel__title commands-panel__title--large">
                  {{ selectedCommandInfo?.name || t('commands.selectCommand') }}
                </h2>
                <p class="commands-panel__subtitle">
                  {{ selectedCommandInfo?.description || t('commands.selectCommandHint') }}
                </p>
              </div>
              <div class="commands-composer__actions">
                <Button
                  v-if="isRunning"
                  variant="danger"
                  density="compact"
                  surface="card"
                  motion="standard"
                  :disabled="!currentSnapshot"
                  @click="handleCancel"
                >
                  <template #leading>
                    <SIcon
                      name="Square"
                      size="w-4 h-4"
                    />
                  </template>
                  {{ t('commands.cancelJob') }}
                </Button>
                <Button
                  variant="primary"
                  density="compact"
                  surface="card"
                  motion="standard"
                  :disabled="!canExecuteSelected"
                  :loading="isRunning"
                  @click="handleExecute"
                >
                  <template #leading>
                    <SIcon
                      name="Play"
                      size="w-4 h-4"
                    />
                  </template>
                  {{ isRunning ? t('commands.executing') : t('commands.run') }}
                </Button>
              </div>
            </div>

            <AsyncStatePanel
              v-if="runtimeUnavailable"
              state="runtime-unavailable"
              :title="runtimeCopy.title"
              :description="t('commands.webUnavailableDetail')"
              compact
              class="commands-runtime-panel"
            />

            <div
              v-else-if="selectedClient !== 'ccr'"
              class="commands-notice commands-notice--warning"
            >
              <SIcon
                name="Lock"
                size="w-5 h-5"
              />
              <div>
                <strong>{{ t('commands.clientUnavailableTitle') }}</strong>
                <p>{{ t('commands.clientUnavailableDescription', { client: selectedClientLabel }) }}</p>
              </div>
            </div>

            <div
              v-else-if="selectedCommandInfo && !selectedCommandInfo.executable"
              class="commands-notice commands-notice--warning"
            >
              <SIcon
                name="Shield"
                size="w-5 h-5"
              />
              <div>
                <strong>{{ t('commands.commandBlockedTitle') }}</strong>
                <p>{{ t('commands.commandBlockedDescription') }}</p>
              </div>
            </div>

            <div class="command-preview">
              <div class="command-preview__label">
                {{ t('commands.previewLabel') }}
              </div>
              <div class="command-preview__body">
                <span class="command-preview__prompt">➜</span>
                <span class="command-preview__binary">{{ commandBinary }}</span>
                <span class="command-preview__command">{{ selectedCommand || '<command>' }}</span>
                <span
                  v-if="args.trim()"
                  class="command-preview__args"
                >{{ args }}</span>
              </div>
            </div>

            <div class="commands-form-grid">
              <label class="commands-field">
                <span>{{ t('commands.args') }}</span>
                <select
                  v-if="selectedCommand === 'switch'"
                  v-model="args"
                  :disabled="!canEditArgs"
                >
                  <option value="">
                    {{ t('commands.selectConfig') }}
                  </option>
                  <option
                    v-for="config in configs"
                    :key="config.name"
                    :value="config.name"
                  >
                    {{ config.name }}
                  </option>
                </select>
                <input
                  v-else
                  v-model="args"
                  type="text"
                  :disabled="!canEditArgs"
                  :placeholder="selectedCommandInfo?.requiresArgs ? t('commands.requiredArgsPlaceholder') : t('commands.argsPlaceholder')"
                  @keydown.enter="canExecuteSelected && handleExecute()"
                >
              </label>

              <label
                v-if="selectedCommandInfo?.dangerous"
                class="commands-danger-confirm"
              >
                <input
                  v-model="dangerAccepted"
                  type="checkbox"
                  :disabled="runtimeUnavailable || isRunning"
                >
                <span>
                  <strong>{{ t('commands.dangerConfirmTitle') }}</strong>
                  {{ t('commands.dangerConfirmDescription') }}
                </span>
              </label>
            </div>
          </Card>

          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            class="commands-panel commands-ledger"
          >
            <div class="commands-panel__header commands-panel__header--wide">
              <div>
                <p class="commands-panel__eyebrow">
                  {{ t('commands.ledgerEyebrow') }}
                </p>
                <h2 class="commands-panel__title">
                  {{ t('commands.output') }}
                </h2>
                <p class="commands-panel__subtitle">
                  {{ ledgerSubtitle }}
                </p>
              </div>
              <div class="commands-panel__actions">
                <Button
                  variant="ghost"
                  density="compact"
                  surface="status"
                  motion="subtle"
                  :disabled="!hasLedgerOutput"
                  @click="handleCopyOutput"
                >
                  {{ t('commands.copy') }}
                </Button>
                <Button
                  variant="ghost"
                  density="compact"
                  surface="status"
                  motion="subtle"
                  :disabled="!currentSnapshot"
                  @click="handleClearOutput"
                >
                  {{ t('commands.clear') }}
                </Button>
              </div>
            </div>

            <div
              v-if="currentSnapshot"
              class="commands-ledger__meta"
            >
              <span>{{ t('commands.jobStatus') }} <strong :class="statusClass(currentSnapshot.status)">{{ statusLabel(currentSnapshot.status) }}</strong></span>
              <span>{{ t('commands.duration') }} <strong>{{ formatDuration(currentSnapshot.duration_ms) }}</strong></span>
              <span>{{ t('commands.exitCode') }} <strong>{{ currentSnapshot.exit_code ?? '—' }}</strong></span>
              <span>{{ t('commands.linesCount', { count: outputLineCount }) }}</span>
            </div>

            <div
              v-if="isRunning"
              class="commands-output commands-output--running"
            >
              <SIcon
                name="Loader2"
                size="w-5 h-5"
                class="animate-spin text-accent-secondary"
              />
              <span>{{ t('commands.processing') }}</span>
            </div>

            <div
              v-if="hasLedgerOutput"
              class="commands-output"
            >
              <div
                v-for="line in ledgerLines"
                :key="`${line.channel}-${line.index}-${line.text}`"
                class="commands-output__line"
                :class="`commands-output__line--${line.channel}`"
              >
                <span>{{ line.channel }}</span>
                <code>{{ line.text }}</code>
              </div>
            </div>

            <AsyncStatePanel
              v-else-if="!isRunning"
              state="empty"
              :title="t('commands.readyTitle')"
              :description="t('commands.readyDescription')"
              compact
            />
          </Card>
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { cancelCcrCommandJob, listCommands, listConfigs, startCcrCommandJob } from '@/api'
import type { CommandInfo, CommandJobSnapshot, CommandJobStatus, ConfigItem } from '@/types'
import { normalizeCliClient, type CliClient } from '@/types/router'
import { logger } from '@/utils/logger'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { isTauriRuntime } from '@/utils/tauriRuntime'

interface CommandClient {
  id: CliClient
  name: string
  icon: string
  executable: boolean
}

interface CommandUiInfo extends CommandInfo {
  category: string
  dangerous: boolean
  readOnly: boolean
  requiresArgs: boolean
  executable: boolean
}

type CommandBadge = 'safe' | 'danger' | 'readonly' | 'args' | 'blocked'
type LedgerChannel = 'stdout' | 'stderr' | 'system'

const { t } = useI18n({ useScope: 'global' })
const route = useRoute()
const router = useRouter()

const runtimeUnavailable = computed(() => !isTauriRuntime())
const runtimeCopy = computed(() => getRuntimeUnavailableCopy('commands'))

const CLI_CLIENTS: CommandClient[] = [
  { id: 'ccr', name: 'CCR', icon: 'Zap', executable: true },
  { id: 'claude', name: 'Claude Code', icon: 'Code2', executable: false },
  { id: 'gemini', name: 'Gemini', icon: 'Gem', executable: false },
]

const dangerousCommands = new Set(['delete', 'import', 'restore'])
const writeCommands = new Set(['switch', 'add', 'delete', 'rename', 'duplicate', 'import', 'backup', 'restore'])
const argsCommands = new Set(['switch', 'add', 'delete', 'rename', 'duplicate', 'show', 'export', 'import', 'restore', 'diff'])
const allowedCommands = new Set([
  'list',
  'switch',
  'add',
  'delete',
  'rename',
  'duplicate',
  'show',
  'validate',
  'export',
  'import',
  'history',
  'version',
  'help',
  'backup',
  'restore',
  'diff',
  'status',
])

const selectedClient = ref<CliClient>('ccr')
const commands = ref<CommandUiInfo[]>([])
const selectedCommand = ref('')
const args = ref('')
const searchQuery = ref('')
const activeCategory = ref('all')
const dangerAccepted = ref(false)
const currentSnapshot = ref<CommandJobSnapshot | null>(null)
const configs = ref<ConfigItem[]>([])
const unlisteners: UnlistenFn[] = []

const fallbackCommandRegistry: Record<CliClient, CommandInfo[]> = {
  ccr: [
    { name: 'status', description: 'Inspect current CCR status.', usage: 'ccr status', examples: ['ccr status'], category: 'read' },
    { name: 'switch', description: 'Switch to a saved CCR configuration.', usage: 'ccr switch <name>', examples: ['ccr switch default'], category: 'write' },
    { name: 'version', description: 'Inspect the installed CCR version.', usage: 'ccr version', examples: ['ccr version'], category: 'read' },
  ],
  claude: [
    { name: 'help', description: 'Preview only. Claude Code execution is not wired to the CCR whitelist.', usage: 'claude --help', examples: ['claude --help'], category: 'blocked' },
    { name: 'version', description: 'Preview only. Claude Code execution is not wired to the CCR whitelist.', usage: 'claude --version', examples: ['claude --version'], category: 'blocked' },
  ],
  gemini: [
    { name: 'help', description: 'Preview only. Gemini execution is not wired to the CCR whitelist.', usage: 'gemini --help', examples: ['gemini --help'], category: 'blocked' },
    { name: 'version', description: 'Preview only. Gemini execution is not wired to the CCR whitelist.', usage: 'gemini --version', examples: ['gemini --version'], category: 'blocked' },
  ],
}

const selectedClientInfo = computed(() => CLI_CLIENTS.find((client) => client.id === selectedClient.value) ?? CLI_CLIENTS[0])
const selectedClientLabel = computed(() => selectedClientInfo.value.name)
const commandBinary = computed(() => selectedClient.value)
const selectedCommandInfo = computed(() => commands.value.find((command) => command.name === selectedCommand.value))
const executableCommandCount = computed(() => commands.value.filter((command) => command.executable).length)
const isRunning = computed(() => currentSnapshot.value?.status === 'queued' || currentSnapshot.value?.status === 'running')
const canRun = computed(() => !runtimeUnavailable.value && selectedClient.value === 'ccr')
const readinessLabel = computed(() => {
  if (runtimeUnavailable.value) return t('commands.runtimeWeb')
  if (selectedClient.value !== 'ccr') return t('commands.runtimeClientPreview')
  if (isRunning.value) return t('commands.runtimeRunning')
  return t('commands.runtimeReady')
})
const canEditArgs = computed(() => canRun.value && Boolean(selectedCommandInfo.value?.executable) && !isRunning.value)
const canExecuteSelected = computed(() => {
  const command = selectedCommandInfo.value
  if (!canEditArgs.value || !command) return false
  if (command.dangerous && !dangerAccepted.value) return false
  if (command.requiresArgs && args.value.trim().length === 0) return false
  return true
})

const categoryTabs = computed(() => {
  const categories = Array.from(new Set(commands.value.map((command) => command.category || 'other')))
  return ['all', ...categories]
})

const filteredCommands = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return commands.value.filter((command) => {
    const matchesCategory = activeCategory.value === 'all' || command.category === activeCategory.value
    const matchesQuery = !query
      || command.name.toLowerCase().includes(query)
      || command.description.toLowerCase().includes(query)
    return matchesCategory && matchesQuery
  })
})

const outputLineCount = computed(() => {
  const snapshot = currentSnapshot.value
  if (!snapshot) return 0
  return snapshot.stdout_lines.length + snapshot.stderr_lines.length + snapshot.system_lines.length
})

const hasLedgerOutput = computed(() => outputLineCount.value > 0)

const ledgerLines = computed(() => {
  const snapshot = currentSnapshot.value
  if (!snapshot) return []
  const build = (channel: LedgerChannel, lines: string[]) => lines.map((text, index) => ({ channel, text, index }))
  return [
    ...build('system', snapshot.system_lines),
    ...build('stdout', snapshot.stdout_lines),
    ...build('stderr', snapshot.stderr_lines),
  ]
})

const ledgerSubtitle = computed(() => {
  const snapshot = currentSnapshot.value
  if (!snapshot) return t('commands.ledgerSubtitleIdle')
  return t('commands.ledgerSubtitleActive', {
    job: snapshot.job_id.slice(0, 18),
    command: `ccr ${snapshot.command}`,
  })
})

const statusLabel = (status: CommandJobStatus) => t(`commands.status.${status}`)

const readinessCards = computed(() => [
  {
    key: 'runtime',
    icon: runtimeUnavailable.value ? 'MonitorOff' : 'CheckCircle2',
    tone: runtimeUnavailable.value ? 'warning' : 'success',
    label: t('commands.cardRuntimeLabel'),
    value: runtimeUnavailable.value ? t('commands.runtimeWeb') : t('commands.runtimeDesktop'),
    detail: runtimeUnavailable.value ? t('commands.cardRuntimeWebDetail') : t('commands.cardRuntimeDesktopDetail'),
  },
  {
    key: 'job',
    icon: isRunning.value ? 'Loader2' : 'Clock',
    tone: isRunning.value ? 'info' : 'neutral',
    label: t('commands.cardJobLabel'),
    value: currentSnapshot.value ? statusLabel(currentSnapshot.value.status) : t('commands.cardJobIdle'),
    detail: currentSnapshot.value ? `ccr ${currentSnapshot.value.command}` : t('commands.cardJobIdleDetail'),
  },
  {
    key: 'trust',
    icon: 'ShieldCheck',
    tone: 'success',
    label: t('commands.cardTrustLabel'),
    value: t('commands.cardTrustValue'),
    detail: t('commands.cardTrustDetail', { count: executableCommandCount.value }),
  },
])

const normalizeCommand = (command: CommandInfo, client: CliClient): CommandUiInfo => {
  const name = command.name
  const executable = client === 'ccr' && allowedCommands.has(name)
  const dangerous = dangerousCommands.has(name)
  const readOnly = !writeCommands.has(name)
  const category = command.category || (dangerous ? 'danger' : readOnly ? 'read' : 'write')
  const clientLabel = CLI_CLIENTS.find((item) => item.id === client)?.name ?? client
  const description = client === 'ccr'
    ? t(`commands.catalog.${name}`)
    : t('commands.clientPreviewCommandDescription', { client: clientLabel })
  return {
    ...command,
    description,
    usage: command.usage || `ccr ${name}`,
    examples: command.examples || [`ccr ${name}`],
    category,
    dangerous,
    readOnly,
    requiresArgs: argsCommands.has(name),
    executable,
  }
}

const applyCommandList = (client: CliClient, list = fallbackCommandRegistry[client]) => {
  commands.value = list.map((command) => normalizeCommand(command, client))
  if (!selectedCommand.value || !commands.value.some((command) => command.name === selectedCommand.value)) {
    selectedCommand.value = commands.value[0]?.name ?? ''
  }
  if (!categoryTabs.value.includes(activeCategory.value)) {
    activeCategory.value = 'all'
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
    const response = await listConfigs<{ configs: ConfigItem[] } | ConfigItem[]>()
    configs.value = Array.isArray(response) ? response : response.configs
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
    applyCommandList('ccr', data.length > 0 ? data : fallbackCommandRegistry.ccr)
  } catch (error) {
    logger.error('Failed to load commands:', error)
    applyCommandList(selectedClient.value)
  }
}

const installJobListeners = async () => {
  if (runtimeUnavailable.value) return
  const handleSnapshot = (event: Event<CommandJobSnapshot>) => {
    if (!currentSnapshot.value || event.payload.job_id === currentSnapshot.value.job_id) {
      currentSnapshot.value = event.payload
    }
  }

  unlisteners.push(await listen<CommandJobSnapshot>('commands:job-progress', handleSnapshot))
  unlisteners.push(await listen<CommandJobSnapshot>('commands:job-finished', handleSnapshot))
  unlisteners.push(await listen<CommandJobSnapshot>('commands:job-cancelled', handleSnapshot))
}

onMounted(() => {
  const initialClient = normalizeCliClient(route.params.client)
  if (initialClient) {
    selectedClient.value = initialClient
  }
  void loadCommands()
  void loadConfigs()
  void installJobListeners()
})

onUnmounted(() => {
  for (const unlisten of unlisteners.splice(0)) {
    unlisten()
  }
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
  dangerAccepted.value = false
  currentSnapshot.value = null
  void loadCommands()

  const current = normalizeCliClient(route.params.client) || 'ccr'
  if (current !== selectedClient.value) {
    void router.replace({ name: 'commands', params: { client: selectedClient.value } })
  }
})

watch(selectedCommand, () => {
  args.value = ''
  dangerAccepted.value = false
})

const setSelectedClient = (client: CliClient) => {
  selectedClient.value = client
}

const setSelectedCommand = (command: string) => {
  selectedCommand.value = command
}

const splitArgs = (value: string): string[] => value
  .split(' ')
  .map((arg) => arg.trim())
  .filter((arg) => arg.length > 0)

const handleExecute = async () => {
  if (!canExecuteSelected.value || !selectedCommandInfo.value) return

  try {
    const response = await startCcrCommandJob({
      command: selectedCommandInfo.value.name,
      args: splitArgs(args.value),
    })
    currentSnapshot.value = response.snapshot
  } catch (error) {
    const message = error instanceof Error ? error.message : t('commands.unknownError')
    currentSnapshot.value = {
      job_id: 'local-error',
      command: selectedCommandInfo.value.name,
      args: splitArgs(args.value),
      status: 'failed',
      started_at: new Date().toISOString(),
      finished_at: new Date().toISOString(),
      duration_ms: 0,
      exit_code: -1,
      stdout_lines: [],
      stderr_lines: [],
      system_lines: [message],
      error: message,
    }
  }
}

const handleCancel = async () => {
  if (!currentSnapshot.value) return
  try {
    currentSnapshot.value = await cancelCcrCommandJob(currentSnapshot.value.job_id)
  } catch (error) {
    logger.error('Failed to cancel command job:', error)
  }
}

const handleCopyOutput = async () => {
  if (!currentSnapshot.value) return
  const text = ledgerLines.value.map((line) => `[${line.channel}] ${line.text}`).join('\n')
  try {
    await navigator.clipboard.writeText(text)
  } catch (error) {
    logger.error('Failed to copy:', error)
  }
}

const handleClearOutput = () => {
  currentSnapshot.value = null
}

const categoryLabel = (category: string) => {
  const labels: Record<string, string> = {
    all: t('commands.categoryAll'),
    read: t('commands.categoryRead'),
    write: t('commands.categoryWrite'),
    danger: t('commands.categoryDanger'),
    blocked: t('commands.categoryBlocked'),
    other: t('commands.categoryOther'),
  }
  return labels[category] || category
}

const commandBadges = (command: CommandUiInfo): CommandBadge[] => {
  const badges: CommandBadge[] = []
  if (!command.executable) badges.push('blocked')
  if (command.dangerous) badges.push('danger')
  if (command.readOnly) badges.push('readonly')
  if (command.requiresArgs) badges.push('args')
  if (badges.length === 0) badges.push('safe')
  return badges
}

const badgeLabel = (badge: CommandBadge) => {
  const labels: Record<CommandBadge, string> = {
    safe: t('commands.badgeSafe'),
    danger: t('commands.badgeDanger'),
    readonly: t('commands.badgeReadOnly'),
    args: t('commands.badgeArgs'),
    blocked: t('commands.badgeBlocked'),
  }
  return labels[badge]
}

const statusClass = (status: CommandJobStatus) => `commands-status--${status}`
const formatDuration = (duration?: number | null) => duration == null ? '—' : `${duration}ms`
</script>

<style scoped>
.commands-page {
  @apply px-4 py-4 sm:px-6 sm:py-6;
}

.commands-shell {
  @apply mx-auto flex max-w-[1480px] flex-col gap-5;
}

.commands-header-meta,
.commands-panel__actions,
.commands-composer__actions {
  @apply flex flex-wrap items-center gap-2;
}

.commands-chip {
  @apply inline-flex items-center gap-1.5 rounded-full border border-border-default/55 px-3 py-1 text-xs font-medium text-text-secondary;

  background-color: rgb(var(--color-bg-elevated-rgb) / 72%);
}

.commands-chip--success {
  @apply border-accent-success/25 bg-accent-success/10 text-accent-success;
}

.commands-chip--warning {
  @apply border-accent-warning/25 bg-accent-warning/10 text-accent-warning;
}

.commands-status-grid {
  @apply grid gap-3 md:grid-cols-3;
}

.commands-status-card {
  @apply flex items-start gap-3 p-4;
}

.commands-status-card__icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl border;
}

.commands-status-card__icon--success {
  @apply border-accent-success/20 bg-accent-success/10 text-accent-success;
}

.commands-status-card__icon--warning {
  @apply border-accent-warning/20 bg-accent-warning/10 text-accent-warning;
}

.commands-status-card__icon--info {
  @apply border-accent-info/20 bg-accent-info/10 text-accent-info;
}

.commands-status-card__icon--neutral {
  @apply border-border-default/50 bg-bg-surface text-text-secondary;
}

.commands-status-card__label,
.commands-panel__eyebrow {
  @apply text-[11px] font-semibold uppercase tracking-[0.14em] text-text-muted;
}

.commands-status-card strong {
  @apply mt-1 block text-sm font-semibold text-text-primary;
}

.commands-status-card span {
  @apply mt-1 block text-xs leading-relaxed text-text-secondary;
}

.commands-workbench {
  @apply grid gap-5 xl:grid-cols-[360px_minmax(0,1fr)];
}

.commands-main-grid {
  @apply grid gap-5 2xl:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)];
}

.commands-panel {
  @apply p-5;
}

.commands-panel--palette,
.commands-ledger,
.commands-composer {
  @apply min-h-[620px];
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

.commands-panel__title--large {
  @apply text-2xl;
}

.commands-panel__subtitle {
  @apply mt-1 max-w-2xl text-sm leading-relaxed text-text-secondary;
}

.commands-client-switcher,
.commands-category-tabs {
  @apply mb-4 flex flex-wrap gap-2;
}

.commands-client-pill,
.commands-category-tabs__item {
  @apply inline-flex items-center gap-2 rounded-full border border-border-default/50 px-3 py-2 text-xs font-medium text-text-secondary transition-colors duration-200;

  background-color: rgb(var(--color-bg-elevated-rgb) / 56%);
}

.commands-client-pill:hover,
.commands-category-tabs__item:hover,
.commands-category-tabs__item--active,
.commands-client-pill--active {
  @apply border-accent-primary/25 bg-accent-primary/10 text-text-primary;
}

.commands-client-pill--disabled small {
  @apply text-[10px] uppercase tracking-[0.12em] text-text-muted;
}

.commands-search {
  @apply mb-4 flex items-center gap-2 rounded-2xl border border-border-default/50 px-3 py-2 text-text-secondary;

  background-color: rgb(var(--color-bg-base-rgb) / 70%);
}

.commands-search input {
  @apply min-h-[34px] flex-1 bg-transparent text-sm text-text-primary outline-none placeholder:text-text-muted;
}

.commands-list {
  @apply flex max-h-[440px] flex-col gap-2 overflow-y-auto pr-1;
}

.command-row {
  @apply flex w-full flex-col gap-2 rounded-2xl border border-transparent px-3 py-3 text-left transition-colors duration-200;
}

.command-row:hover,
.command-row--active {
  background-color: rgb(var(--color-bg-surface-rgb) / 72%);
  border-color: rgb(var(--color-border-default-rgb) / 70%);
}

.command-row--active {
  @apply border-accent-primary/25 bg-accent-primary/10;
}

.command-row--disabled {
  @apply opacity-70;
}

.command-row__topline {
  @apply flex flex-wrap items-center gap-2;
}

.command-row__topline strong {
  @apply font-mono text-sm font-semibold text-text-primary;
}

.command-row p {
  @apply text-xs leading-relaxed text-text-secondary;
}

.command-badge {
  @apply rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.11em];
}

.command-badge--safe,
.command-badge--readonly {
  @apply border-accent-success/20 bg-accent-success/10 text-accent-success;
}

.command-badge--danger {
  @apply border-accent-danger/20 bg-accent-danger/10 text-accent-danger;
}

.command-badge--args {
  @apply border-accent-info/20 bg-accent-info/10 text-accent-info;
}

.command-badge--blocked {
  @apply border-accent-warning/20 bg-accent-warning/10 text-accent-warning;
}

.commands-runtime-panel,
.commands-notice,
.command-preview,
.commands-form-grid {
  @apply mt-4;
}

.commands-notice {
  @apply flex gap-3 rounded-2xl border border-accent-warning/25 bg-accent-warning/10 p-4 text-sm text-accent-warning;
}

.commands-notice p {
  @apply mt-1 text-xs leading-relaxed text-text-secondary;
}

.command-preview {
  @apply rounded-2xl border border-border-default/55 p-4;

  background-color: rgb(var(--color-bg-base-rgb) / 86%);
}

.command-preview__label {
  @apply mb-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-text-muted;
}

.command-preview__body {
  @apply flex flex-wrap items-center gap-2 font-mono text-sm;
}

.command-preview__prompt {
  @apply font-semibold text-accent-success;
}

.command-preview__binary,
.command-preview__command {
  @apply font-semibold text-text-primary;
}

.command-preview__args {
  @apply text-text-secondary;
}

.commands-form-grid {
  @apply grid gap-4;
}

.commands-field {
  @apply grid gap-2 text-sm font-medium text-text-secondary;
}

.commands-field input,
.commands-field select {
  @apply min-h-[46px] rounded-2xl border border-border-default/60 px-3 py-2 text-sm text-text-primary outline-none transition-colors duration-200 placeholder:text-text-muted;

  background-color: rgb(var(--color-bg-elevated-rgb) / 62%);
}

.commands-field input:focus,
.commands-field select:focus {
  @apply border-accent-secondary/35;
}

.commands-field input:disabled,
.commands-field select:disabled {
  @apply cursor-not-allowed opacity-60;
}

.commands-danger-confirm {
  @apply flex items-start gap-3 rounded-2xl border border-accent-danger/25 bg-accent-danger/10 p-4 text-sm text-text-secondary;
}

.commands-danger-confirm input {
  @apply mt-1;
}

.commands-danger-confirm strong {
  @apply mr-1 text-accent-danger;
}

.commands-ledger__meta {
  @apply mb-4 flex flex-wrap gap-3 rounded-2xl border border-border-default/45 px-3 py-2 text-xs text-text-secondary;

  background-color: rgb(var(--color-bg-elevated-rgb) / 46%);
}

.commands-ledger__meta strong {
  @apply font-semibold text-text-primary;
}

.commands-status--success {
  @apply text-accent-success;
}

.commands-status--failed,
.commands-status--cancelled,
.commands-status--unavailable {
  @apply text-accent-danger;
}

.commands-status--queued,
.commands-status--running {
  @apply text-accent-info;
}

.commands-output {
  @apply flex max-h-[520px] min-h-[320px] flex-col gap-1 overflow-y-auto rounded-2xl border border-border-default/50 p-4 font-mono text-xs;

  background-color: rgb(var(--color-bg-base-rgb) / 74%);
}

.commands-output--running {
  @apply mb-3 min-h-0 flex-row items-center gap-2 py-3 font-sans text-sm text-text-secondary;
}

.commands-output__line {
  @apply grid grid-cols-[64px_minmax(0,1fr)] gap-3 rounded-xl px-2 py-1.5;
}

.commands-output__line span {
  @apply text-[10px] font-semibold uppercase tracking-[0.12em] text-text-muted;
}

.commands-output__line code {
  @apply whitespace-pre-wrap break-words text-text-primary;
}

.commands-output__line--stderr code {
  @apply text-accent-danger;
}

.commands-output__line--system code {
  @apply text-text-secondary;
}
</style>
