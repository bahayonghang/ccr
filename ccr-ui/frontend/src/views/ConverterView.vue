<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="max-w-[1800px] mx-auto">
      <Navbar />

      <div class="grid grid-cols-[auto_1fr] gap-4">
        <CollapsibleSidebar module="converter" />

        <main class="space-y-6">
          <!-- Header -->
          <div
            class="rounded-xl p-6 glass-effect"
            :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
          >
            <div class="flex items-center justify-between mb-2">
              <h1
                class="text-3xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('converter.title') }}
              </h1>
              <RouterLink
                to="/"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                :style="{
                  background: 'var(--bg-secondary)',
                  color: 'var(--text-secondary)',
                  border: '1px solid var(--border-color)'
                }"
              >
                <Home class="w-4 h-4" />
                <span>{{ $t('converter.backToHome') }}</span>
              </RouterLink>
            </div>
            <p :style="{ color: 'var(--text-muted)' }">
              {{ $t('converter.description') }}
            </p>
          </div>

          <!-- Error/Success Messages -->
          <div
            v-if="error"
            class="rounded-lg p-4 flex items-start gap-3"
            :style="{ background: 'rgba(var(--color-danger-rgb), 0.1)', border: '1px solid var(--color-danger)' }"
          >
            <AlertCircle
              class="w-5 h-5 flex-shrink-0 mt-0.5"
              :style="{ color: 'var(--color-danger)' }"
            />
            <div :style="{ color: 'var(--color-danger)' }">
              {{ error }}
            </div>
          </div>

          <div
            v-if="successMessage"
            class="rounded-lg p-4 flex items-start gap-3"
            :style="{ background: 'rgba(var(--color-success-rgb), 0.1)', border: '1px solid var(--color-success)' }"
          >
            <Check
              class="w-5 h-5 flex-shrink-0 mt-0.5"
              :style="{ color: 'var(--color-success)' }"
            />
            <div :style="{ color: 'var(--color-success)' }">
              {{ successMessage }}
            </div>
          </div>

          <!-- Format Selection -->
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <!-- Source Format -->
            <div
              class="rounded-xl p-6 glass-effect"
              :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
            >
              <div class="flex items-center gap-2 mb-2">
                <FileJson
                  class="w-5 h-5"
                  :style="{ color: 'var(--accent-primary)' }"
                />
                <h2
                  class="text-xl font-bold"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('converter.sourceFormat') }}
                </h2>
              </div>
              <p
                class="mb-4"
                :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
              >
                {{ $t('converter.selectSource') }}
              </p>

              <div class="space-y-2">
                <div
                  v-for="type in cliTypes"
                  :key="type.value"
                  class="p-4 rounded-lg cursor-pointer transition-all"
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
                  <div class="flex items-center justify-between mb-1">
                    <span
                      class="font-medium"
                      :style="{ color: 'var(--text-primary)' }"
                    >{{
                      type.label
                    }}</span>
                    <span
                      v-if="sourceFormat === type.value"
                      class="px-2 py-0.5 rounded text-xs font-semibold"
                      :style="{ background: 'var(--accent-primary)', color: 'white' }"
                    >
                      {{ $t('converter.selected') }}
                    </span>
                  </div>
                  <p
                    class="text-sm"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ type.description }}
                  </p>
                </div>
              </div>
            </div>

            <!-- Target Format -->
            <div
              class="rounded-xl p-6 glass-effect"
              :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
            >
              <div class="flex items-center gap-2 mb-2">
                <FileCode
                  class="w-5 h-5"
                  :style="{ color: 'var(--accent-secondary)' }"
                />
                <h2
                  class="text-xl font-bold"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('converter.targetFormat') }}
                </h2>
              </div>
              <p
                class="mb-4"
                :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
              >
                {{ $t('converter.selectTarget') }}
              </p>

              <div class="space-y-2">
                <div
                  v-for="type in cliTypes"
                  :key="type.value"
                  class="p-4 rounded-lg cursor-pointer transition-all"
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
                  <div class="flex items-center justify-between mb-1">
                    <span
                      class="font-medium"
                      :style="{ color: 'var(--text-primary)' }"
                    >{{
                      type.label
                    }}</span>
                    <span
                      v-if="targetFormat === type.value && sourceFormat !== type.value"
                      class="px-2 py-0.5 rounded text-xs font-semibold"
                      :style="{ background: 'var(--accent-secondary)', color: 'white' }"
                    >
                      {{ $t('converter.selected') }}
                    </span>
                  </div>
                  <p
                    class="text-sm"
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
            class="rounded-xl p-6 glass-effect"
            :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
          >
            <h2
              class="text-xl font-bold mb-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('converter.convertOptions') }}
            </h2>
            <p
              class="mb-4"
              :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
            >
              {{ $t('converter.convertOptionsDesc') }}
            </p>

            <div class="flex flex-wrap gap-6">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="convertMcp"
                  type="checkbox"
                  class="w-4 h-4 cursor-pointer"
                >
                <span :style="{ color: 'var(--text-secondary)' }">{{ $t('converter.mcpServers') }}</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="convertCommands"
                  type="checkbox"
                  class="w-4 h-4 cursor-pointer"
                >
                <span :style="{ color: 'var(--text-secondary)' }">{{ $t('converter.slashCommands') }}</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="convertAgents"
                  type="checkbox"
                  class="w-4 h-4 cursor-pointer"
                >
                <span :style="{ color: 'var(--text-secondary)' }">{{ $t('converter.agentsConfig') }}</span>
              </label>
            </div>
          </div>

          <!-- Config Input -->
          <div
            class="rounded-xl p-6 glass-effect"
            :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
          >
            <div class="flex items-center justify-between mb-4">
              <div>
                <h2
                  class="text-xl font-bold mb-1"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('converter.configInput') }}
                </h2>
                <p :style="{ color: 'var(--text-muted)', fontSize: '14px' }">
                  {{ $t('converter.configInputDesc') }}
                </p>
              </div>
              <div class="flex gap-2">
                <button
                  class="px-3 py-1.5 rounded-lg font-medium text-sm"
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
                    class="px-3 py-1.5 rounded-lg font-medium text-sm cursor-pointer flex items-center gap-2"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      color: 'var(--text-primary)',
                      border: '1px solid var(--border-color)'
                    }"
                  >
                    <Upload class="w-4 h-4" />
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
              class="w-full px-3 py-2 rounded-lg font-mono text-sm resize-none"
              :style="{
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-primary)',
                minHeight: '300px'
              }"
            />
            <div
              class="mt-2 text-sm"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('converter.supportedFormats') }}
            </div>
          </div>

          <!-- Convert Button -->
          <div class="flex justify-center">
            <button
              class="px-8 py-3 rounded-lg font-semibold text-white flex items-center gap-2"
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
              <Loader2
                v-if="isConverting"
                class="w-5 h-5 animate-spin"
              />
              <ArrowRight
                v-else
                class="w-5 h-5"
              />
              {{ isConverting ? $t('converter.converting') : $t('converter.startConvert') }}
            </button>
          </div>

          <!-- Conversion Result -->
          <div
            v-if="result"
            class="space-y-6"
          >
            <!-- Statistics -->
            <div
              class="rounded-xl p-6 glass-effect"
              :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
            >
              <h2
                class="text-xl font-bold mb-4"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('converter.conversionStats') }}
              </h2>

              <div class="grid grid-cols-2 md:grid-cols-5 gap-4 mb-4">
                <div class="text-center">
                  <div
                    class="text-3xl font-bold"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    {{ result.stats?.mcp_servers || 0 }}
                  </div>
                  <div
                    class="text-sm mt-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.mcpServersCount') }}
                  </div>
                </div>
                <div class="text-center">
                  <div
                    class="text-3xl font-bold"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    {{ result.stats?.slash_commands || 0 }}
                  </div>
                  <div
                    class="text-sm mt-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.slashCommandsCount') }}
                  </div>
                </div>
                <div class="text-center">
                  <div
                    class="text-3xl font-bold"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    {{ result.stats?.agents || 0 }}
                  </div>
                  <div
                    class="text-sm mt-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.agentsCount') }}
                  </div>
                </div>
                <div class="text-center">
                  <div
                    class="text-3xl font-bold"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    {{ result.stats?.profiles || 0 }}
                  </div>
                  <div
                    class="text-sm mt-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.profilesCount') }}
                  </div>
                </div>
                <div class="text-center">
                  <div
                    class="text-3xl font-bold"
                    :style="{ color: 'var(--accent-primary)' }"
                  >
                    {{ result.stats?.base_config ? '✓' : '✗' }}
                  </div>
                  <div
                    class="text-sm mt-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('converter.baseConfig') }}
                  </div>
                </div>
              </div>

              <div
                v-if="result.warnings && result.warnings.length > 0"
                class="rounded-lg p-4"
                :style="{ background: 'rgba(var(--color-warning-rgb), 0.1)', border: '1px solid var(--color-warning)' }"
              >
                <div
                  class="font-medium mb-2"
                  :style="{ color: 'var(--color-warning)' }"
                >
                  {{ $t('converter.warnings') }}
                </div>
                <ul class="list-disc list-inside space-y-1">
                  <li
                    v-for="(warning, index) in result.warnings"
                    :key="index"
                    class="text-sm"
                    :style="{ color: 'var(--color-warning)' }"
                  >
                    {{ warning }}
                  </li>
                </ul>
              </div>
            </div>

            <!-- Result Display -->
            <div
              class="rounded-xl p-6 glass-effect"
              :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
            >
              <div class="flex items-center justify-between mb-4">
                <div>
                  <h2
                    class="text-xl font-bold mb-1"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('converter.conversionResult') }}
                  </h2>
                  <p :style="{ color: 'var(--text-muted)', fontSize: '14px' }">
                    {{ $t('converter.resultFormat', { format: result.format?.toUpperCase() || '' }) }}
                  </p>
                </div>
                <div class="flex gap-2">
                  <button
                    class="px-3 py-1.5 rounded-lg font-medium text-sm flex items-center gap-2"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      color: 'var(--text-primary)',
                      border: '1px solid var(--border-color)'
                    }"
                    @click="handleCopyResult"
                  >
                    <Copy class="w-4 h-4" />
                    {{ $t('converter.copy') }}
                  </button>
                  <button
                    class="px-3 py-1.5 rounded-lg font-medium text-sm flex items-center gap-2"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      color: 'var(--text-primary)',
                      border: '1px solid var(--border-color)'
                    }"
                    @click="handleDownloadResult"
                  >
                    <Download class="w-4 h-4" />
                    {{ $t('converter.download') }}
                  </button>
                </div>
              </div>

              <textarea
                :value="result.converted_data"
                readonly
                class="w-full px-3 py-2 rounded-lg font-mono text-sm resize-none"
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
            class="rounded-xl p-6 glass-effect"
            :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
          >
            <h2
              class="text-xl font-bold mb-4"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('converter.usageGuide') }}
            </h2>

            <div class="space-y-4">
              <div>
                <h4
                  class="font-medium mb-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('converter.usageNotes.supportedPathsTitle') }}
                </h4>
                <ul
                  class="list-disc list-inside space-y-1 text-sm"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  <li>{{ $t('converter.usageNotes.claudeCodex') }}</li>
                  <li>{{ $t('converter.usageNotes.otherFormats') }}</li>
                </ul>
              </div>
              <div>
                <h4
                  class="font-medium mb-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('converter.usageNotes.conversionNotesTitle') }}
                </h4>
                <ul
                  class="list-disc list-inside space-y-1 text-sm"
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
                  class="font-medium mb-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('converter.usageNotes.importantNotesTitle') }}
                </h4>
                <ul
                  class="list-disc list-inside space-y-1 text-sm"
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
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import {
  Home,
  FileJson,
  FileCode,
  Upload,
  ArrowRight,
  Loader2,
  Check,
  AlertCircle,
  Copy,
  Download,
} from 'lucide-vue-next'
import { convertConfig } from '@/api/client'
import type { ConverterRequest, ConverterResponse, CliType } from '@/types'
import Navbar from '@/components/Navbar.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'

