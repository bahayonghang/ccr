<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <!-- 🎨 动态背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-0 right-0 w-[600px] h-[600px] rounded-full opacity-10 blur-3xl"
        :style="{ background: 'radial-gradient(circle, var(--accent-primary) 0%, transparent 70%)' }"
      />
      <div
        class="absolute bottom-0 left-0 w-[500px] h-[500px] rounded-full opacity-10 blur-3xl"
        :style="{ background: 'radial-gradient(circle, var(--accent-secondary) 0%, transparent 70%)' }"
      />
    </div>

    <div class="max-w-[1800px] mx-auto">
      <!-- 导航栏 -->
      <Navbar />

      <!-- 页面标题 -->
      <div class="mb-6 mt-6">
        <div class="flex items-center gap-3 mb-2">
          <div class="p-2 rounded-lg bg-bg-surface">
            <Terminal class="w-6 h-6 text-accent-secondary" />
          </div>
          <div>
            <h1 class="text-2xl font-bold text-text-primary">
              {{ $t('commands.title') }}
            </h1>
            <p class="text-sm text-text-secondary">
              {{ $t('commands.description') }}
            </p>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-[280px_1fr] gap-6">
        <!-- 左侧：工具与命令选择 -->
        <aside class="flex flex-col gap-6">
          <!-- 工具选择器 -->
          <Card
            variant="glass"
            class="flex flex-col overflow-hidden"
          >
            <div class="p-4 border-b border-border-default/50">
              <h2 class="text-xs font-bold uppercase tracking-wider text-text-secondary">
                {{ $t('commands.selectClient') }}
              </h2>
            </div>
            
            <div class="p-2 space-y-1">
              <button
                v-for="client in CLI_CLIENTS"
                :key="client.id"
                class="group relative flex min-h-[48px] w-full items-center gap-3 rounded-lg px-3 py-2.5 transition-colors"
                :class="selectedClient === client.id ? 'bg-bg-elevated' : 'hover:bg-bg-elevated/50'"
                @click="setSelectedClient(client.id)"
              >
                <div 
                  class="absolute bottom-2 left-0 top-2 w-1 rounded-full transition-opacity"
                  :class="[client.markerClass, selectedClient === client.id ? 'opacity-100' : 'opacity-0']"
                />
                
                <div 
                  class="rounded-md p-1.5 transition-colors"
                  :class="selectedClient === client.id ? client.surfaceClass : 'bg-bg-surface text-text-secondary'"
                >
                  <component
                    :is="client.icon"
                    class="w-4 h-4"
                    :class="selectedClient === client.id ? client.textClass : 'text-text-secondary'"
                  />
                </div>
                
                <span 
                  class="text-sm font-medium"
                  :class="selectedClient === client.id ? 'text-text-primary' : 'text-text-secondary'"
                >
                  {{ client.name }}
                </span>
                
                <ChevronRight 
                  v-if="selectedClient === client.id"
                  class="w-4 h-4 ml-auto text-text-secondary"
                />
              </button>
            </div>
          </Card>

          <!-- 命令列表 -->
          <Card
            variant="glass"
            class="flex-1 flex flex-col overflow-hidden min-h-[400px]"
          >
            <div class="p-4 border-b border-border-default/50">
              <h2 class="text-xs font-bold uppercase tracking-wider text-text-secondary">
                {{ $t('commands.availableCommands') }}
              </h2>
            </div>
            
            <div class="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
              <button
                v-for="cmd in commands"
                :key="cmd.name"
                class="group relative w-full overflow-hidden rounded-lg px-4 py-3 text-left transition-colors"
                :class="selectedCommand === cmd.name ? 'bg-bg-elevated' : 'hover:bg-bg-elevated/50'"
                @click="setSelectedCommand(cmd.name)"
              >
                <div 
                  class="absolute left-0 top-0 bottom-0 w-1 transition-opacity bg-accent-secondary"
                  :style="{ 
                    opacity: selectedCommand === cmd.name ? 1 : 0
                  }"
                />
                <div class="flex items-center justify-between">
                  <span 
                    class="font-mono text-sm font-semibold"
                    :class="selectedCommand === cmd.name ? 'text-accent-secondary' : 'text-text-primary'"
                  >
                    {{ cmd.name }}
                  </span>
                  <ChevronRight 
                    v-if="selectedCommand === cmd.name"
                    class="w-4 h-4 text-accent-secondary"
                  />
                </div>
                <p class="text-xs mt-1 line-clamp-1 text-text-secondary">
                  {{ cmd.description }}
                </p>
              </button>
            </div>
          </Card>
        </aside>

        <!-- 右侧：执行区域 -->
        <main class="flex flex-col gap-6 min-w-0">
          <!-- 命令详情与输入 -->
          <Card
            variant="glass"
            class="p-6"
          >
            <!-- 头部信息 -->
            <div class="mb-6">
              <div class="flex items-center gap-3 mb-2">
                <div class="p-2 rounded-lg bg-bg-surface">
                  <component
                    :is="currentClientInfo?.icon"
                    class="w-6 h-6 text-text-primary"
                  />
                </div>
                <div>
                  <h1 class="text-2xl font-bold text-text-primary">
                    {{ selectedCommandInfo?.name || 'Select a command' }}
                  </h1>
                  <p class="text-sm text-text-secondary">
                    {{ selectedCommandInfo?.description }}
                  </p>
                </div>
              </div>
            </div>

            <!-- 终端输入框 -->
            <div 
              class="rounded-2xl border border-border-default/60 bg-bg-base/95 p-4 font-mono text-sm shadow-inner"
            >
              <div class="mb-2 flex items-center gap-2 text-xs text-text-muted opacity-70 select-none">
                <Terminal class="w-3 h-3" />
                <span>COMMAND INPUT</span>
              </div>
              <div class="flex items-center gap-3 flex-wrap">
                <span class="select-none font-bold text-accent-success">➜</span>
                <span class="select-none font-bold text-accent-info">~</span>
                <span class="select-none font-bold text-text-primary">{{ selectedClient }}</span>
                <span class="select-none font-bold text-accent-warning">{{ selectedCommand }}</span>
                
                <!-- Switch Command Dropdown -->
                <div
                  v-if="selectedCommand === 'switch'"
                  class="flex-1 min-w-[200px]"
                >
                  <label
                    for="commands-switch-select"
                    class="sr-only"
                  >{{ $t('commands.argsPlaceholder') }}</label>
                  <select
                    id="commands-switch-select"
                    v-model="args"
                    class="min-h-[44px] w-full cursor-pointer rounded-lg border border-border-default bg-bg-surface px-3 py-2 text-text-primary focus:border-accent-secondary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
                    @keydown.enter="!loading && handleExecute()"
                  >
                    <option
                      value=""
                      disabled
                      class="bg-bg-surface text-text-muted"
                    >
                      Select a configuration
                    </option>
                    <option 
                      v-for="config in configs" 
                      :key="config.name" 
                      :value="config.name"
                      class="bg-bg-surface text-text-primary"
                    >
                      {{ config.name }}
                    </option>
                  </select>
                </div>

                <!-- Default Text Input -->
                <template v-else>
                  <label
                    for="commands-args"
                    class="sr-only"
                  >{{ $t('commands.argsPlaceholder') }}</label>
                  <input
                    id="commands-args"
                    v-model="args"
                    type="text"
                    :placeholder="$t('commands.argsPlaceholder')"
                    class="min-h-[44px] min-w-[200px] flex-1 rounded-lg border border-transparent bg-transparent px-2 text-text-primary placeholder:text-text-muted outline-none transition-colors focus:border-accent-secondary/30 focus:bg-bg-surface/40"
                    @keydown.enter="!loading && handleExecute()"
                  >
                </template>
              </div>
            </div>

            <!-- 执行按钮 -->
            <div class="mt-4 flex justify-end">
              <button
                type="button"
                class="flex min-h-[44px] items-center gap-2 rounded-xl bg-accent-secondary px-8 py-2.5 text-sm font-semibold text-white shadow-lg shadow-accent-secondary/20 transition-colors hover:bg-accent-secondary/90 active:scale-[0.98]"
                :class="{ 'opacity-70 cursor-not-allowed': loading }"
                :disabled="loading"
                @click="handleExecute"
              >
                <Loader2
                  v-if="loading"
                  class="w-4 h-4 animate-spin"
                />
                <Play
                  v-else
                  class="w-4 h-4"
                />
                {{ loading ? $t('commands.executing') : $t('commands.executeCommand') }}
              </button>
            </div>
          </Card>

          <!-- 输出区域 -->
          <Card
            v-if="output || loading"
            variant="glass"
            class="flex-1 overflow-hidden flex flex-col min-h-[400px] border-border-default/50"
            :class="'bg-bg-base/95'"
          >
            <!-- 终端头部 -->
            <div class="flex items-center justify-between border-b border-border-default/50 bg-bg-surface/80 px-4 py-2">
              <div class="flex items-center gap-2">
                <div class="flex gap-1.5">
                  <div class="h-3 w-3 rounded-full bg-accent-danger" />
                  <div class="h-3 w-3 rounded-full bg-accent-warning" />
                  <div class="h-3 w-3 rounded-full bg-accent-success" />
                </div>
                <span class="ml-3 font-mono text-xs text-text-muted">bash - 80x24</span>
              </div>
              
              <div
                v-if="output"
                class="flex items-center gap-2"
              >
                <button
                  type="button"
                  class="flex min-h-[36px] min-w-[36px] items-center justify-center rounded-lg text-text-muted transition-colors hover:bg-bg-elevated hover:text-text-primary"
                  :title="$t('commands.copyOutput')"
                  :aria-label="$t('commands.copyOutput')"
                  @click="handleCopyOutput"
                >
                  <Copy class="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  class="flex min-h-[36px] min-w-[36px] items-center justify-center rounded-lg text-text-muted transition-colors hover:bg-bg-elevated hover:text-text-primary"
                  :title="$t('commands.clearOutputButton')"
                  :aria-label="$t('commands.clearOutputButton')"
                  @click="handleClearOutput"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>

            <!-- 终端内容 -->
            <div class="flex-1 p-4 font-mono text-sm overflow-y-auto custom-scrollbar relative">
              <div
                v-if="loading"
                class="absolute inset-0 flex items-center justify-center bg-black/20 backdrop-blur-md"
              >
                <div class="flex flex-col items-center gap-3">
                  <Loader2 class="w-8 h-8 text-accent-secondary animate-spin" />
                  <span class="animate-pulse text-xs text-text-muted">Processing command...</span>
                </div>
              </div>

              <template v-if="output">
                <!-- 命令行回显 -->
                <div class="flex items-center gap-2 mb-4 opacity-50">
                  <span class="text-accent-success">➜</span>
                  <span class="text-accent-info">~</span>
                  <span class="text-text-secondary">{{ selectedClient }} {{ selectedCommand }} {{ args }}</span>
                </div>

                <!-- 实际输出 -->
                <pre
                  class="whitespace-pre-wrap break-words leading-relaxed hljs"
                  :class="output.success ? 'text-text-primary' : 'text-accent-danger'"
                  v-html="highlightedContent"
                />

                <!-- 状态行 -->
                <div class="mt-6 flex items-center gap-4 border-t border-border-default/40 pt-4 text-xs font-mono">
                  <div class="flex items-center gap-2">
                    <span class="text-text-muted">Status:</span>
                    <span :class="output.success ? 'text-accent-success' : 'text-accent-danger'">
                      {{ output.success ? 'SUCCESS' : 'FAILED' }}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-text-muted">Code:</span>
                    <span :class="output.exit_code === 0 ? 'text-text-primary' : 'text-accent-danger'">
                      {{ output.exit_code }}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-text-muted">Time:</span>
                    <span class="text-accent-secondary">{{ output.duration_ms }}ms</span>
                  </div>
                </div>
              </template>
              
              <div
                v-else-if="!loading"
                class="flex h-full flex-col items-center justify-center gap-2 text-text-muted"
              >
                <Terminal class="w-12 h-12 opacity-20" />
                <p class="text-sm">
                  Ready to execute commands
                </p>
              </div>
            </div>
          </Card>
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import {
  Zap, Sparkles, Gem, Workflow,
  Play, Copy, Trash2, Terminal,
  ChevronRight, Loader2, Code2
} from 'lucide-vue-next'
import hljs from 'highlight.js/lib/core'
import bash from 'highlight.js/lib/languages/bash'
import json from 'highlight.js/lib/languages/json'
import markdown from 'highlight.js/lib/languages/markdown'
import plaintext from 'highlight.js/lib/languages/plaintext'
import 'highlight.js/styles/atom-one-dark.css'

