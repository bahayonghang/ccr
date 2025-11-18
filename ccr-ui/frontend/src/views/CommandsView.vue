<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="max-w-[1800px] mx-auto">
      <!-- 导航栏 -->
      <Navbar />

      <!-- 状态信息头部 -->
      <StatusHeader
        :current-config="currentConfig"
        :total-configs="totalConfigs"
        :history-count="historyCount"
      />

      <!-- 布局：可折叠侧边栏 + 主命令区域 -->
      <div class="grid grid-cols-[auto_1fr] gap-4">
        <!-- 可折叠导航 -->
        <CollapsibleSidebar module="commands" />

        <!-- 主命令区域 -->
        <div class="space-y-4">
          <!-- CLI 客户端选择 -->
          <section
            class="rounded-xl p-5 glass-effect"
            :style="{
              border: '1px solid var(--border-color)',
              boxShadow: 'var(--shadow-small)'
            }"
          >
            <h2
              class="text-xs font-semibold uppercase tracking-wider mb-3"
              :style="{ color: 'var(--text-secondary)' }"
            >
              选择Cli命令行工具
            </h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
              <button
                v-for="client in CLI_CLIENTS"
                :key="client.id"
                class="p-4 rounded-lg transition-all hover:scale-[1.02]"
                :class="selectedClient === client.id ? 'ring-2 shadow-lg' : ''"
                :style="{
                  background: selectedClient === client.id
                    ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                    : 'var(--bg-tertiary)',
                  border: `1px solid ${selectedClient === client.id ? 'var(--accent-primary)' : 'var(--border-color)'}`,
                  color: selectedClient === client.id ? 'white' : 'var(--text-primary)',
                  boxShadow: selectedClient === client.id ? '0 0 20px var(--glow-primary)' : undefined
                }"
                @click="setSelectedClient(client.id)"
              >
                <div class="flex flex-col items-center gap-2">
                  <component
                    :is="client.icon"
                    class="w-6 h-6"
                  />
                  <span class="text-sm font-semibold">{{ client.name }}</span>
                </div>
              </button>
            </div>
          </section>

          <!-- 命令列表和执行区域 -->
          <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
            <!-- 命令列表 -->
            <aside
              class="lg:col-span-1 rounded-xl p-5 glass-effect"
              :style="{
                border: '1px solid var(--border-color)',
                boxShadow: 'var(--shadow-small)'
              }"
            >
              <h2
                class="text-xs font-semibold uppercase tracking-wider mb-4"
                :style="{ color: 'var(--text-secondary)' }"
              >
                可用命令
              </h2>
              <nav
                class="space-y-2"
                aria-label="命令列表"
              >
                <button
                  v-for="cmd in commands"
                  :key="cmd.name"
                  class="w-full text-left px-4 py-3 rounded-lg transition-all hover:scale-[1.02]"
                  :class="selectedCommand === cmd.name ? 'text-white shadow-lg' : ''"
                  :style="{
                    background: selectedCommand === cmd.name
                      ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                      : 'var(--bg-tertiary)',
                    border: `1px solid ${selectedCommand === cmd.name ? 'var(--accent-primary)' : 'var(--border-color)'}`,
                    boxShadow: selectedCommand === cmd.name ? '0 0 15px var(--glow-primary)' : undefined
                  }"
                  :aria-pressed="selectedCommand === cmd.name"
                  @click="setSelectedCommand(cmd.name)"
                >
                  <div class="font-mono font-bold text-sm">
                    {{ cmd.name }}
                  </div>
                  <div
                    class="text-xs mt-1 leading-relaxed"
                    :style="{
                      color: selectedCommand === cmd.name ? 'rgba(255,255,255,0.9)' : 'var(--text-secondary)'
                    }"
                  >
                    {{ cmd.description }}
                  </div>
                </button>
              </nav>
            </aside>

            <!-- 命令执行面板 -->
            <main class="lg:col-span-2 space-y-4">
              <!-- 命令信息 -->
              <section
                v-if="selectedCommandInfo"
                class="rounded-xl p-5 glass-effect"
                :style="{
                  border: '1px solid var(--border-color)',
                  boxShadow: 'var(--shadow-small)'
                }"
              >
                <div class="flex items-center gap-2 mb-2">
                  <component
                    :is="currentClientInfo.icon"
                    v-if="currentClientInfo"
                    class="w-5 h-5"
                  />
                  <h1
                    class="text-xl font-bold"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ selectedCommandInfo.name }}
                  </h1>
                  <span
                    class="px-2 py-1 rounded text-xs font-semibold"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      color: 'var(--text-muted)'
                    }"
                  >
                    {{ currentClientInfo?.name }}
                  </span>
                </div>
                <p
                  class="mb-4"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ selectedCommandInfo.description }}
                </p>
                <div class="space-y-3">
                  <div>
                    <span
                      class="text-sm font-semibold"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      Usage:
                    </span>
                    <code
                      class="ml-2 text-sm px-3 py-1 rounded font-mono"
                      :style="{ background: 'var(--bg-tertiary)', color: 'var(--accent-primary)' }"
                    >
                      {{ selectedCommandInfo.usage }}
                    </code>
                  </div>
                  <div>
                    <span
                      class="text-sm font-semibold"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      Examples:
                    </span>
                    <ul class="mt-2 space-y-2">
                      <li
                        v-for="(example, idx) in selectedCommandInfo.examples"
                        :key="idx"
                      >
                        <code
                          class="text-sm px-3 py-2 rounded font-mono block"
                          :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)' }"
                        >
                          {{ example }}
                        </code>
                      </li>
                    </ul>
                  </div>
                </div>
              </section>

              <!-- 命令输入 -->
              <section
                class="rounded-xl p-5 glass-effect"
                :style="{
                  border: '1px solid var(--border-color)',
                  boxShadow: 'var(--shadow-small)'
                }"
              >
                <h3
                  class="text-xs font-semibold uppercase tracking-wider mb-3"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  参数 (可选)
                </h3>
                <label
                  for="command-args"
                  class="sr-only"
                >命令参数</label>
                <input
                  id="command-args"
                  v-model="args"
                  type="text"
                  placeholder="例如: --help 或 anthropic"
                  class="w-full px-4 py-2.5 rounded-lg font-mono text-sm focus:outline-none transition-all"
                  :style="{
                    background: 'var(--bg-tertiary)',
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-primary)'
                  }"
                  @keydown.enter="!loading && handleExecute()"
                >
                <button
                  class="mt-3 w-full px-6 py-3 rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2 font-semibold text-sm text-white hover:scale-[1.02]"
                  :style="{
                    background: loading
                      ? 'var(--bg-tertiary)'
                      : 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                    boxShadow: loading ? 'none' : '0 0 20px var(--glow-primary)'
                  }"
                  :disabled="loading"
                  :aria-label="loading ? '执行中...' : '执行命令'"
                  @click="handleExecute"
                >
                  <div
                    v-if="loading"
                    class="w-4 h-4 rounded-full border-2 border-white border-t-transparent animate-spin"
                    aria-hidden="true"
                  />
                  <Play
                    v-else
                    class="w-4 h-4"
                    aria-hidden="true"
                  />
                  <span>{{ loading ? '执行中...' : '执行命令' }}</span>
                </button>
              </section>

              <!-- 命令输出 -->
              <section
                v-if="output"
                class="rounded-xl overflow-hidden"
                :style="{
                  border: output.success
                    ? '2px solid rgba(34, 197, 94, 0.3)'
                    : '2px solid rgba(239, 68, 68, 0.3)',
                  boxShadow: output.success
                    ? '0 0 30px rgba(34, 197, 94, 0.15)'
                    : '0 0 30px rgba(239, 68, 68, 0.15)'
                }"
                aria-live="polite"
                aria-atomic="true"
              >
                <!-- 头部 -->
                <header
                  class="px-5 py-4"
                  :style="{
                    background: output.success
                      ? 'linear-gradient(135deg, rgba(34, 197, 94, 0.1), rgba(16, 185, 129, 0.05))'
                      : 'linear-gradient(135deg, rgba(239, 68, 68, 0.1), rgba(220, 38, 38, 0.05))',
                    borderBottom: `1px solid ${output.success ? 'rgba(34, 197, 94, 0.2)' : 'rgba(239, 68, 68, 0.2)'}`
                  }"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                      <!-- 状态图标 -->
                      <div
                        class="flex items-center gap-2 px-3 py-1.5 rounded-lg"
                        :style="{
                          background: output.success
                            ? 'rgba(34, 197, 94, 0.15)'
                            : 'rgba(239, 68, 68, 0.15)'
                        }"
                      >
                        <Check
                          v-if="output.success"
                          class="w-4 h-4"
                          :style="{ color: '#22c55e' }"
                        />
                        <X
                          v-else
                          class="w-4 h-4"
                          :style="{ color: '#ef4444' }"
                        />
                        <span
                          class="text-sm font-bold"
                          :style="{ color: output.success ? '#22c55e' : '#ef4444' }"
                        >
                          {{ output.success ? '执行成功' : '执行失败' }}
                        </span>
                      </div>

                      <!-- 退出码 -->
                      <div class="flex items-center gap-2">
                        <span
                          class="text-xs font-semibold"
                          :style="{ color: 'var(--text-muted)' }"
                        >
                          Exit Code:
                        </span>
                        <span
                          class="px-2 py-0.5 rounded font-mono text-xs font-bold"
                          :style="{
                            background: output.exit_code === 0 ? 'rgba(34, 197, 94, 0.2)' : 'rgba(239, 68, 68, 0.2)',
                            color: output.exit_code === 0 ? '#22c55e' : '#ef4444'
                          }"
                        >
                          {{ output.exit_code }}
                        </span>
                      </div>

                      <!-- 执行时间 -->
                      <div class="flex items-center gap-2">
                        <span
                          class="text-xs font-semibold"
                          :style="{ color: 'var(--text-muted)' }"
                        >
                          Duration:
                        </span>
                        <span
                          class="px-2 py-0.5 rounded font-mono text-xs font-bold"
                          :style="{
                            background: 'rgba(139, 92, 246, 0.15)',
                            color: 'var(--accent-primary)'
                          }"
                        >
                          {{ output.duration_ms }}ms
                        </span>
                      </div>
                    </div>

                    <!-- 操作按钮 -->
                    <div class="flex space-x-2">
                      <button
                        class="p-2 rounded-lg transition-all hover:scale-110"
                        :style="{
                          background: 'var(--bg-tertiary)',
                          border: '1px solid var(--border-color)',
                          color: 'var(--accent-primary)'
                        }"
                        title="复制输出"
                        aria-label="复制输出"
                        @click="handleCopyOutput"
                      >
                        <Copy class="w-4 h-4" />
                      </button>
                      <button
                        class="p-2 rounded-lg transition-all hover:scale-110"
                        :style="{
                          background: 'var(--bg-tertiary)',
                          border: '1px solid var(--border-color)',
                          color: 'var(--accent-danger)'
                        }"
                        title="清除输出"
                        aria-label="清除输出"
                        @click="handleClearOutput"
                      >
                        <Trash2 class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </header>

                <!-- 输出内容 - Terminal 风格 -->
                <div
                  class="terminal-output p-5 font-mono text-sm leading-relaxed"
                  :style="{
                    background: '#0f172a',
                    color: '#e2e8f0',
                    minHeight: '200px',
                    maxHeight: '500px',
                    overflowY: 'auto'
                  }"
                >
                  <div
                    v-if="output.output"
                    class="space-y-1 whitespace-pre-wrap"
                  >
                    {{ output.output }}
                  </div>

                  <div
                    v-if="output.error"
                    class="mt-4 p-4 rounded-lg border-l-4"
                    :style="{
                      background: 'rgba(239, 68, 68, 0.1)',
                      borderColor: '#ef4444'
                    }"
                    role="alert"
                  >
                    <div class="flex items-start gap-3">
                      <X
                        class="w-5 h-5 flex-shrink-0 mt-0.5"
                        :style="{ color: '#ef4444' }"
                      />
                      <div class="flex-1">
                        <p
                          class="text-sm font-bold mb-1"
                          :style="{ color: '#ef4444' }"
                        >
                          Error
                        </p>
                        <pre
                          class="text-sm whitespace-pre-wrap break-words"
                          :style="{ color: '#fca5a5' }"
                        >{{ output.error }}</pre>
                      </div>
                    </div>
                  </div>

                  <div
                    v-if="!output.output && !output.error"
                    class="flex items-center justify-center h-32"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    <p class="text-sm">
                      无输出内容
                    </p>
                  </div>
                </div>

                <!-- 底部状态栏 -->
                <div
                  class="px-5 py-2 flex items-center justify-between text-xs"
                  :style="{
                    background: 'var(--bg-tertiary)',
                    borderTop: '1px solid var(--border-color)'
                  }"
                >
                  <span :style="{ color: 'var(--text-muted)' }">Terminal Output</span>
                  <span :style="{ color: 'var(--text-muted)' }">
                    {{ output.output?.split('\n').length || 0 }} lines
                  </span>
                </div>
              </section>
            </main>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { Zap, Sparkles, Gem, Workflow, Play, Copy, Trash2, Check, X } from 'lucide-vue-next'
