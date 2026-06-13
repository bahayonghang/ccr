<template>
  <div
    class="converter-view"
    :style="{ background: 'var(--bg-primary)' }"
  >
    <div class="converter-view__container">
      <div class="converter-view__stack">
        <ModuleSubnav module="converter" />

        <main class="converter-view__main">
          <!-- Header -->
          <div
            class="converter-card converter-card--header glass-effect"
            :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
          >
            <div class="converter-header">
              <h1
                class="converter-title"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('converter.title') }}
              </h1>
              <RouterLink
                to="/"
                class="converter-chip-button"
                :style="{
                  background: 'var(--bg-secondary)',
                  color: 'var(--text-secondary)',
                  border: '1px solid var(--border-color)'
                }"
              >
                <SIcon
                  name="Home"
                  size="w-4 h-4"
                />
                <span>{{ $t('converter.backToHome') }}</span>
              </RouterLink>
            </div>
            <p
              class="converter-muted-text"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('converter.description') }}
            </p>
          </div>

          <!-- Error/Success Messages -->
          <div
            v-if="error"
            class="converter-alert converter-alert--error"
            :style="{ background: 'rgba(var(--color-danger-rgb), 0.1)', border: '1px solid var(--color-danger)' }"
          >
            <SIcon
              name="AlertCircle"
              size="w-5 h-5"
              class="converter-alert__icon"
              :style="{ color: 'var(--color-danger)' }"
            />
            <div :style="{ color: 'var(--color-danger)' }">
              {{ error }}
            </div>
          </div>

          <div
            v-if="successMessage"
            class="converter-alert converter-alert--success"
            :style="{ background: 'rgba(var(--color-success-rgb), 0.1)', border: '1px solid var(--color-success)' }"
          >
            <SIcon
              name="Check"
              size="w-5 h-5"
              class="converter-alert__icon"
              :style="{ color: 'var(--color-success)' }"
            />
            <div :style="{ color: 'var(--color-success)' }">
              {{ successMessage }}
            </div>
          </div>

          <!-- Format Selection -->
          <div class="converter-selection-grid">
            <!-- Source Format -->
            <div
              class="converter-card glass-effect"
              :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
            >
              <div class="converter-section-heading">
                <SIcon
                  name="FileJson"
                  size="w-5 h-5"
                  :style="{ color: 'var(--accent-primary)' }"
                />
                <h2
                  class="converter-card__title"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('converter.sourceFormat') }}
                </h2>
              </div>
              <p
                class="converter-section-copy"
                :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
              >
                {{ $t('converter.selectSource') }}
              </p>

              <div class="converter-option-list">
                <div
                  v-for="type in cliTypes"
                  :key="type.value"
                  class="converter-option-card"
                  :style="{
                    border:
                      sourceFormat === type.value
                        ? '2px solid var(--accent-primary)'
                        : '1px solid var(--border-color)',
                    background:
                      sourceFormat === type.value
                        ? 'rgba(var(--color-accent-secondary-rgb), 0.1)'
                        : 'var(--bg-tertiary)'
                  }"
                  @click="sourceFormat = type.value"
                >
                  <div class="converter-option-card__header">
                    <span
                      class="converter-option-card__title"
                      :style="{ color: 'var(--text-primary)' }"
                    >{{
                      type.label
                    }}</span>
                    <span
                      v-if="sourceFormat === type.value"
                      class="converter-option-badge"
                      :style="{ background: 'var(--accent-primary)', color: 'white' }"
                    >
                      {{ $t('converter.selected') }}
                    </span>
                  </div>
                  <p
                    class="converter-option-card__description"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ type.description }}
                  </p>
                </div>
              </div>
            </div>

            <!-- Target Format -->
            <div
              class="converter-card glass-effect"
              :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
            >
              <div class="converter-section-heading">
                <SIcon
                  name="FileCode"
                  size="w-5 h-5"
                  :style="{ color: 'var(--accent-secondary)' }"
                />
                <h2
                  class="converter-card__title"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('converter.targetFormat') }}
                </h2>
              </div>
              <p
                class="converter-section-copy"
                :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
              >
                {{ $t('converter.selectTarget') }}
              </p>

              <div class="converter-option-list">
                <div
                  v-for="type in cliTypes"
                  :key="type.value"
                  class="converter-option-card"
                  :style="{
                    border:
                      targetFormat === type.value && sourceFormat !== type.value
                        ? '2px solid var(--accent-secondary)'
                        : '1px solid var(--border-color)',
                    background:
                      targetFormat === type.value && sourceFormat !== type.value
                        ? 'rgba(var(--color-accent-secondary-rgb), 0.1)'
                        : 'var(--bg-tertiary)',
                    opacity: sourceFormat === type.value ? 0.5 : 1,
                    cursor: sourceFormat === type.value ? 'not-allowed' : 'pointer'
                  }"
                  @click="
                    () => {
                      if (sourceFormat !== type.value) {
                        targetFormat = type.value
                      }
                    }
                  "
                >
                  <div class="converter-option-card__header">
                    <span
                      class="converter-option-card__title"
                      :style="{ color: 'var(--text-primary)' }"
                    >{{
                      type.label
                    }}</span>
                    <span
                      v-if="targetFormat === type.value && sourceFormat !== type.value"
                      class="converter-option-badge"
                      :style="{ background: 'var(--accent-secondary)', color: 'white' }"
                    >
                      {{ $t('converter.selected') }}
                    </span>
                  </div>
                  <p
                    class="converter-option-card__description"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ type.description }}
                  </p>
                </div>
              </div>
            </div>
          </div>

          <!-- Conversion Options -->
          <div
            class="converter-card glass-effect"
            :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
          >
            <h2
              class="converter-card__title converter-card__title--with-gap"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('converter.convertOptions') }}
            </h2>
            <p
              class="converter-section-copy"
              :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
            >
              {{ $t('converter.convertOptionsDesc') }}
            </p>

            <div class="converter-toggle-list">
              <label class="converter-toggle">
                <input
                  v-model="convertMcp"
                  type="checkbox"
                  class="converter-checkbox"
                >
                <span :style="{ color: 'var(--text-secondary)' }">{{ $t('converter.mcpServers') }}</span>
              </label>
              <label class="converter-toggle">
                <input
                  v-model="convertCommands"
                  type="checkbox"
                  class="converter-checkbox"
                >
                <span :style="{ color: 'var(--text-secondary)' }">{{ $t('converter.slashCommands') }}</span>
              </label>
              <label class="converter-toggle">
                <input
                  v-model="convertAgents"
                  type="checkbox"
                  class="converter-checkbox"
                >
                <span :style="{ color: 'var(--text-secondary)' }">{{ $t('converter.agentsConfig') }}</span>
              </label>
            </div>
          </div>

          <!-- Config Input -->
          <div
            class="converter-card glass-effect"
            :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
          >
            <div class="converter-toolbar">
              <div>
                <h2
                  class="converter-card__title converter-card__title--compact"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('converter.configInput') }}
                </h2>
                <p
                  class="converter-section-copy converter-section-copy--compact"
                  :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
                >
                  {{ $t('converter.configInputDesc') }}
                </p>
              </div>
              <div class="converter-toolbar__actions">
                <button
                  class="converter-toolbar-button"
                  :style="{
                    background: 'var(--bg-tertiary)',
                    color: 'var(--text-primary)',
                    border: '1px solid var(--border-color)'
                  }"
                  @click="handleLoadExample"
                >
                  {{ $t('converter.loadExample') }}
                </button>
                <label>
                  <span
                    class="converter-toolbar-button converter-toolbar-button--label"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      color: 'var(--text-primary)',
                      border: '1px solid var(--border-color)'
                    }"
                  >
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
              :style="{
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-primary)',
                minHeight: '300px'
              }"
            />
            <div
              class="converter-help-text"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('converter.supportedFormats') }}
            </div>
          </div>

          <!-- Convert Button -->
          <div class="converter-action-row">
            <button
              class="converter-primary-action"
              :style="{
                background:
                  isConverting || !configData.trim() || sourceFormat === targetFormat
                    ? 'var(--bg-tertiary)'
                    : 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                boxShadow:
                  isConverting || !configData.trim() || sourceFormat === targetFormat
                    ? 'none'
                    : '0 0 20px var(--glow-primary)',
                opacity: isConverting || !configData.trim() || sourceFormat === targetFormat ? 0.5 : 1,
                cursor:
                  isConverting || !configData.trim() || sourceFormat === targetFormat
                    ? 'not-allowed'
                    : 'pointer'
              }"
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

          <!-- Conversion Result -->
          <div
            v-if="result"
            class="converter-results"
          >
            <!-- Statistics -->
            <div
              class="converter-card glass-effect"
              :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
            >
              <h2
                class="converter-card__title converter-card__title--section"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('converter.conversionStats') }}
              </h2>

              <div class="converter-stats-grid">
                <div class="converter-stat">
                  <div
                    class="converter-stat__value"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    {{ result.stats?.mcp_servers || 0 }}
                  </div>
                  <div
                    class="converter-stat__label"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.mcpServersCount') }}
                  </div>
                </div>
                <div class="converter-stat">
                  <div
                    class="converter-stat__value"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    {{ result.stats?.slash_commands || 0 }}
                  </div>
                  <div
                    class="converter-stat__label"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.slashCommandsCount') }}
                  </div>
                </div>
                <div class="converter-stat">
                  <div
                    class="converter-stat__value"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    {{ result.stats?.agents || 0 }}
                  </div>
                  <div
                    class="converter-stat__label"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.agentsCount') }}
                  </div>
                </div>
                <div class="converter-stat">
                  <div
                    class="converter-stat__value"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    {{ result.stats?.profiles || 0 }}
                  </div>
                  <div
                    class="converter-stat__label"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.profilesCount') }}
                  </div>
                </div>
                <div class="converter-stat">
                  <div
                    class="converter-stat__value"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    <SIcon
                      :name="result.stats?.base_config ? 'Check' : 'X'"
                      size="w-6 h-6"
                      class="mx-auto"
                    />
                  </div>
                  <div
                    class="converter-stat__label"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.baseConfig') }}
                  </div>
                </div>
              </div>

              <div
                v-if="result.warnings && result.warnings.length > 0"
                class="converter-warning-panel"
                :style="{ background: 'rgba(var(--color-warning-rgb), 0.1)', border: '1px solid var(--color-warning)' }"
              >
                <div
                  class="converter-warning-panel__title"
                  :style="{ color: 'var(--color-warning)' }"
                >
                  {{ $t('converter.warnings') }}
                </div>
                <ul class="converter-warning-list">
                  <li
                    v-for="(warning, index) in result.warnings"
                    :key="index"
                    class="converter-warning-list__item"
                    :style="{ color: 'var(--color-warning)' }"
                  >
                    {{ warning }}
                  </li>
                </ul>
              </div>
            </div>

            <!-- Result Display -->
            <div
              class="converter-card glass-effect"
              :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
            >
              <div class="converter-toolbar">
                <div>
                  <h2
                    class="converter-card__title converter-card__title--compact"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('converter.conversionResult') }}
                  </h2>
                  <p
                    class="converter-section-copy converter-section-copy--compact"
                    :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
                  >
                    {{ $t('converter.resultFormat', { format: result.format?.toUpperCase() || '' }) }}
                  </p>
                </div>
                <div class="converter-toolbar__actions">
                  <button
                    class="converter-toolbar-button converter-toolbar-button--label"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      color: 'var(--text-primary)',
                      border: '1px solid var(--border-color)'
                    }"
                    @click="handleCopyResult"
                  >
                    <SIcon
                      name="Copy"
                      size="w-4 h-4"
                    />
                    {{ $t('converter.copy') }}
                  </button>
                  <button
                    class="converter-toolbar-button converter-toolbar-button--label"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      color: 'var(--text-primary)',
                      border: '1px solid var(--border-color)'
                    }"
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
                :value="result.converted_data"
                readonly
                class="converter-textarea converter-textarea--result"
                :style="{
                  background: 'var(--bg-tertiary)',
                  border: '1px solid var(--border-color)',
                  color: 'var(--text-primary)',
                  minHeight: '400px'
                }"
              />
            </div>
          </div>

          <!-- Usage Guide -->
          <div
            class="converter-card glass-effect"
            :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
          >
            <h2
              class="converter-card__title converter-card__title--section"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('converter.usageGuide') }}
            </h2>

            <div class="converter-guide">
              <div>
                <h4
                  class="converter-guide__title"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('converter.usageNotes.supportedPathsTitle') }}
                </h4>
                <ul
                  class="converter-guide__list"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  <li>{{ $t('converter.usageNotes.claudeCodex') }}</li>
                  <li>{{ $t('converter.usageNotes.otherFormats') }}</li>
                </ul>
              </div>
              <div>
                <h4
                  class="converter-guide__title"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('converter.usageNotes.conversionNotesTitle') }}
                </h4>
                <ul
                  class="converter-guide__list"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  <li>{{ $t('converter.usageNotes.note1') }}</li>
                  <li>{{ $t('converter.usageNotes.note2') }}</li>
                  <li>{{ $t('converter.usageNotes.note3') }}</li>
                  <li>{{ $t('converter.usageNotes.note4') }}</li>
                </ul>
              </div>
              <div>
                <h4
                  class="converter-guide__title"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('converter.usageNotes.importantNotesTitle') }}
                </h4>
                <ul
                  class="converter-guide__list"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  <li>{{ $t('converter.usageNotes.caution1') }}</li>
                  <li>{{ $t('converter.usageNotes.caution2') }}</li>
                  <li>{{ $t('converter.usageNotes.caution3') }}</li>
                </ul>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  </div>
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

    const response = await convertConfig<ConverterResponse>(request)
    result.value = response
    successMessage.value = t('converter.convertSuccess')
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('converter.convertError')
  } finally {
    isConverting.value = false
  }
}