import { listCommands, executeCommand, listConfigs } from '@/api'
import type { CommandInfo, CommandResponse, ConfigItem } from '@/types'
import { normalizeCliClient, type CliClient } from '@/types/router'
import Navbar from '@/components/Navbar.vue'
import Card from '@/components/ui/Card.vue'
import { sanitizeTerminal } from '@/utils/sanitize'
import { logger } from '@/utils/logger'

// Register languages
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('json', json)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('plaintext', plaintext)

const { t } = useI18n({ useScope: 'global' })
const route = useRoute()
const router = useRouter()

const CLI_CLIENTS = [
  { id: 'ccr' as CliClient, name: 'CCR', icon: Zap, surfaceClass: 'bg-accent-primary/10', textClass: 'text-accent-primary', markerClass: 'bg-accent-primary' },
  { id: 'claude' as CliClient, name: 'Claude Code', icon: Code2, surfaceClass: 'bg-accent-secondary/10', textClass: 'text-accent-secondary', markerClass: 'bg-accent-secondary' },
  { id: 'qwen' as CliClient, name: 'Qwen', icon: Sparkles, surfaceClass: 'bg-accent-warning/10', textClass: 'text-accent-warning', markerClass: 'bg-accent-warning' },
  { id: 'gemini' as CliClient, name: 'Gemini', icon: Gem, surfaceClass: 'bg-accent-info/10', textClass: 'text-accent-info', markerClass: 'bg-accent-info' },
  { id: 'iflow' as CliClient, name: 'IFLOW', icon: Workflow, surfaceClass: 'bg-accent-primary/10', textClass: 'text-accent-primary', markerClass: 'bg-accent-primary' }
]

