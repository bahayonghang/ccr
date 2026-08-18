<template>
  <PageShell class="converter-view">
    <template #header>
      <PageHeader
        :title="$t('converter.title')"
        :description="$t('converter.description')"
      >
        <template #actions>
          <RouterLink
            to="/"
            class="converter-chip-button"
          >
            <SIcon
              name="Home"
              size="w-4 h-4"
            />
            <span>{{ $t('converter.backToHome') }}</span>
          </RouterLink>
        </template>
      </PageHeader>
    </template>
    <template #subnav>
      <ModuleSubnav module="converter" />
    </template>

    <div
      v-if="error"
      class="converter-alert converter-alert--error"
    >
      <SIcon
        name="AlertCircle"
        size="w-5 h-5"
        class="converter-alert__icon"
      />
      <div>{{ error }}</div>
    </div>

    <div
      v-if="successMessage"
      class="converter-alert converter-alert--success"
    >
      <SIcon
        name="Check"
        size="w-5 h-5"
        class="converter-alert__icon"
      />
      <div>{{ successMessage }}</div>
    </div>

    <div class="converter-selection-grid">
      <div class="converter-card">
        <div class="converter-section-heading">
          <SIcon
            name="FileJson"
            size="w-5 h-5"
            class="converter-section-heading__icon"
          />
          <h2 class="converter-card__title">
            {{ $t('converter.sourceFormat') }}
          </h2>
        </div>
        <p class="converter-section-copy">
          {{ $t('converter.selectSource') }}
        </p>

        <div class="converter-option-list">
          <button
            v-for="type in cliTypes"
            :key="type.value"
            type="button"
            class="converter-option-card"
            :class="{ 'converter-option-card--active': sourceFormat === type.value }"
            @click="sourceFormat = type.value"
          >
            <div class="converter-option-card__header">
              <span class="converter-option-card__title">{{ type.label }}</span>
              <span
                v-if="sourceFormat === type.value"
                class="converter-option-badge"
              >
                {{ $t('converter.selected') }}
              </span>
            </div>
            <p class="converter-option-card__description">
              {{ type.description }}
            </p>
          </button>
        </div>
      </div>

      <div class="converter-card">
        <div class="converter-section-heading">
          <SIcon
            name="FileCode"
            size="w-5 h-5"
            class="converter-section-heading__icon"
          />
          <h2 class="converter-card__title">
            {{ $t('converter.targetFormat') }}
          </h2>
        </div>
        <p class="converter-section-copy">
          {{ $t('converter.selectTarget') }}
        </p>

        <div class="converter-option-list">
          <button
            v-for="type in cliTypes"
            :key="type.value"
            type="button"
            class="converter-option-card"
            :class="{
              'converter-option-card--active': targetFormat === type.value && sourceFormat !== type.value,
              'converter-option-card--disabled': sourceFormat === type.value,
            }"
            :disabled="sourceFormat === type.value"
            @click="targetFormat = type.value"
          >
            <div class="converter-option-card__header">
              <span class="converter-option-card__title">{{ type.label }}</span>
              <span
                v-if="targetFormat === type.value && sourceFormat !== type.value"
                class="converter-option-badge"
              >
                {{ $t('converter.selected') }}
              </span>
            </div>
            <p class="converter-option-card__description">
              {{ type.description }}
            </p>
          </button>
        </div>
      </div>
    </div>

    <div class="converter-card">
      <h2 class="converter-card__title converter-card__title--with-gap">
        {{ $t('converter.convertOptions') }}
      </h2>
      <p class="converter-section-copy">
        {{ $t('converter.convertOptionsDesc') }}
      </p>

      <div class="converter-toggle-list">
        <label class="converter-toggle">
          <input
            v-model="convertMcp"
            type="checkbox"
            class="converter-checkbox"
          >
          <span>{{ $t('converter.mcpServers') }}</span>
        </label>
        <label class="converter-toggle">
          <input
            v-model="convertCommands"
            type="checkbox"
            class="converter-checkbox"
          >
          <span>{{ $t('converter.slashCommands') }}</span>
        </label>
        <label class="converter-toggle">
          <input
            v-model="convertAgents"
            type="checkbox"
            class="converter-checkbox"
          >
          <span>{{ $t('converter.agentsConfig') }}</span>
        </label>
      </div>
    </div>

    <div class="converter-card">
      <div class="converter-toolbar">
        <div>
          <h2 class="converter-card__title converter-card__title--compact">
            {{ $t('converter.configInput') }}
          </h2>
          <p class="converter-section-copy converter-section-copy--compact">
            {{ $t('converter.configInputDesc') }}
          </p>
        </div>
        <div class="converter-toolbar__actions">
          <button
            type="button"
            class="converter-toolbar-button"
            @click="handleLoadExample"
          >
            {{ $t('converter.loadExample') }}
          </button>
          <label>
            <span class="converter-toolbar-button converter-toolbar-button--label">
              <SIcon
                name="Upload"
                size="w-4 h-4"
              />
              {{ $t('converter.uploadFile') }}
            </span>
            <input
              type="file"
              accept=".json,.toml,.yaml,.yml,.txt"
              class="hidden"
              @change="handleFileUpload"
            >
          </label>
        </div>
      </div>

      <textarea
        v-model="configData"
        :placeholder="$t('converter.inputPlaceholder')"
        class="converter-textarea"
      />
      <div class="converter-help-text">
        {{ $t('converter.supportedFormats') }}
      </div>
    </div>

    <div class="converter-action-row">
      <button
        type="button"
        class="converter-primary-action"
        :disabled="isConverting || !configData.trim() || sourceFormat === targetFormat"
        @click="handleConvert"
      >
        <SIcon
          v-if="isConverting"
          name="Loader2"
          size="w-5 h-5"
          class="animate-spin"
        />
        <SIcon
          v-else
          name="ArrowRight"
          size="w-5 h-5"
        />
        {{ isConverting ? $t('converter.converting') : $t('converter.startConvert') }}
      </button>
    </div>

    <div
      v-if="result"
      class="converter-results"
    >
      <div class="converter-card">
        <h2 class="converter-card__title converter-card__title--section">
          {{ $t('converter.conversionStats') }}
        </h2>

        <div class="converter-stats-grid">
          <div class="converter-stat">
            <div class="converter-stat__value">
              {{ result.stats?.mcp_servers || 0 }}
            </div>
            <div class="converter-stat__label">
              {{ $t('converter.mcpServersCount') }}
            </div>
          </div>
          <div class="converter-stat">
            <div class="converter-stat__value">
              {{ result.stats?.slash_commands || 0 }}
            </div>
            <div class="converter-stat__label">
              {{ $t('converter.slashCommandsCount') }}
            </div>
          </div>
          <div class="converter-stat">
            <div class="converter-stat__value">
              {{ result.stats?.agents || 0 }}
            </div>
            <div class="converter-stat__label">
              {{ $t('converter.agentsCount') }}
            </div>
          </div>
          <div class="converter-stat">
            <div class="converter-stat__value">
              {{ result.stats?.profiles || 0 }}
            </div>
            <div class="converter-stat__label">
              {{ $t('converter.profilesCount') }}
            </div>
          </div>
          <div class="converter-stat">
            <div class="converter-stat__value">
              <SIcon
                :name="result.stats?.base_config ? 'Check' : 'X'"
                size="w-6 h-6"
                class="mx-auto"
              />
            </div>
            <div class="converter-stat__label">
              {{ $t('converter.baseConfig') }}
            </div>
          </div>
        </div>

        <div
          v-if="result.warnings && result.warnings.length > 0"
          class="converter-warning-panel"
        >
          <div class="converter-warning-panel__title">
            {{ $t('converter.warnings') }}
          </div>
          <ul class="converter-warning-list">
            <li
              v-for="(warning, index) in result.warnings"
              :key="index"
              class="converter-warning-list__item"
            >
              {{ warning }}
            </li>
          </ul>
        </div>
      </div>

      <div class="converter-card">
        <div class="converter-toolbar">
          <div>
            <h2 class="converter-card__title converter-card__title--compact">
              {{ $t('converter.conversionResult') }}
            </h2>
            <p class="converter-section-copy converter-section-copy--compact">
              {{ $t('converter.resultFormat', { format: result.format?.toUpperCase() || '' }) }}
            </p>
          </div>
          <div class="converter-toolbar__actions">
            <button
              type="button"
              class="converter-toolbar-button converter-toolbar-button--label"
              @click="handleCopyResult"
            >
              <SIcon
                name="Copy"
                size="w-4 h-4"
              />
              {{ $t('converter.copy') }}
            </button>
            <button
              type="button"
              class="converter-toolbar-button converter-toolbar-button--label"
              @click="handleDownloadResult"
            >
              <SIcon
                name="Download"
                size="w-4 h-4"
              />
              {{ $t('converter.download') }}
            </button>
          </div>
        </div>

        <textarea
          :value="result.content"
          readonly
          class="converter-textarea converter-textarea--result"
        />
      </div>
    </div>

    <div class="converter-card">
      <h2 class="converter-card__title converter-card__title--section">
        {{ $t('converter.usageGuide') }}
      </h2>

      <div class="converter-guide">
        <div>
          <h4 class="converter-guide__title">
            {{ $t('converter.usageNotes.supportedPathsTitle') }}
          </h4>
          <ul class="converter-guide__list">
            <li>{{ $t('converter.usageNotes.claudeCodex') }}</li>
            <li>{{ $t('converter.usageNotes.otherFormats') }}</li>
          </ul>
        </div>
        <div>
          <h4 class="converter-guide__title">
            {{ $t('converter.usageNotes.conversionNotesTitle') }}
          </h4>
          <ul class="converter-guide__list">
            <li>{{ $t('converter.usageNotes.note1') }}</li>
            <li>{{ $t('converter.usageNotes.note2') }}</li>
            <li>{{ $t('converter.usageNotes.note3') }}</li>
            <li>{{ $t('converter.usageNotes.note4') }}</li>
          </ul>
        </div>
        <div>
          <h4 class="converter-guide__title">
            {{ $t('converter.usageNotes.importantNotesTitle') }}
          </h4>
          <ul class="converter-guide__list">
            <li>{{ $t('converter.usageNotes.caution1') }}</li>
            <li>{{ $t('converter.usageNotes.caution2') }}</li>
            <li>{{ $t('converter.usageNotes.caution3') }}</li>
          </ul>
        </div>
      </div>
    </div>
  </PageShell>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { convertConfig } from '@/api'
