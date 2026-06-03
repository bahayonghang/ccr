<template>
  <div class="commands-page">
    <div class="commands-shell">
      <Card
        surface="workspace"
        :elevation="2"
        motion="subtle"
        class="commands-run-strip"
      >
        <div class="commands-run-strip__identity">
          <p class="commands-panel__eyebrow">
            {{ t('commands.operatorBadge') }}
          </p>
          <div>
            <h1>{{ t('commands.title') }}</h1>
            <p>{{ t('commands.description') }}</p>
          </div>
        </div>

        <div class="commands-run-strip__signals">
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
          <span class="commands-chip">
            <SIcon
              name="Activity"
              size="w-3.5 h-3.5"
            />
            {{ currentSnapshot ? statusLabel(currentSnapshot.status) : t('commands.cardJobIdle') }}
          </span>
        </div>
      </Card>

      <div class="commands-workbench">
        <aside class="commands-palette">
          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            class="commands-panel commands-panel--palette"
            body-class="!h-auto"
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
              <div
                v-if="activeCollection === 'history'"
                class="commands-panel__actions"
              >
                <Button
                  variant="ghost"
                  density="compact"
                  surface="status"
                  motion="subtle"
                  :disabled="historyItems.length === 0 || runtimeUnavailable"
                  @click="handleClearHistory"
                >
                  {{ t('commands.clearHistory') }}
                </Button>
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

            <div class="commands-source-tabs">
              <button
                v-for="collection in collectionTabs"
                :key="collection"
                type="button"
                class="commands-source-tabs__item"
                :class="{ 'commands-source-tabs__item--active': activeCollection === collection }"
                @click="activeCollection = collection"
              >
                {{ collectionLabel(collection) }}
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

            <div
              v-if="activeCollection === 'catalog'"
              class="commands-category-tabs"
            >
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

            <div
              v-if="activeCollection === 'catalog'"
              class="commands-list"
            >
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

            <div
              v-else-if="activeCollection === 'favorites'"
              class="commands-list"
            >
              <button
                v-for="favorite in filteredFavorites"
                :key="favorite.id"
                type="button"
                class="command-row"
                :class="{
                  'command-row--active': selectedCommand === resolvedCommandName(favorite.command),
                  'command-row--disabled': !canLoadPersistedCommand(favorite.command),
                }"
                @click="loadFavorite(favorite)"
              >
                <div class="command-row__topline">
                  <strong>{{ favorite.display_name || favorite.command }}</strong>
                  <span
                    v-if="!canLoadPersistedCommand(favorite.command)"
                    class="command-badge command-badge--blocked"
                  >
                    {{ t('commands.stale') }}
                  </span>
                </div>
                <p>{{ persistedCommandSummary(favorite.command, favorite.args) }}</p>
              </button>

              <div
                v-if="filteredFavorites.length === 0"
                class="commands-list-empty"
              >
                {{ t('commands.noFavorites') }}
              </div>
            </div>

            <div
              v-else
              class="commands-list"
            >
              <button
                v-for="item in filteredHistory"
                :key="item.id"
                type="button"
                class="command-row"
                :class="{
                  'command-row--active': selectedCommand === resolvedCommandName(item.command),
                  'command-row--disabled': !canLoadPersistedCommand(item.command),
                }"
                @click="loadHistoryItem(item)"
              >
                <div class="command-row__topline">
                  <strong>{{ item.full_command || `ccr ${item.command}` }}</strong>
                  <span
                    class="command-badge"
                    :class="item.success ? 'command-badge--readonly' : 'command-badge--danger'"
                  >
                    {{ item.success ? t('commands.historySuccess') : t('commands.historyFailed') }}
                  </span>
                  <span
                    v-if="!canLoadPersistedCommand(item.command)"
                    class="command-badge command-badge--blocked"
                  >
                    {{ t('commands.stale') }}
                  </span>
                </div>
                <p>{{ persistedCommandSummary(item.command, item.args) }} · {{ formatDuration(item.duration_ms) }}</p>
              </button>

              <div
                v-if="filteredHistory.length === 0"
                class="commands-list-empty"
              >
                {{ t('commands.noHistory') }}
              </div>
            </div>
          </Card>
        </aside>

        <section class="commands-workbench__main">
          <Card
            surface="card"
            :elevation="3"
            motion="standard"
            class="commands-panel commands-composer"
            body-class="!h-auto"
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
                  variant="ghost"
                  density="compact"
                  surface="card"
                  motion="subtle"
                  :disabled="!canFavoriteSelected"
                  @click="handleToggleFavorite"
                >
                  <template #leading>
                    <SIcon
                      :name="isSelectedFavorite ? 'StarOff' : 'Star'"
                      size="w-4 h-4"
                    />
                  </template>
                  {{ isSelectedFavorite ? t('commands.removeFavorite') : t('commands.addFavorite') }}
                </Button>
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

            <div
              v-if="runtimeUnavailable"
              class="commands-notice commands-notice--neutral commands-runtime-panel"
              role="status"
              aria-live="polite"
            >
              <SIcon
                name="MonitorOff"
                size="w-5 h-5"
              />
              <div>
                <strong>{{ runtimeCopy.title }}</strong>
                <p>{{ t('commands.webUnavailableDetail') }}</p>
              </div>
            </div>

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
                <p>
                  {{ t('commands.clientUnavailableDescription', { client: selectedClientLabel }) }}
                </p>
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

            <div class="command-strip">
              <div class="command-strip__label">
                {{ t('commands.previewLabel') }}
              </div>
              <div class="command-strip__body">
                <span class="command-strip__prompt">➜</span>
                <span class="command-strip__binary">{{ commandPreview }}</span>
                <span
                  v-if="args.trim()"
                  class="command-strip__args"
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
                  :placeholder="
                    selectedCommandInfo?.requiresArgs
                      ? t('commands.requiredArgsPlaceholder')
                      : t('commands.argsPlaceholder')
                  "
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
            body-class="!h-auto"
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
              class="commands-ledger__metrics"
            >
              <div class="commands-ledger__metric">
                <span>{{ t('commands.jobStatus') }}</span>
                <strong :class="statusClass(currentSnapshot.status)">{{
                  statusLabel(currentSnapshot.status)
                }}</strong>
              </div>
              <div class="commands-ledger__metric">
                <span>{{ t('commands.duration') }}</span>
                <strong>{{ formatDuration(currentSnapshot.duration_ms) }}</strong>
              </div>
              <div class="commands-ledger__metric">
                <span>{{ t('commands.exitCode') }}</span>
                <strong>{{ currentSnapshot.exit_code ?? '—' }}</strong>
              </div>
              <div class="commands-ledger__metric">
                <span>{{ t('commands.terminalOutput') }}</span>
                <strong>{{ t('commands.linesCount', { count: outputLineCount }) }}</strong>
              </div>
            </div>

            <div
              v-if="isRunning"
              class="commands-ledger__status-strip"
              role="status"
              aria-live="polite"
            >
              <span class="commands-ledger__pulse" />
              <span>{{ t('commands.processing') }}</span>
            </div>

            <div
              v-if="hasLedgerOutput"
              class="commands-terminal"
            >
              <div
                v-for="line in ledgerLines"
                :key="`${line.channel}-${line.index}-${line.text}`"
                class="commands-terminal__line"
                :class="`commands-terminal__line--${line.channel}`"
              >
                <span class="commands-terminal__channel">{{ line.channel }}</span>
                <code
                  class="commands-terminal__text"
                  v-html="line.safeHtml"
                />
              </div>
            </div>

            <div
              v-else-if="!isRunning"
              class="commands-ledger-empty"
              role="status"
              aria-live="polite"
            >
              <SIcon
                name="FileX"
                size="w-6 h-6"
              />
              <strong>{{ t('commands.readyTitle') }}</strong>
              <p>{{ t('commands.readyDescription') }}</p>
            </div>
          </Card>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { cancelCcrCommandJob, listCommands, listConfigs, startCcrCommandJob } from '@/api'
