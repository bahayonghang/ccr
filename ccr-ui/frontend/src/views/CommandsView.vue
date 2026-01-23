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
          <div class="p-2 rounded-lg bg-guofeng-bg-tertiary">
            <Terminal class="w-6 h-6 text-guofeng-jade" />
          </div>
          <div>
            <h1 class="text-2xl font-bold text-guofeng-text-primary">
              {{ $t('commands.title') }}
            </h1>
            <p class="text-sm text-guofeng-text-secondary">
              {{ $t('commands.description') }}
            </p>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-[280px_1fr] gap-6">
        <!-- 左侧：工具与命令选择 -->
        <aside class="flex flex-col gap-6">
          <!-- 工具选择器 -->
          <GuofengCard
            variant="glass"
            class="flex flex-col overflow-hidden"
          >
            <div class="p-4 border-b border-guofeng-border/50">
              <h2 class="text-xs font-bold uppercase tracking-wider text-guofeng-text-secondary">
                {{ $t('commands.selectClient') }}
              </h2>
            </div>
            
            <div class="p-2 space-y-1">
              <button
                v-for="client in CLI_CLIENTS"
                :key="client.id"
                class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all relative group"
                :class="selectedClient === client.id ? 'bg-guofeng-bg-secondary' : 'hover:bg-guofeng-bg-secondary/50'"
                @click="setSelectedClient(client.id)"
              >
                <div 
                  class="absolute left-0 top-2 bottom-2 w-1 rounded-full transition-all"
                  :style="{ 
                    background: selectedClient === client.id ? client.color.replace('0.2', '1') : 'transparent',
                    opacity: selectedClient === client.id ? 1 : 0
                  }"
                />
                
                <div 
                  class="p-1.5 rounded-md transition-colors"
                  :style="{ 
                    background: selectedClient === client.id ? client.color : 'var(--bg-tertiary)' 
                  }"
                >
                  <component
                    :is="client.icon"
                    class="w-4 h-4"
                    :style="{ color: selectedClient === client.id ? 'white' : 'var(--text-secondary)' }"
                  />
                </div>
                
                <span 
                  class="text-sm font-medium"
                  :class="selectedClient === client.id ? 'text-guofeng-text-primary' : 'text-guofeng-text-secondary'"
                >
                  {{ client.name }}
                </span>
                
                <ChevronRight 
                  v-if="selectedClient === client.id"
                  class="w-4 h-4 ml-auto text-guofeng-text-secondary"
                />
              </button>
            </div>
          </GuofengCard>

          <!-- 命令列表 -->
          <GuofengCard
            variant="glass"
            class="flex-1 flex flex-col overflow-hidden min-h-[400px]"
          >
            <div class="p-4 border-b border-guofeng-border/50">
              <h2 class="text-xs font-bold uppercase tracking-wider text-guofeng-text-secondary">
                {{ $t('commands.availableCommands') }}
              </h2>
            </div>
            
            <div class="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
              <button
                v-for="cmd in commands"
                :key="cmd.name"
                class="w-full text-left px-4 py-3 rounded-lg transition-all group relative overflow-hidden"
                :class="selectedCommand === cmd.name ? 'bg-guofeng-bg-secondary' : 'hover:bg-guofeng-bg-secondary/50'"
                @click="setSelectedCommand(cmd.name)"
              >
                <div 
                  class="absolute left-0 top-0 bottom-0 w-1 transition-all bg-guofeng-jade"
                  :style="{ 
                    opacity: selectedCommand === cmd.name ? 1 : 0
                  }"
                />
                <div class="flex items-center justify-between">
                  <span 
                    class="font-mono text-sm font-semibold"
                    :class="selectedCommand === cmd.name ? 'text-guofeng-jade' : 'text-guofeng-text-primary'"
                  >
                    {{ cmd.name }}
                  </span>
                  <ChevronRight 
                    v-if="selectedCommand === cmd.name"
                    class="w-4 h-4 text-guofeng-jade"
                  />
                </div>
                <p class="text-xs mt-1 line-clamp-1 text-guofeng-text-secondary">
                  {{ cmd.description }}
                </p>
              </button>
            </div>
          </GuofengCard>
        </aside>

        <!-- 右侧：执行区域 -->
        <main class="flex flex-col gap-6 min-w-0">
          <!-- 命令详情与输入 -->
          <GuofengCard
            variant="glass"
            class="p-6"
          >
            <!-- 头部信息 -->
            <div class="mb-6">
              <div class="flex items-center gap-3 mb-2">
                <div class="p-2 rounded-lg bg-guofeng-bg-tertiary">
                  <component
                    :is="currentClientInfo?.icon"
                    class="w-6 h-6 text-guofeng-text-primary"
                  />
                </div>
                <div>
                  <h1 class="text-2xl font-bold text-guofeng-text-primary">
                    {{ selectedCommandInfo?.name || 'Select a command' }}
                  </h1>
                  <p class="text-sm text-guofeng-text-secondary">
                    {{ selectedCommandInfo?.description }}
                  </p>
                </div>
              </div>
            </div>

            <!-- 终端输入框 -->
            <div 
              class="rounded-lg p-4 font-mono text-sm transition-all bg-[#1e1e1e] border border-guofeng-border/50 shadow-inner"
            >
              <div class="flex items-center gap-2 mb-2 text-xs opacity-50 select-none text-gray-400">
                <Terminal class="w-3 h-3" />
                <span>COMMAND INPUT</span>
              </div>
              <div class="flex items-center gap-3 flex-wrap">
                <span class="font-bold select-none text-green-500">➜</span>
                <span class="font-bold select-none text-blue-400">~</span>
                <span class="font-bold select-none text-gray-200">{{ selectedClient }}</span>
                <span class="font-bold select-none text-yellow-500">{{ selectedCommand }}</span>
                
                <!-- Switch Command Dropdown -->
                <div
                  v-if="selectedCommand === 'switch'"
                  class="flex-1 min-w-[200px]"
                >
                  <select
                    v-model="args"
                    class="w-full bg-[#1e1e1e] border border-gray-700 rounded px-2 py-1 text-gray-200 focus:border-guofeng-jade focus:outline-none cursor-pointer hover:bg-[#2d2d2d] transition-colors"
                    @keydown.enter="!loading && handleExecute()"
                  >
                    <option
                      value=""
                      disabled
                      class="bg-[#1e1e1e] text-gray-500"
                    >
                      Select a configuration
                    </option>
                    <option 
                      v-for="config in configs" 
                      :key="config.name" 
                      :value="config.name"
                      class="bg-[#1e1e1e] text-gray-200 py-1"
                    >
                      {{ config.name }}
                    </option>
                  </select>
                </div>

                <!-- Default Text Input -->
                <input
                  v-else
                  v-model="args"
                  type="text"
                  :placeholder="$t('commands.argsPlaceholder')"
                  class="flex-1 bg-transparent border-none outline-none font-mono text-gray-200 min-w-[200px]"
                  @keydown.enter="!loading && handleExecute()"
                >
              </div>
            </div>

            <!-- 执行按钮 -->
            <div class="mt-4 flex justify-end">
              <button
                class="px-8 py-2.5 rounded-lg font-semibold text-sm text-white transition-all hover:scale-[1.02] active:scale-[0.98] flex items-center gap-2 bg-gradient-to-r from-guofeng-jade to-guofeng-jade-dark shadow-lg shadow-guofeng-jade/20"
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
          </GuofengCard>

          <!-- 输出区域 -->
          <GuofengCard
            v-if="output || loading"
            variant="glass"
            class="flex-1 overflow-hidden flex flex-col min-h-[400px] border-guofeng-border/50"
            :style="{ background: '#1e1e1e' }"
          >
            <!-- 终端头部 -->
            <div class="flex items-center justify-between px-4 py-2 bg-[#2d2d2d] border-b border-[#333]">
              <div class="flex items-center gap-2">
                <div class="flex gap-1.5">
                  <div class="w-3 h-3 rounded-full bg-[#ff5f56]" />
                  <div class="w-3 h-3 rounded-full bg-[#ffbd2e]" />
                  <div class="w-3 h-3 rounded-full bg-[#27c93f]" />
                </div>
                <span class="ml-3 text-xs text-gray-400 font-mono">bash — 80x24</span>
              </div>
              
              <div
                v-if="output"
                class="flex items-center gap-2"
              >
                <button
                  class="p-1.5 rounded hover:bg-[#3e3e3e] transition-colors text-gray-400 hover:text-white"
                  :title="$t('commands.copyOutput')"
                  :aria-label="$t('commands.copyOutput')"
                  @click="handleCopyOutput"
                >
                  <Copy class="w-3.5 h-3.5" />
                </button>
                <button
                  class="p-1.5 rounded hover:bg-[#3e3e3e] transition-colors text-gray-400 hover:text-white"
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
                class="absolute inset-0 flex items-center justify-center bg-black/20 backdrop-blur-sm"
              >
                <div class="flex flex-col items-center gap-3">
                  <Loader2 class="w-8 h-8 text-guofeng-jade animate-spin" />
                  <span class="text-gray-400 text-xs animate-pulse">Processing command...</span>
                </div>
              </div>

              <template v-if="output">
                <!-- 命令行回显 -->
                <div class="flex items-center gap-2 mb-4 opacity-50">
                  <span class="text-green-500">➜</span>
                  <span class="text-blue-400">~</span>
                  <span class="text-gray-300">{{ selectedClient }} {{ selectedCommand }} {{ args }}</span>
                </div>

                <!-- 实际输出 -->
                <pre
                  class="whitespace-pre-wrap break-words leading-relaxed hljs"
                  :class="output.success ? 'text-gray-200' : 'text-red-300'"
                  v-html="highlightedContent"
                />

                <!-- 状态行 -->
                <div class="mt-6 pt-4 border-t border-gray-800 flex items-center gap-4 text-xs font-mono">
                  <div class="flex items-center gap-2">
                    <span class="text-gray-500">Status:</span>
                    <span :class="output.success ? 'text-green-500' : 'text-red-500'">
                      {{ output.success ? 'SUCCESS' : 'FAILED' }}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-gray-500">Code:</span>
                    <span :class="output.exit_code === 0 ? 'text-gray-300' : 'text-red-500'">
                      {{ output.exit_code }}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-gray-500">Time:</span>
                    <span class="text-guofeng-jade">{{ output.duration_ms }}ms</span>
                  </div>
                </div>
              </template>
              
              <div
                v-else-if="!loading"
                class="h-full flex flex-col items-center justify-center text-gray-600 gap-2"
              >
                <Terminal class="w-12 h-12 opacity-20" />
                <p class="text-sm">
                  Ready to execute commands
                </p>
              </div>
            </div>
          </GuofengCard>
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
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