import { listCommands, executeCommand, listConfigs, getHistory } from '@/api/client'
import type { CommandInfo, CommandResponse } from '@/types'
import Navbar from '@/components/Navbar.vue'
import StatusHeader from '@/components/StatusHeader.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'

type CliClient = 'ccr' | 'qwen' | 'gemini-cli' | 'iflow'

const CLI_CLIENTS = [
  { id: 'ccr' as CliClient, name: 'CCR', icon: Zap, color: 'rgba(139, 92, 246, 0.2)' },
  { id: 'qwen' as CliClient, name: 'Qwen', icon: Sparkles, color: 'rgba(251, 191, 36, 0.2)' },
  { id: 'gemini-cli' as CliClient, name: 'Gemini Cli', icon: Gem, color: 'rgba(59, 130, 246, 0.2)' },
  { id: 'iflow' as CliClient, name: 'IFLOW', icon: Workflow, color: 'rgba(168, 85, 247, 0.2)' }
]

const selectedClient = ref<CliClient>('ccr')
const commands = ref<CommandInfo[]>([])
const selectedCommand = ref<string>('')
const args = ref<string>('')
const output = ref<CommandResponse | null>(null)
const loading = ref(false)
const currentConfig = ref<string>('')
const totalConfigs = ref(0)
const historyCount = ref(0)

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
      if (data.length > 0) {
        selectedCommand.value = data[0].name
      }
    } else {
      commands.value = [
        {
          name: 'help',
          description: `${CLI_CLIENTS.find((c) => c.id === selectedClient.value)?.name} 帮助命令 (功能开发中)`,
          usage: `${selectedClient.value} help`,
          examples: [`${selectedClient.value} help`]
        },
        {
          name: 'version',
          description: `显示 ${CLI_CLIENTS.find((c) => c.id === selectedClient.value)?.name} 版本`,
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

const loadStats = async () => {
  try {
    const configData = await listConfigs()
    currentConfig.value = configData.current_config
    totalConfigs.value = configData.configs.length

    const historyData = await getHistory()
    historyCount.value = historyData.total
  } catch (err) {
    console.error('Failed to load stats:', err)
  }
}

onMounted(() => {
  loadCommands()
  loadStats()
})

watch(selectedClient, () => {
  loadCommands()
  output.value = null
})

const setSelectedClient = (client: CliClient) => {
  selectedClient.value = client
}

const setSelectedCommand = (cmd: string) => {
  selectedCommand.value = cmd
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
      output.value = {
        success: true,
        output: `${CLI_CLIENTS.find((c) => c.id === selectedClient.value)?.name} 命令执行功能正在开发中...\n\n该功能将支持执行 ${selectedClient.value} 相关命令。`,
        error: '',
        exit_code: 0,
        duration_ms: 100
      }
    }
  } catch (err) {
    output.value = {
      success: false,
      output: '',
      error: err instanceof Error ? err.message : 'Unknown error',
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
    alert('输出已复制到剪贴板！')
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

const handleClearOutput = () => {
  output.value = null
}
</script>

<style scoped>
.terminal-output {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Courier New', monospace;
}
</style>