const selectedClient = ref<CliClient>('ccr')
const commands = ref<CommandInfo[]>([])
const selectedCommand = ref<string>('')
const args = ref<string>('')
const output = ref<CommandResponse | null>(null)
const streamingOutput = ref<string>('')
const loading = ref(false)
const configs = ref<ConfigItem[]>([])

const selectedCommandInfo = computed(() =>
  commands.value.find((c) => c.name === selectedCommand.value)
)

const currentClientInfo = computed(() =>
  CLI_CLIENTS.find((c) => c.id === selectedClient.value)
)

const highlightedContent = computed(() => {
  // Prefer streamingOutput during active streaming
  const content = streamingOutput.value || (output.value?.output || output.value?.error) || ''
  if (!content) return ''

  try {
    // If it's an error, just return plain text or wrap in error style
    if (output.value && !output.value.success) {
      return `<span class="text-red-300">${sanitizeTerminal(content)}</span>`
    }

    // Auto-detect language and sanitize
    const result = hljs.highlightAuto(content)
    return sanitizeTerminal(result.value)
  } catch (e) {
    return sanitizeTerminal(content)
  }
})

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
    duration_ms: 0
  }
}

const loadConfigs = async () => {
  try {
    const response = await listConfigs<{ configs: ConfigItem[] }>()
    configs.value = response.configs
  } catch (err) {
    logger.error('Failed to load configs:', err)
  }
}