import { listCommands, executeCommand, listConfigs } from '@/api/client'
import type { CommandInfo, CommandResponse, ConfigItem } from '@/types'
import Navbar from '@/components/Navbar.vue'
import GuofengCard from '@/components/common/GuofengCard.vue'

// Register languages
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('json', json)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('plaintext', plaintext)

type CliClient = 'ccr' | 'claude' | 'qwen' | 'gemini' | 'iflow'

const { t } = useI18n({ useScope: 'global' })

const CLI_CLIENTS = [
  { id: 'ccr' as CliClient, name: 'CCR', icon: Zap, color: 'rgba(139, 92, 246, 0.2)' },
  { id: 'claude' as CliClient, name: 'Claude Code', icon: Code2, color: 'rgba(234, 88, 12, 0.2)' },
  { id: 'qwen' as CliClient, name: 'Qwen', icon: Sparkles, color: 'rgba(251, 191, 36, 0.2)' },
  { id: 'gemini' as CliClient, name: 'Gemini', icon: Gem, color: 'rgba(59, 130, 246, 0.2)' },
  { id: 'iflow' as CliClient, name: 'IFLOW', icon: Workflow, color: 'rgba(168, 85, 247, 0.2)' }
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
      return `<span class="text-red-300">${content}</span>`
    }
    
    // Auto-detect language
    const result = hljs.highlightAuto(content)
    return result.value
  } catch (e) {
    return content
  }
})

const loadConfigs = async () => {
  try {
    const response = await listConfigs()
    configs.value = response.configs
  } catch (err) {
    console.error('Failed to load configs:', err)
  }
}

const loadCommands = async () => {
  try {
    if (selectedClient.value === 'ccr') {
      const data = await listCommands()
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
    console.error('Failed to load commands:', err)
  }
}

onMounted(() => {
  loadCommands()
  loadConfigs()
})

watch(selectedClient, () => {
  selectedCommand.value = ''
  args.value = ''
  output.value = null
  loadCommands()
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
      output.value = result
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
      output.value = result
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
    console.error('Failed to copy:', err)
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