import { copyText } from '@/utils/clipboard'
import type { ConverterRequest, ConverterResponse, CliType } from '@/types'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'

const { t } = useI18n({ useScope: 'global' })

const CLI_DEFINITIONS: { value: CliType; label: string; descriptionKey: string }[] = [
  { value: 'claude-code', label: 'Claude Code', descriptionKey: 'converter.formatDescriptions.claudeCode' },
  { value: 'codex', label: 'Codex', descriptionKey: 'converter.formatDescriptions.codex' },
  { value: 'gemini', label: 'Antigravity CLI', descriptionKey: 'converter.formatDescriptions.gemini' },
]

const cliTypes = computed(() =>
  CLI_DEFINITIONS.map(type => ({
    ...type,
    description: t(type.descriptionKey),
  }))
)

const sourceFormat = ref<CliType>('claude-code')
const targetFormat = ref<CliType>('codex')
const configData = ref('')
const convertMcp = ref(true)
const convertCommands = ref(true)
const convertAgents = ref(true)
const isConverting = ref(false)
const result = ref<ConverterResponse | null>(null)
const error = ref<string | null>(null)
const successMessage = ref<string | null>(null)

// 成功提示自动消失：复用单个定时器，避免散落的裸 setTimeout 在卸载后回写已销毁的 ref
const SUCCESS_MESSAGE_MS = 2000
const LOAD_MESSAGE_MS = 3000
let successMessageTimer: ReturnType<typeof setTimeout> | null = null
const flashSuccess = (message: string, duration = SUCCESS_MESSAGE_MS) => {
  successMessage.value = message
  if (successMessageTimer) clearTimeout(successMessageTimer)
  successMessageTimer = setTimeout(() => {
    successMessage.value = null
  }, duration)
}
onBeforeUnmount(() => {
  if (successMessageTimer) clearTimeout(successMessageTimer)
})