import {
  addFavorite as addFavoriteItem,
  addRecentItem,
  clearRecentItems,
  getFavorites,
  getRecentItems,
  removeFavorite as removeFavoriteItem,
} from '@/api/domains/uiState'
import type { CommandInfo, CommandJobSnapshot, CommandJobStatus, ConfigItem } from '@/types'
import { normalizeCliClient, type CliClient } from '@/types/router'
import { createAnsiRenderer } from '@/utils/ansiRenderer'
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
type CommandCollection = 'catalog' | 'favorites' | 'history'

interface FavoriteCommand {
  id: string
  command: string
  args: string[]
  display_name?: string | null
  module: string
  created_at: string
}

interface CommandHistoryItem {
  id: string
  full_command: string
  command: string
  args: string[]
  success: boolean
  executed_at: string
  duration_ms: number
}

const { t } = useI18n({ useScope: 'global' })
const route = useRoute()
const router = useRouter()

const runtimeUnavailable = computed(() => !isTauriRuntime())
const runtimeCopy = computed(() => getRuntimeUnavailableCopy('commands'))

const CLI_CLIENTS: CommandClient[] = [
  { id: 'ccr', name: 'CCR', icon: 'Zap', executable: true },
  { id: 'claude', name: 'Claude Code', icon: 'Code2', executable: false },
  { id: 'gemini', name: 'Antigravity CLI', icon: 'Sparkles', executable: false },
]