const { t } = useI18n({ useScope: 'global' })

const CLI_DEFINITIONS: { value: CliType; label: string; descriptionKey: string }[] = [
  { value: 'claude-code', label: 'Claude Code', descriptionKey: 'converter.formatDescriptions.claudeCode' },
  { value: 'codex', label: 'Codex', descriptionKey: 'converter.formatDescriptions.codex' },
  { value: 'gemini', label: 'Gemini', descriptionKey: 'converter.formatDescriptions.gemini' },
  { value: 'qwen', label: 'Qwen', descriptionKey: 'converter.formatDescriptions.qwen' },
  { value: 'iflow', label: 'iFlow', descriptionKey: 'converter.formatDescriptions.iflow' }
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

const handleFileUpload = (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) {
    const reader = new FileReader()
    reader.onload = (e) => {
      const content = e.target?.result as string
      configData.value = content
      successMessage.value = t('converter.fileLoaded', { name: file.name })
      setTimeout(() => (successMessage.value = null), 3000)
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
  if (result.value?.converted_data) {
    navigator.clipboard.writeText(result.value.converted_data)
    successMessage.value = t('converter.copied')
    setTimeout(() => (successMessage.value = null), 2000)
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

    successMessage.value = t('converter.fileDownloaded')
    setTimeout(() => (successMessage.value = null), 2000)
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
  successMessage.value = t('converter.exampleLoaded')
  setTimeout(() => (successMessage.value = null), 3000)
}
</script>
