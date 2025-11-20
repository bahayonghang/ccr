<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh' }">
    <div class="relative z-10 p-6">
      <div class="max-w-[1800px] mx-auto">
        <!-- 导航栏 -->
        <Navbar />

        <!-- 页面标题 -->
        <div class="mb-6">
          <div class="flex items-center gap-3 mb-2">
            <div 
              class="p-2 rounded-lg"
              :style="{ background: 'var(--bg-tertiary)' }"
            >
              <Terminal
                class="w-6 h-6"
                :style="{ color: 'var(--accent-primary)' }"
              />
            </div>
            <div>
              <h1
                class="text-2xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('commands.title') }}
              </h1>
              <p
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t('commands.description') }}
              </p>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-[260px_1fr] gap-6">
          <!-- 左侧：工具与命令选择 -->
          <aside class="flex flex-col gap-6">
            <!-- 工具选择器 (垂直列表) -->
            <section
              class="rounded-xl glass-effect overflow-hidden"
              :style="{
                border: '1px solid var(--border-color)',
                boxShadow: 'var(--shadow-small)'
              }"
            >
              <div
                class="p-4 border-b"
                :style="{ borderColor: 'var(--border-color)' }"
              >
                <h2
                  class="text-xs font-bold uppercase tracking-wider"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('commands.selectClient') }}
                </h2>
              </div>
              
              <div class="p-2 space-y-1">
                <button
                  v-for="client in CLI_CLIENTS"
                  :key="client.id"
                  class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all relative group"
                  :style="{
                    background: selectedClient === client.id ? 'var(--bg-secondary)' : 'transparent',
                  }"
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
                    :style="{ color: selectedClient === client.id ? 'var(--text-primary)' : 'var(--text-secondary)' }"
                  >
                    {{ client.name }}
                  </span>
                  
                  <ChevronRight 
                    v-if="selectedClient === client.id"
                    class="w-4 h-4 ml-auto"
                    :style="{ color: 'var(--text-secondary)' }"
                  />
                </button>
              </div>
            </section>

            <!-- 命令列表 -->
            <section
              class="flex-1 rounded-xl glass-effect flex flex-col overflow-hidden min-h-[400px]"
              :style="{
                border: '1px solid var(--border-color)',
                boxShadow: 'var(--shadow-small)'
              }"
            >
              <div
                class="p-4 border-b"
                :style="{ borderColor: 'var(--border-color)' }"
              >
                <h2
                  class="text-xs font-bold uppercase tracking-wider"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('commands.availableCommands') }}
                </h2>
              </div>
              
              <div class="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
                <button
                  v-for="cmd in commands"
                  :key="cmd.name"
                  class="w-full text-left px-4 py-3 rounded-lg transition-all group relative overflow-hidden"
                  :style="{
                    background: selectedCommand === cmd.name ? 'var(--bg-secondary)' : 'transparent',
                  }"
                  @click="setSelectedCommand(cmd.name)"
                >
                  <div 
                    class="absolute left-0 top-0 bottom-0 w-1 transition-all"
                    :style="{ 
                      background: selectedCommand === cmd.name ? 'var(--accent-primary)' : 'transparent',
                      opacity: selectedCommand === cmd.name ? 1 : 0
                    }"
                  />
                  <div class="flex items-center justify-between">
                    <span 
                      class="font-mono text-sm font-semibold"
                      :style="{ color: selectedCommand === cmd.name ? 'var(--accent-primary)' : 'var(--text-primary)' }"
                    >
                      {{ cmd.name }}
                    </span>
                    <ChevronRight 
                      v-if="selectedCommand === cmd.name"
                      class="w-4 h-4"
                      :style="{ color: 'var(--accent-primary)' }"
                    />
                  </div>
                  <p 
                    class="text-xs mt-1 line-clamp-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ cmd.description }}
                  </p>
                </button>
              </div>
            </section>
          </aside>

          <!-- 右侧：执行区域 -->
          <main class="flex flex-col gap-6 min-w-0">
            <!-- 命令详情与输入 -->
            <section
              class="rounded-xl p-6 glass-effect"
              :style="{
                border: '1px solid var(--border-color)',
                boxShadow: 'var(--shadow-small)'
              }"
            >
              <!-- 头部信息 -->
              <div class="mb-6">
                <div class="flex items-center gap-3 mb-2">
                  <div 
                    class="p-2 rounded-lg"
                    :style="{ background: 'var(--bg-tertiary)' }"
                  >
                    <component
                      :is="currentClientInfo?.icon"
                      class="w-6 h-6"
                      :style="{ color: 'var(--text-primary)' }"
                    />
                  </div>
                  <div>
                    <h1
                      class="text-2xl font-bold"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      {{ selectedCommandInfo?.name || 'Select a command' }}
                    </h1>
                    <p
                      class="text-sm"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ selectedCommandInfo?.description }}
                    </p>
                  </div>
                </div>
              </div>

              <!-- 终端输入框 -->
              <div 
                class="rounded-lg p-4 font-mono text-sm transition-all"
                :style="{ 
                  background: '#1e1e1e',
                  border: '1px solid var(--border-color)',
                  boxShadow: 'inset 0 2px 4px rgba(0,0,0,0.2)'
                }"
              >
                <div class="flex items-center gap-2 mb-2 text-xs opacity-50 select-none">
                  <Terminal class="w-3 h-3" />
                  <span>COMMAND INPUT</span>
                </div>
                <div class="flex items-center gap-3">
                  <span
                    class="font-bold select-none"
                    :style="{ color: 'var(--accent-success)' }"
                  >➜</span>
                  <span
                    class="font-bold select-none"
                    :style="{ color: 'var(--accent-info)' }"
                  >~</span>
                  <span
                    class="font-bold select-none"
                    :style="{ color: 'var(--text-primary)' }"
                  >{{ selectedClient }}</span>
                  <span
                    class="font-bold select-none"
                    :style="{ color: 'var(--accent-warning)' }"
                  >{{ selectedCommand }}</span>
                  
                  <input
                    v-model="args"
                    type="text"
                    :placeholder="$t('commands.argsPlaceholder')"
                    class="flex-1 bg-transparent border-none outline-none font-mono"
                    :style="{ color: 'var(--text-primary)' }"
                    @keydown.enter="!loading && handleExecute()"
                  >
                </div>
              </div>

              <!-- 执行按钮 -->
              <div class="mt-4 flex justify-end">
                <button
                  class="px-8 py-2.5 rounded-lg font-semibold text-sm text-white transition-all hover:scale-[1.02] active:scale-[0.98] flex items-center gap-2"
                  :style="{
                    background: loading
                      ? 'var(--bg-tertiary)'
                      : 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                    boxShadow: loading ? 'none' : '0 4px 12px var(--glow-primary)',
                    opacity: loading ? 0.7 : 1,
                    cursor: loading ? 'not-allowed' : 'pointer'
                  }"
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
            </section>

            <!-- 输出区域 -->
            <section
              v-if="output || loading"
              class="flex-1 rounded-xl overflow-hidden flex flex-col min-h-[400px]"
              :style="{
                background: '#1e1e1e',
                border: '1px solid var(--border-color)',
                boxShadow: 'var(--shadow-medium)'
              }"
            >
              <!-- 终端头部 -->
              <div 
                class="flex items-center justify-between px-4 py-2"
                :style="{ background: '#2d2d2d', borderBottom: '1px solid #333' }"
              >
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
                    @click="handleCopyOutput"
                  >
                    <Copy class="w-3.5 h-3.5" />
                  </button>
                  <button
                    class="p-1.5 rounded hover:bg-[#3e3e3e] transition-colors text-gray-400 hover:text-white"
                    :title="$t('commands.clearOutputButton')"
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
                    <Loader2 class="w-8 h-8 text-[var(--accent-primary)] animate-spin" />
                    <span class="text-gray-400 text-xs animate-pulse">Processing command...</span>
                  </div>
                </div>

                <template v-if="output">
                  <!-- 命令行回显 -->
                  <div class="flex items-center gap-2 mb-4 opacity-50">
                    <span class="text-[var(--accent-success)]">➜</span>
                    <span class="text-[var(--accent-info)]">~</span>
                    <span class="text-gray-300">{{ selectedClient }} {{ selectedCommand }} {{ args }}</span>
                  </div>

                  <!-- 实际输出 -->
                  <pre 
                    class="whitespace-pre-wrap break-words leading-relaxed"
                    :style="{ color: output.success ? '#e2e8f0' : '#fca5a5' }"
                  >{{ output.output || output.error }}</pre>

                  <!-- 状态行 -->
                  <div class="mt-6 pt-4 border-t border-gray-800 flex items-center gap-4 text-xs font-mono">
                    <div class="flex items-center gap-2">
                      <span class="text-gray-500">Status:</span>
                      <span :class="output.success ? 'text-[var(--accent-success)]' : 'text-[var(--accent-danger)]'">
                        {{ output.success ? 'SUCCESS' : 'FAILED' }}
                      </span>
                    </div>
                    <div class="flex items-center gap-2">
                      <span class="text-gray-500">Code:</span>
                      <span :class="output.exit_code === 0 ? 'text-gray-300' : 'text-[var(--accent-danger)]'">
                        {{ output.exit_code }}
                      </span>
                    </div>
                    <div class="flex items-center gap-2">
                      <span class="text-gray-500">Time:</span>
                      <span class="text-[var(--accent-primary)]">{{ output.duration_ms }}ms</span>
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
            </section>
          </main>
        </div>
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
  ChevronRight, Loader2 
} from 'lucide-vue-next'
import { listCommands, executeCommand } from '@/api/client'
import type { CommandInfo, CommandResponse } from '@/types'
import Navbar from '@/components/Navbar.vue'