const selectedClient = ref<CliClient>('ccr')
const commands = ref<CommandUiInfo[]>([])
const selectedCommand = ref('')
const args = ref('')
const searchQuery = ref('')
const activeCategory = ref('all')
const activeCollection = ref<CommandCollection>('catalog')
const dangerAccepted = ref(false)
const currentSnapshot = ref<CommandJobSnapshot | null>(null)
const configs = ref<ConfigItem[]>([])
const favorites = ref<FavoriteCommand[]>([])
const historyItems = ref<CommandHistoryItem[]>([])
const preserveArgsOnNextCommandChange = ref(false)
const unlisteners: UnlistenFn[] = []
const recordedJobIds = new Set<string>()
const ansiRenderer = createAnsiRenderer()

const fallbackCommandRegistry: Record<CliClient, CommandInfo[]> = {
  ccr: [
    {
      name: 'status',
      description: 'Inspect current CCR status.',
      usage: 'ccr status',
      examples: ['ccr status'],
      category: 'read',
      risk: 'safe',
      executable: true,
    },
    {
      name: 'switch',
      description: 'Switch to a saved CCR configuration.',
      usage: 'ccr switch <name>',
      examples: ['ccr switch default'],
      category: 'write',
      risk: 'writes_config',
      executable: true,
      args: [
        {
          name: 'config_name',
          label: 'Configuration',
          type: 'select',
          required: true,
          source: 'configs',
          description: 'Configuration name from the CCR config list.',
        },
      ],
    },
    {
      name: 'version',
      description: 'Inspect the installed CCR version.',
      usage: 'ccr version',
      examples: ['ccr version'],
      category: 'read',
      risk: 'safe',
      executable: true,
    },
  ],
  claude: [
    {
      name: 'help',
      description: 'Preview only. Claude Code execution is not wired to the CCR whitelist.',
      usage: 'claude --help',
      examples: ['claude --help'],
      category: 'blocked',
    },
    {
      name: 'version',
      description: 'Preview only. Claude Code execution is not wired to the CCR whitelist.',
      usage: 'claude --version',
      examples: ['claude --version'],
      category: 'blocked',
    },
  ],
  gemini: [
    {
      name: 'help',
      description: 'Preview only. Antigravity CLI execution is not wired to the CCR whitelist.',
      usage: 'agy --help',
      examples: ['agy --help'],
      category: 'blocked',
    },
    {
      name: 'version',
      description: 'Preview only. Antigravity CLI execution is not wired to the CCR whitelist.',
      usage: 'agy --version',
      examples: ['agy --version'],
      category: 'blocked',
    },
    {
      name: 'plugin-import-gemini',
      description: 'Preview only. Antigravity migration import is not wired to the CCR whitelist.',
      usage: 'agy plugin import gemini',
      examples: ['agy plugin import gemini'],
      category: 'blocked',
    },
  ],
}