const handleCopyResult = () => {
  if (result.value?.converted_data) {
    void copyText(result.value.converted_data)
    flashSuccess(t('converter.copied'))
  }
}

const handleDownloadResult = () => {
  if (result.value?.converted_data) {
    const blob = new Blob([result.value.converted_data], { type: 'text/plain' })
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
  min-height: 100%;
  padding: 1.25rem;
  transition: background-color 0.3s ease, color 0.3s ease;
}

.converter-view__container {
  max-width: 1800px;
  margin: 0 auto;
}

.converter-view__stack,
.converter-view__main,
.converter-results,
.converter-guide,
.converter-option-list {
  display: flex;
  flex-direction: column;
}

.converter-view__stack {
  gap: 1rem;
}

.converter-view__main,
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
  border-radius: 0.75rem;
  padding: 1.5rem;
}

.converter-card--header {
  overflow: hidden;
}

.converter-header,
.converter-toolbar,
.converter-option-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.converter-header {
  margin-bottom: 0.5rem;
}

.converter-title,
.converter-card__title {
  font-weight: 700;
  line-height: 1.2;
}

.converter-title {
  font-size: 1.875rem;
}

.converter-card__title {
  font-size: 1.25rem;
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

.converter-muted-text,
.converter-section-copy {
  line-height: 1.6;
}

.converter-section-copy {
  margin-bottom: 1rem;
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
  transition: background-color 0.2s ease, border-color 0.2s ease, color 0.2s ease;
}

.converter-alert {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  border-radius: 0.5rem;
  padding: 1rem;
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
  grid-template-columns: repeat(1, minmax(0, 1fr));
}

.converter-section-heading {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.converter-option-card {
  border-radius: 0.5rem;
  padding: 1rem;
  transition: background-color 0.2s ease, border-color 0.2s ease, opacity 0.2s ease;
}

.converter-option-card__header {
  margin-bottom: 0.25rem;
}

.converter-option-card__title,
.converter-warning-panel__title,
.converter-guide__title {
  font-weight: 500;
}

.converter-option-card__description,
.converter-warning-list__item,
.converter-guide__list {
  font-size: 0.875rem;
  line-height: 1.6;
}

.converter-option-badge {
  border-radius: 0.25rem;
  padding: 0.125rem 0.5rem;
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
}

.converter-toolbar-button--label {
  cursor: pointer;
}

.converter-textarea {
  width: 100%;
  resize: none;
  border-radius: 0.5rem;
  padding: 0.5rem 0.75rem;
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
}

.converter-action-row {
  justify-content: center;
}

.converter-primary-action {
  padding: 0.75rem 2rem;
  color: white;
  font-weight: 600;
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
  font-size: 1.875rem;
  font-weight: 700;
}

.converter-stat__label {
  margin-top: 0.25rem;
  font-size: 0.875rem;
}

.converter-warning-panel {
  border-radius: 0.5rem;
  padding: 1rem;
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
  .converter-header,
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