const handleFileUpload = (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) {
    const reader = new FileReader()
    reader.onload = (e) => {
      const content = e.target?.result as string
      configData.value = content
      flashSuccess(t('converter.fileLoaded', { name: file.name }), LOAD_MESSAGE_MS)
    }
    reader.onerror = () => {
      error.value = t('converter.fileLoadFailed')
    }
    reader.readAsText(file)
  }
}

const handleConvert = async () => {
  error.value = null
  successMessage.value = null
  result.value = null

  if (!configData.value.trim()) {
    error.value = t('converter.inputRequired')
    return
  }

  if (sourceFormat.value === targetFormat.value) {
    error.value = t('converter.sameFormatError')
    return
  }

  isConverting.value = true

  try {
    const request: ConverterRequest = {
      source_format: sourceFormat.value,
      target_format: targetFormat.value,
      config_data: configData.value,
      convert_mcp: convertMcp.value,
      convert_commands: convertCommands.value,
      convert_agents: convertAgents.value
    }

    const response = await convertConfig(request)
    result.value = response
    successMessage.value = t('converter.convertSuccess')
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('converter.convertError')
  } finally {
    isConverting.value = false
  }
}

const handleCopyResult = () => {
  if (result.value?.content) {
    void copyText(result.value.content)
    flashSuccess(t('converter.copied'))
  }
}