const loadCommands = async () => {
  try {
    if (selectedClient.value === 'ccr') {
      const data = await listCommands<CommandInfo[]>()
      commands.value = data
      if (data.length > 0 && !selectedCommand.value) {
        selectedCommand.value = data[0].name
      }
    } else {
      const clientName = CLI_CLIENTS.find((c) => c.id === selectedClient.value)?.name || selectedClient.value
      commands.value = [
        {
          name: 'help',
          description: t('commands.helpDescription', { client: clientName }),
          usage: `${selectedClient.value} --help`,
          examples: [`${selectedClient.value} --help`]
        },
        {
          name: 'version',
          description: t('commands.versionDescription', { client: clientName }),
          usage: `${selectedClient.value} --version`,
          examples: [`${selectedClient.value} --version`]
        },
        {
          name: 'login',
          description: `Login to ${clientName}`,
          usage: `${selectedClient.value} login`,
          examples: [`${selectedClient.value} login`]
        }
      ]
      selectedCommand.value = 'help'
    }
  } catch (err) {
    logger.error('Failed to load commands:', err)
  }
}

onMounted(() => {
  const initialClient = normalizeCliClient(route.params.client)
  if (initialClient) {
    selectedClient.value = initialClient
  }
  loadCommands()
  loadConfigs()
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
  output.value = null
  loadCommands()

  const current = normalizeCliClient(route.params.client) || 'ccr'
  if (current !== selectedClient.value) {
    router.replace({ name: 'commands', params: { client: selectedClient.value } })
  }
})