type CliClient = 'ccr' | 'qwen' | 'gemini-cli' | 'iflow'

const { t } = useI18n({ useScope: 'global' })

const CLI_CLIENTS = [
  { id: 'ccr' as CliClient, name: 'CCR', icon: Zap, color: 'rgba(139, 92, 246, 0.2)' },
  { id: 'qwen' as CliClient, name: 'Qwen', icon: Sparkles, color: 'rgba(251, 191, 36, 0.2)' },
  { id: 'gemini-cli' as CliClient, name: 'Gemini', icon: Gem, color: 'rgba(59, 130, 246, 0.2)' },
  { id: 'iflow' as CliClient, name: 'IFLOW', icon: Workflow, color: 'rgba(168, 85, 247, 0.2)' }
]

const selectedClient = ref<CliClient>('ccr')
const commands = ref<CommandInfo[]>([])
const selectedCommand = ref<string>('')
const args = ref<string>('')
const output = ref<CommandResponse | null>(null)
const loading = ref(false)

const selectedCommandInfo = computed(() =>
  commands.value.find((c) => c.name === selectedCommand.value)
)

const currentClientInfo = computed(() =>
  CLI_CLIENTS.find((c) => c.id === selectedClient.value)
)

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
          usage: `${selectedClient.value} help`,
          examples: [`${selectedClient.value} help`]
        },
        {
          name: 'version',
          description: t('commands.versionDescription', { client: clientName }),
          usage: `${selectedClient.value} --version`,
          examples: [`${selectedClient.value} --version`]
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
    if (selectedClient.value === 'ccr') {
      const argsArray = args.value
        .split(' ')
        .map((a) => a.trim())
        .filter((a) => a.length > 0)

      const result = await executeCommand({
        command: selectedCommand.value,
        args: argsArray
      })

      output.value = result
    } else {
      // Mock execution for other clients
      await new Promise(resolve => setTimeout(resolve, 800))
      const clientName = CLI_CLIENTS.find((c) => c.id === selectedClient.value)?.name || selectedClient.value
      output.value = {
        success: true,
        output: `${t('commands.developingFeature', { client: clientName })}\n\n${t('commands.developingMessage', { client: selectedClient.value })}`,
        error: '',
        exit_code: 0,
        duration_ms: 120
      }
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
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