const handleDownloadResult = () => {
  if (result.value?.content) {
    const blob = new Blob([result.value.content], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url

    const extension =
      result.value.format === 'json' ? 'json' : result.value.format === 'toml' ? 'toml' : 'txt'
    const getCliLabel = (value: CliType) =>
      CLI_DEFINITIONS.find((type) => type.value === value)?.label || value
    const sourceLabel = getCliLabel(sourceFormat.value)
    const targetLabel = getCliLabel(targetFormat.value)
    a.download = `${sourceLabel}-to-${targetLabel}.${extension}`

    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)

    flashSuccess(t('converter.fileDownloaded'))
  }
}

const handleLoadExample = () => {
  const exampleJson = `{
  "mcpServers": {
    "context7": {
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"],
      "env": {
        "API_KEY": "your-api-key-here"
      }
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/files"]
    }
  }
}`
  configData.value = exampleJson
  sourceFormat.value = 'claude-code'
  flashSuccess(t('converter.exampleLoaded'), LOAD_MESSAGE_MS)
}
</script>

<style scoped>
.converter-view {
  min-width: 0;
}

.converter-results,
.converter-guide,
.converter-option-list {
  display: flex;
  flex-direction: column;
}

.converter-results {
  gap: 1.5rem;
}

.converter-guide {
  gap: 1rem;
}

.converter-option-list {
  gap: 0.5rem;
}

.converter-card {
  border: 1px solid var(--color-border-subtle);
  border-radius: 0.75rem;
  padding: 1.5rem;
  background: var(--color-bg-surface);
}

.converter-header,
.converter-toolbar,
.converter-option-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.converter-card__title {
  margin: 0;
  font-size: 1.0625rem;
  font-weight: 600;
  line-height: 1.3;
  color: var(--color-text-primary);
}

.converter-card__title--with-gap {
  margin-bottom: 0.5rem;
}

.converter-card__title--section {
  margin-bottom: 1rem;
}

.converter-card__title--compact {
  margin-bottom: 0.25rem;
}

.converter-section-copy {
  margin: 0 0 1rem;
  color: var(--color-text-muted);
  font-size: 0.875rem;
  line-height: 1.6;
}

.converter-section-copy--compact {
  margin-bottom: 0;
}

.converter-chip-button,
.converter-toolbar-button,
.converter-primary-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  border-radius: 0.5rem;
  font-size: 0.875rem;
  font-weight: 500;
}

.converter-chip-button {
  padding: 0.5rem 1rem;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  text-decoration: none;
}

.converter-chip-button:hover {
  color: var(--color-text-primary);
}

.converter-alert {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  border-radius: 0.5rem;
  padding: 1rem;
}

.converter-alert--error {
  border: 1px solid rgb(var(--color-danger-rgb) / 30%);
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--color-danger);
}