const setSelectedClient = (client: CliClient) => {
  selectedClient.value = client
}

const setSelectedCommand = (cmd: string) => {
  selectedCommand.value = cmd
  args.value = '' // Clear args when switching commands
}

const handleExecute = async () => {
  if (!selectedCommand.value) return

  loading.value = true
  try {
    const argsArray = args.value
      .split(' ')
      .map((a) => a.trim())
      .filter((a) => a.length > 0)

    if (selectedClient.value === 'ccr') {
      const result = await executeCommand({
        command: selectedCommand.value,
        args: argsArray
      })
      output.value = normalizeCommandResponse(result)
    } else {
      // For other clients, map selected command to args if needed
      const finalArgs = [...argsArray]
      
      // Prepend command-specific flags if they aren't already in args
      if (selectedCommand.value === 'help' && !finalArgs.includes('--help')) {
        finalArgs.unshift('--help')
      } else if (selectedCommand.value === 'version' && !finalArgs.includes('--version')) {
        finalArgs.unshift('--version')
      } else if (selectedCommand.value === 'login' && !finalArgs.includes('login')) {
        finalArgs.unshift('login')
      }

      const result = await executeCommand({
        command: selectedClient.value,
        args: finalArgs
      })
      output.value = normalizeCommandResponse(result)
    }
  } catch (err) {
    output.value = {
      success: false,
      output: '',
      error: err instanceof Error ? err.message : t('commands.unknownError'),
      exit_code: -1,
      duration_ms: 0
    }
  } finally {
    loading.value = false
  }
}

const handleCopyOutput = async () => {
  if (!output.value) return
  const text = output.value.output + (output.value.error ? '\n' + output.value.error : '')
  try {
    await navigator.clipboard.writeText(text)
    // Could add a toast notification here
  } catch (err) {
    logger.error('Failed to copy:', err)
  }
}

const handleClearOutput = () => {
  output.value = null
}
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgb(255 255 255 / 10%);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgb(255 255 255 / 20%);
}

/* Override highlight.js background to match our theme */
:deep(.hljs) {
  background: transparent !important;
  padding: 0 !important;
}
</style>