const selectedClientInfo = computed(
  () => CLI_CLIENTS.find((client) => client.id === selectedClient.value) ?? CLI_CLIENTS[0]
)
const selectedClientLabel = computed(() => selectedClientInfo.value.name)
const selectedCommandInfo = computed(() =>
  commands.value.find((command) => command.name === selectedCommand.value)
)
const commandPreview = computed(() => {
  const command = selectedCommandInfo.value
  if (!command)
    return selectedClient.value === 'gemini' ? 'agy <command>' : `${selectedClient.value} <command>`

  const usage = command.usage?.trim()
  if (usage) return usage

  const binary = selectedClient.value === 'gemini' ? 'agy' : selectedClient.value
  return `${binary} ${command.name}`
})
const executableCommandCount = computed(
  () => commands.value.filter((command) => command.executable).length
)
const isRunning = computed(
  () => currentSnapshot.value?.status === 'queued' || currentSnapshot.value?.status === 'running'
)
const canRun = computed(() => !runtimeUnavailable.value && selectedClient.value === 'ccr')
const readinessLabel = computed(() => {
  if (runtimeUnavailable.value) return t('commands.runtimeWeb')
  if (selectedClient.value !== 'ccr') return t('commands.runtimeClientPreview')
  if (isRunning.value) return t('commands.runtimeRunning')
  return t('commands.runtimeReady')
})
const collectionTabs: CommandCollection[] = ['catalog', 'favorites', 'history']
const canEditArgs = computed(
  () => canRun.value && Boolean(selectedCommandInfo.value?.executable) && !isRunning.value
)
const canExecuteSelected = computed(() => {
  const command = selectedCommandInfo.value
  if (!canEditArgs.value || !command) return false
  if (command.dangerous && !dangerAccepted.value) return false
  if (command.requiresArgs && args.value.trim().length === 0) return false
  return true
})
const selectedCommandArgs = computed(() => splitArgs(args.value))
const selectedConfirmationToken = computed(() => {
  const command = selectedCommandInfo.value
  if (!command?.dangerous || !dangerAccepted.value) return undefined
  return `desktop-confirm:${command.name}`
})
const selectedFavorite = computed(() =>
  favorites.value.find(
    (item) =>
      item.command === selectedCommand.value &&
      JSON.stringify(item.args) === JSON.stringify(selectedCommandArgs.value)
  ) ?? null
)
const isSelectedFavorite = computed(() => Boolean(selectedFavorite.value))
const canFavoriteSelected = computed(
  () =>
    selectedClient.value === 'ccr' &&
    Boolean(selectedCommandInfo.value) &&
    Boolean(selectedCommandInfo.value?.executable)
)

const categoryTabs = computed(() => {
  const categories = Array.from(
    new Set(commands.value.map((command) => command.category || 'other'))
  )
  return ['all', ...categories]
})

const filteredCommands = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return commands.value.filter((command) => {
    const matchesCategory =
      activeCategory.value === 'all' || command.category === activeCategory.value
    const matchesQuery =
      !query ||
      command.name.toLowerCase().includes(query) ||
      command.description.toLowerCase().includes(query)
    return matchesCategory && matchesQuery
  })
})

const filteredFavorites = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return favorites.value.filter((item) => {
    if (!query) return true
    return (
      item.command.toLowerCase().includes(query) ||
      item.display_name?.toLowerCase().includes(query) ||
      item.args.some((arg) => arg.toLowerCase().includes(query))
    )
  })
})