.converter-alert--success {
  border: 1px solid rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--color-success);
}

.converter-alert__icon {
  flex-shrink: 0;
  margin-top: 0.125rem;
}

.converter-selection-grid,
.converter-stats-grid {
  display: grid;
  gap: 1.5rem;
}

.converter-selection-grid {
  grid-template-columns: minmax(0, 1fr);
}

.converter-section-heading {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.converter-section-heading__icon {
  color: var(--color-accent-primary);
}

.converter-option-card {
  width: 100%;
  border: 1px solid var(--color-border-subtle);
  border-radius: 0.5rem;
  padding: 1rem;
  background: var(--color-bg-elevated);
  text-align: left;
  cursor: pointer;
}

.converter-option-card--active {
  border-color: var(--color-accent-primary);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

.converter-option-card--disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.converter-option-card__header {
  margin-bottom: 0.25rem;
}

.converter-option-card__title,
.converter-warning-panel__title,
.converter-guide__title {
  font-weight: 500;
  color: var(--color-text-primary);
}

.converter-option-card__description,
.converter-warning-list__item,
.converter-guide__list {
  margin: 0;
  font-size: 0.875rem;
  line-height: 1.6;
  color: var(--color-text-muted);
}

.converter-option-badge {
  border-radius: 0.25rem;
  padding: 0.125rem 0.5rem;
  background: var(--color-accent-primary);
  color: var(--color-accent-primary-contrast);
  font-size: 0.75rem;
  font-weight: 600;
}

.converter-toggle-list,
.converter-toolbar__actions,
.converter-action-row {
  display: flex;
  gap: 1rem;
}

.converter-toggle-list {
  flex-wrap: wrap;
  gap: 1.5rem;
}

.converter-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.converter-checkbox {
  width: 1rem;
  height: 1rem;
  cursor: pointer;
}

.converter-toolbar {
  margin-bottom: 1rem;
}

.converter-toolbar__actions {
  gap: 0.5rem;
}

.converter-toolbar-button {
  padding: 0.375rem 0.75rem;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
}

.converter-toolbar-button--label {
  cursor: pointer;
}

.converter-textarea {
  width: 100%;
  min-height: 300px;
  resize: none;
  border: 1px solid var(--color-border-subtle);
  border-radius: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-size: 0.875rem;
  line-height: 1.6;
}

.converter-textarea--result {
  min-height: 400px;
}

.converter-help-text {
  margin-top: 0.5rem;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.converter-action-row {
  justify-content: center;
}

.converter-primary-action {
  min-height: 44px;
  padding: 0.75rem 2rem;
  border: 1px solid transparent;
  background: var(--color-accent-primary);
  color: var(--color-accent-primary-contrast);
  font-weight: 600;
}

.converter-primary-action:disabled {
  cursor: not-allowed;
  background: var(--color-bg-elevated);
  color: var(--color-text-muted);
  opacity: 0.6;
}

.converter-stats-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin-bottom: 1rem;
  gap: 1rem;
}

.converter-stat {
  text-align: center;
}

.converter-stat__value {
  font-size: 1.5rem;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--color-text-primary);
}

.converter-stat__label {
  margin-top: 0.25rem;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.converter-warning-panel {
  border: 1px solid rgb(var(--color-warning-rgb) / 30%);
  border-radius: 0.5rem;
  background: rgb(var(--color-warning-rgb) / 10%);
  padding: 1rem;
  color: var(--color-warning);
}

.converter-warning-panel__title {
  margin-bottom: 0.5rem;
}

.converter-warning-list,
.converter-guide__list {
  padding-left: 1.25rem;
}

.converter-warning-list {
  list-style: disc;
}

.converter-guide__title {
  margin-bottom: 0.5rem;
  color: var(--color-text-secondary);
}

.converter-guide__list {
  list-style: disc;
}

@media (width >= 1024px) {
  .converter-selection-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width >= 768px) {
  .converter-stats-grid {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }
}

@media (width <= 767px) {
  .converter-toolbar {
    flex-direction: column;
    align-items: flex-start;
  }

  .converter-toolbar__actions {
    width: 100%;
    flex-wrap: wrap;
  }
}
</style>