const filteredHistory = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return historyItems.value.filter((item) => {
    if (!query) return true
    return (
      item.command.toLowerCase().includes(query) ||
      item.full_command.toLowerCase().includes(query) ||
      item.args.some((arg) => arg.toLowerCase().includes(query))
    )
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
  const build = (channel: LedgerChannel, lines: string[]) =>
    lines.map((text, index) => ({
      channel,
      text,
      index,
      safeHtml: ansiRenderer.renderLine(text),
    }))
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

const normalizeCommand = (command: CommandInfo, client: CliClient): CommandUiInfo => {
  const name = command.name
  const risk = command.risk ?? (command.category === 'danger' ? 'destructive' : 'safe')
  const executable = client === 'ccr' ? command.executable ?? true : false
  const dangerous = Boolean(command.requiresConfirmation) || risk === 'destructive'
  const category = command.category || (dangerous ? 'danger' : risk === 'writes_config' ? 'write' : 'read')
  const readOnly = risk === 'safe' || category === 'read' || category === 'diagnostic'
  const requiresArgs = command.args?.some((arg) => arg.required) ?? false
  const clientLabel = CLI_CLIENTS.find((item) => item.id === client)?.name ?? client
  const description =
    client === 'ccr' && command.description
      ? command.description
      : t('commands.clientPreviewCommandDescription', { client: clientLabel })
  return {
    ...command,
    description,
    usage: command.usage || `ccr ${name}`,
    examples: command.examples || [`ccr ${name}`],
    category,
    dangerous,
    readOnly,
    requiresArgs,
    executable,
  }
}

const applyCommandList = (client: CliClient, list = fallbackCommandRegistry[client]) => {
  commands.value = list.map((command) => normalizeCommand(command, client))
  if (
    !selectedCommand.value ||
    !commands.value.some((command) => command.name === selectedCommand.value)
  ) {
    selectedCommand.value = commands.value[0]?.name ?? ''
  }
  if (!categoryTabs.value.includes(activeCategory.value)) {
    activeCategory.value = 'all'
  }
}

const loadConfigs = async () => {
  if (runtimeUnavailable.value) {
    configs.value = [{ name: 'default' } as ConfigItem, { name: 'workspace' } as ConfigItem]
    return
  }

  try {
    const response = await listConfigs<{ configs: ConfigItem[] } | ConfigItem[]>()
    configs.value = Array.isArray(response) ? response : response.configs
  } catch (error) {
    logger.error('Failed to load configs:', error)
  }
}

const loadPersistedState = async () => {
  if (runtimeUnavailable.value) {
    favorites.value = []
    historyItems.value = []
    return
  }

  try {
    const [favoriteData, historyData] = await Promise.all([
      getFavorites<FavoriteCommand[]>(),
      getRecentItems<CommandHistoryItem[]>(20),
    ])
    favorites.value = favoriteData
    historyItems.value = historyData
  } catch (error) {
    logger.error('Failed to load command favorites/history:', error)
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
    void maybeRecordHistory(event.payload)
  }

  unlisteners.push(await listen<CommandJobSnapshot>('commands:job-progress', handleSnapshot))
  unlisteners.push(await listen<CommandJobSnapshot>('commands:job-finished', handleSnapshot))
  unlisteners.push(await listen<CommandJobSnapshot>('commands:job-cancelled', handleSnapshot))
}

const maybeRecordHistory = async (snapshot: CommandJobSnapshot) => {
  if (recordedJobIds.has(snapshot.job_id)) return
  if (!['success', 'failed', 'cancelled'].includes(snapshot.status)) return

  recordedJobIds.add(snapshot.job_id)
  try {
    await addRecentItem<CommandHistoryItem>(
      snapshot.command,
      snapshot.args,
      snapshot.status === 'success',
      snapshot.duration_ms ?? 0
    )
    historyItems.value = await getRecentItems<CommandHistoryItem[]>(20)
  } catch (error) {
    logger.error('Failed to persist command history:', error)
  }
}

onMounted(() => {
  const initialClient = normalizeCliClient(route.params.client)
  if (initialClient) {
    selectedClient.value = initialClient
  }
  void loadCommands()
  void loadConfigs()
  void loadPersistedState()
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
  }
)

watch(selectedClient, () => {
  selectedCommand.value = ''
  args.value = ''
  dangerAccepted.value = false
  currentSnapshot.value = null
  activeCollection.value = 'catalog'
  void loadCommands()

  const current = normalizeCliClient(route.params.client) || 'ccr'
  if (current !== selectedClient.value) {
    void router.replace({ name: 'commands', params: { client: selectedClient.value } })
  }
})

watch(selectedCommand, (command, previousCommand) => {
  if (command !== previousCommand && !preserveArgsOnNextCommandChange.value) {
    args.value = ''
  }
  preserveArgsOnNextCommandChange.value = false
  dangerAccepted.value = false
})

const setSelectedClient = (client: CliClient) => {
  selectedClient.value = client
}

const setSelectedCommand = (command: string) => {
  activeCollection.value = 'catalog'
  selectedCommand.value = command
}

const splitArgs = (value: string): string[] =>
  value
    .split(' ')
    .map((arg) => arg.trim())
    .filter((arg) => arg.length > 0)

const resolvedCommandName = (command: string) => command.trim().split(/\s+/)[0] ?? ''

const canLoadPersistedCommand = (command: string) =>
  commands.value.some((item) => item.name === resolvedCommandName(command))

const persistedCommandSummary = (command: string, persistedArgs: string[]) =>
  persistedArgs.length > 0 ? `ccr ${command} ${persistedArgs.join(' ')}` : `ccr ${command}`

const loadPersistedCommand = (command: string, persistedArgs: string[]) => {
  const nextCommand = resolvedCommandName(command)
  if (!canLoadPersistedCommand(command)) return

  if (selectedCommand.value !== nextCommand) {
    preserveArgsOnNextCommandChange.value = true
    selectedCommand.value = nextCommand
  }
  args.value = persistedArgs.join(' ')
  activeCollection.value = 'catalog'
  dangerAccepted.value = false
}

const loadFavorite = (favorite: FavoriteCommand) => {
  loadPersistedCommand(favorite.command, favorite.args)
}

const loadHistoryItem = (item: CommandHistoryItem) => {
  loadPersistedCommand(item.command, item.args)
}

const handleExecute = async () => {
  if (!canExecuteSelected.value || !selectedCommandInfo.value) return

  try {
    const response = await startCcrCommandJob({
      command: selectedCommandInfo.value.name,
      args: splitArgs(args.value),
      confirmationToken: selectedConfirmationToken.value,
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
    await maybeRecordHistory(currentSnapshot.value)
  } catch (error) {
    logger.error('Failed to cancel command job:', error)
  }
}

const handleToggleFavorite = async () => {
  if (!selectedCommandInfo.value) return

  try {
    if (selectedFavorite.value) {
      const favoriteId = selectedFavorite.value.id
      await removeFavoriteItem(favoriteId)
      favorites.value = favorites.value.filter((item) => item.id !== favoriteId)
      return
    }

    const favorite = await addFavoriteItem<FavoriteCommand>(
      selectedCommandInfo.value.name,
      selectedCommandArgs.value,
      selectedCommandInfo.value.title || selectedCommandInfo.value.name,
      'commands'
    )
    favorites.value = [favorite, ...favorites.value]
  } catch (error) {
    logger.error('Failed to toggle favorite:', error)
  }
}

const handleClearHistory = async () => {
  try {
    await clearRecentItems()
    historyItems.value = []
  } catch (error) {
    logger.error('Failed to clear recent history:', error)
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
  ansiRenderer.clear()
  currentSnapshot.value = null
}

const categoryLabel = (category: string) => {
  const labels: Record<string, string> = {
    all: t('commands.categoryAll'),
    read: t('commands.categoryRead'),
    write: t('commands.categoryWrite'),
    danger: t('commands.categoryDanger'),
    diagnostic: t('commands.categoryRead'),
    preview: t('commands.categoryBlocked'),
    blocked: t('commands.categoryBlocked'),
    other: t('commands.categoryOther'),
  }
  return labels[category] || category
}

const collectionLabel = (collection: CommandCollection) => {
  const labels: Record<CommandCollection, string> = {
    catalog: t('commands.catalogTab'),
    favorites: t('commands.favorites'),
    history: t('commands.history'),
  }
  return labels[collection]
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
const formatDuration = (duration?: number | null) => (duration == null ? '—' : `${duration}ms`)
</script>

<style scoped>
.commands-page {
  @apply px-4 py-4 sm:px-6 sm:py-6;
}

.commands-shell {
  @apply mx-auto flex max-w-[1480px] flex-col gap-4;
}

.commands-run-strip {
  @apply flex flex-col gap-3 p-3 md:flex-row md:items-center md:justify-between;
}

.commands-run-strip__identity {
  @apply flex min-w-0 flex-1 items-center gap-3;
}

.commands-run-strip__identity h1 {
  @apply text-lg font-semibold tracking-[-0.02em] text-text-primary;
}

.commands-run-strip__identity p:not(.commands-panel__eyebrow) {
  @apply mt-0.5 max-w-2xl text-xs leading-relaxed text-text-secondary;
}

.commands-run-strip__signals,
.commands-panel__actions,
.commands-composer__actions {
  @apply flex flex-wrap items-center gap-2;
}

.commands-composer__actions {
  @apply ml-auto justify-end;
}

.commands-chip {
  @apply inline-flex items-center gap-1.5 rounded-full border border-border-default/55 px-2.5 py-1 text-[11px] font-medium text-text-secondary;

  background-color: rgb(var(--color-bg-elevated-rgb) / 72%);
}

.commands-chip--success {
  @apply border-accent-success/25 bg-accent-success/10 text-accent-success;
}

.commands-chip--warning {
  @apply border-accent-warning/25 bg-accent-warning/10 text-accent-warning;
}

.commands-panel__eyebrow {
  @apply text-[11px] font-semibold uppercase tracking-[0.14em] text-text-muted;
}

.commands-workbench {
  display: grid;
  grid-template-columns: minmax(260px, 360px) minmax(0, 1fr);
  gap: 1rem;
  align-items: start;
}

.commands-workbench__main {
  @apply grid min-w-0 gap-4;
}

.commands-panel {
  @apply p-3.5;
}

.commands-panel--palette,
.commands-ledger {
  min-height: clamp(420px, calc(100vh - 460px), 560px);
}

.commands-panel--palette {
  min-height: clamp(580px, calc(100vh - 190px), 820px);
}

.commands-composer {
  min-height: auto;
}

.commands-panel__header {
  @apply mb-3 flex items-start justify-between gap-3;
}

.commands-panel__header--wide {
  @apply flex-wrap;
}

.commands-panel__title {
  @apply text-base font-semibold text-text-primary;
}

.commands-panel__title--large {
  @apply text-xl;
}

.commands-panel__subtitle {
  @apply mt-1 max-w-2xl text-xs leading-relaxed text-text-secondary;
}

.commands-client-switcher,
.commands-source-tabs,
.commands-category-tabs {
  @apply mb-4 flex flex-wrap gap-2;
}

.commands-client-pill,
.commands-source-tabs__item,
.commands-category-tabs__item {
  @apply inline-flex items-center gap-2 rounded-full border border-border-default/50 px-3 py-2 text-xs font-medium text-text-secondary transition-colors duration-200;

  background-color: rgb(var(--color-bg-elevated-rgb) / 56%);
}

.commands-client-pill:hover,
.commands-source-tabs__item:hover,
.commands-category-tabs__item:hover,
.commands-source-tabs__item--active,
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
  @apply flex max-h-[470px] flex-col gap-2 overflow-y-auto pr-1;
}

.commands-list-empty {
  @apply rounded-2xl border border-dashed border-border-default/60 px-4 py-6 text-center text-sm text-text-secondary;

  background-color: rgb(var(--color-bg-base-rgb) / 56%);
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
.command-strip,
.commands-form-grid {
  @apply mt-4;
}

.commands-notice {
  @apply flex items-start gap-3 rounded-2xl border border-accent-warning/25 bg-accent-warning/10 p-3 text-sm text-accent-warning;
}

.commands-notice p {
  @apply mt-1 text-xs leading-relaxed text-text-secondary;
}

.commands-notice--neutral {
  @apply border-border-default/55 bg-bg-surface/70 text-text-secondary;
}

.command-strip {
  @apply grid items-center gap-2 rounded-2xl border border-border-default/55 p-3;

  background-color: rgb(var(--color-bg-base-rgb) / 86%);
  grid-template-columns: auto minmax(0, 1fr);
}

.command-strip__label {
  @apply text-[11px] font-semibold uppercase tracking-[0.14em] text-text-muted;
}

.command-strip__body {
  @apply flex min-w-0 items-center gap-2 overflow-x-auto whitespace-nowrap rounded-xl border border-border-default/35 px-3 py-2 font-mono text-sm;

  background-color: rgb(var(--color-bg-elevated-rgb) / 48%);
}

.command-strip__prompt {
  @apply font-semibold text-accent-success;
}

.command-strip__binary,
.command-strip__command {
  @apply font-semibold text-text-primary;
}

.command-strip__args {
  @apply text-text-secondary;
}

.commands-form-grid {
  @apply grid gap-3;
}

.commands-field {
  @apply grid gap-2 text-sm font-medium text-text-secondary;
}

.commands-field input,
.commands-field select {
  @apply min-h-[40px] rounded-2xl border border-border-default/60 px-3 py-2 text-sm text-text-primary outline-none transition-colors duration-200 placeholder:text-text-muted;

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

.commands-ledger__metrics {
  @apply mb-3 grid gap-2 rounded-2xl border border-border-default/45 p-2;

  background-color: rgb(var(--color-bg-elevated-rgb) / 46%);
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.commands-ledger__metric {
  @apply rounded-xl border border-border-default/25 px-3 py-2;

  background-color: rgb(var(--color-bg-base-rgb) / 44%);
}

.commands-ledger__metric span {
  @apply block text-[10px] font-semibold uppercase tracking-[0.12em] text-text-muted;
}

.commands-ledger__metric strong {
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

.commands-ledger__status-strip {
  @apply mb-3 flex items-center gap-2 rounded-2xl border border-accent-info/20 px-3 py-2 text-sm text-text-secondary;

  background-color: rgb(var(--color-info-rgb) / 8%);
}

.commands-ledger__pulse {
  @apply h-2 w-2 rounded-full bg-accent-info;

  animation: ledger-pulse 1.8s ease-in-out infinite;
}

.commands-terminal {
  @apply max-h-[460px] min-h-[270px] overflow-auto rounded-2xl border border-border-default/50 p-3 font-mono text-xs;

  background-color: rgb(var(--color-bg-base-rgb) / 82%);
  scrollbar-gutter: stable both-edges;
}

.commands-terminal__line {
  display: grid;
  grid-template-columns: 4.5rem max-content;
  gap: 0.75rem;
  min-width: max-content;
  border-radius: 0.7rem;
  padding: 0.32rem 0.5rem;
}

.commands-terminal__channel {
  @apply text-[10px] font-semibold uppercase tracking-[0.12em] text-text-muted;
}

.commands-terminal__text {
  @apply whitespace-pre text-text-primary;
}

.commands-terminal__line--stderr .commands-terminal__text {
  @apply text-accent-danger;
}

.commands-terminal__line--system .commands-terminal__text {
  @apply text-text-secondary;
}

.commands-terminal__text :deep(.ansi-black-fg) {
  color: rgb(var(--color-text-muted-rgb));
}

.commands-terminal__text :deep(.ansi-red-fg),
.commands-terminal__text :deep(.ansi-bright-red-fg) {
  color: rgb(var(--color-danger-rgb));
}

.commands-terminal__text :deep(.ansi-green-fg),
.commands-terminal__text :deep(.ansi-bright-green-fg) {
  color: rgb(var(--color-success-rgb));
}

.commands-terminal__text :deep(.ansi-yellow-fg),
.commands-terminal__text :deep(.ansi-bright-yellow-fg) {
  color: rgb(var(--color-warning-rgb));
}

.commands-terminal__text :deep(.ansi-blue-fg),
.commands-terminal__text :deep(.ansi-bright-blue-fg),
.commands-terminal__text :deep(.ansi-cyan-fg),
.commands-terminal__text :deep(.ansi-bright-cyan-fg) {
  color: rgb(var(--color-info-rgb));
}

.commands-terminal__text :deep(.ansi-magenta-fg),
.commands-terminal__text :deep(.ansi-bright-magenta-fg) {
  color: rgb(var(--color-accent-secondary-rgb));
}

.commands-terminal__text :deep(.ansi-white-fg),
.commands-terminal__text :deep(.ansi-bright-white-fg) {
  color: rgb(var(--color-text-primary-rgb));
}

.commands-terminal__text :deep(.ansi-red-bg) {
  background-color: rgb(var(--color-danger-rgb) / 18%);
}

.commands-terminal__text :deep(.ansi-green-bg) {
  background-color: rgb(var(--color-success-rgb) / 18%);
}

.commands-terminal__text :deep(.ansi-yellow-bg) {
  background-color: rgb(var(--color-warning-rgb) / 18%);
}

.commands-terminal__text :deep(.ansi-blue-bg),
.commands-terminal__text :deep(.ansi-cyan-bg) {
  background-color: rgb(var(--color-info-rgb) / 18%);
}

.commands-terminal__text :deep(.ansi-magenta-bg) {
  background-color: rgb(var(--color-accent-secondary-rgb) / 18%);
}

@keyframes ledger-pulse {
  0%,
  100% {
    opacity: 0.45;
    transform: scale(0.86);
  }

  50% {
    opacity: 1;
    transform: scale(1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .commands-ledger__pulse {
    animation: none;
  }
}

.commands-ledger-empty {
  @apply flex min-h-[190px] flex-col items-center justify-center rounded-2xl border border-border-default/50 p-5 text-center text-sm text-text-secondary;

  background-color: rgb(var(--color-bg-base-rgb) / 70%);
}

.commands-ledger-empty svg {
  @apply mb-3 text-text-muted;
}

.commands-ledger-empty strong {
  @apply text-base font-semibold text-text-primary;
}

.commands-ledger-empty p {
  @apply mt-2 max-w-[240px] text-xs leading-relaxed text-text-secondary;
}

@media (width <= 1240px) {
  .commands-panel--palette .commands-panel__subtitle {
    @apply hidden;
  }

  .commands-panel--palette .commands-panel__header,
  .commands-client-switcher,
  .commands-source-tabs,
  .commands-category-tabs,
  .commands-search {
    @apply mb-3;
  }

  .commands-client-pill,
  .commands-source-tabs__item,
  .commands-category-tabs__item {
    @apply px-2.5 py-1.5;
  }

  .commands-client-pill--disabled small {
    @apply hidden;
  }

  .commands-list {
    @apply max-h-[220px];
  }

  .commands-ledger {
    min-height: 320px;
  }

  .commands-ledger__metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width <= 900px) {
  .commands-workbench {
    grid-template-columns: 1fr;
  }

  .commands-ledger {
    grid-column: auto;
  }

  .commands-palette {
    grid-row: auto;
  }

  .commands-panel--palette,
  .commands-ledger {
    min-height: auto;
  }

  .commands-ledger__metrics {
    grid-template-columns: 1fr;
  }
}
</style>
