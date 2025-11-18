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
                配置转换器
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
                <span>返回首页</span>
              </RouterLink>
            </div>
            <p :style="{ color: 'var(--text-muted)' }">
              支持多种 CLI 配置格式之间的互相转换，包括 Claude Code、Codex、Gemini、Qwen、iFlow 等
            </p>
          </div>

          <!-- Error/Success Messages -->
          <div
            v-if="error"
            class="rounded-lg p-4 flex items-start gap-3"
            :style="{ background: 'rgba(239, 68, 68, 0.1)', border: '1px solid rgb(239, 68, 68)' }"
          >
            <AlertCircle
              class="w-5 h-5 flex-shrink-0 mt-0.5"
              :style="{ color: 'rgb(239, 68, 68)' }"
            />
            <div :style="{ color: 'rgb(239, 68, 68)' }">
              {{ error }}
            </div>
          </div>

          <div
            v-if="successMessage"
            class="rounded-lg p-4 flex items-start gap-3"
            :style="{ background: 'rgba(34, 197, 94, 0.1)', border: '1px solid rgb(34, 197, 94)' }"
          >
            <Check
              class="w-5 h-5 flex-shrink-0 mt-0.5"
              :style="{ color: 'rgb(34, 197, 94)' }"
            />
            <div :style="{ color: 'rgb(34, 197, 94)' }">
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
                  源格式
                </h2>
              </div>
              <p
                class="mb-4"
                :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
              >
                选择要转换的配置格式
              </p>

              <div class="space-y-2">
                <div
                  v-for="type in CLI_TYPES"
                  :key="type.value"
                  class="p-4 rounded-lg cursor-pointer transition-all"
                  :style="{
                    border:
                      sourceFormat === type.value
                        ? '2px solid var(--accent-primary)'
                        : '1px solid var(--border-color)',
                    background:
                      sourceFormat === type.value
                        ? 'rgba(139, 92, 246, 0.1)'
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
                      已选择
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
                  目标格式
                </h2>
              </div>
              <p
                class="mb-4"
                :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
              >
                选择转换后的配置格式
              </p>

              <div class="space-y-2">
                <div
                  v-for="type in CLI_TYPES"
                  :key="type.value"
                  class="p-4 rounded-lg cursor-pointer transition-all"
                  :style="{
                    border:
                      targetFormat === type.value && sourceFormat !== type.value
                        ? '2px solid var(--accent-secondary)'
                        : '1px solid var(--border-color)',
                    background:
                      targetFormat === type.value && sourceFormat !== type.value
                        ? 'rgba(168, 85, 247, 0.1)'
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
                      已选择
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
              转换选项
            </h2>
            <p
              class="mb-4"
              :style="{ color: 'var(--text-muted)', fontSize: '14px' }"
            >
              选择要转换的配置项
            </p>

            <div class="flex flex-wrap gap-6">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="convertMcp"
                  type="checkbox"
                  class="w-4 h-4 cursor-pointer"
                >
                <span :style="{ color: 'var(--text-secondary)' }">MCP 服务器配置</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="convertCommands"
                  type="checkbox"
                  class="w-4 h-4 cursor-pointer"
                >
                <span :style="{ color: 'var(--text-secondary)' }">Slash 命令配置</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="convertAgents"
                  type="checkbox"
                  class="w-4 h-4 cursor-pointer"
                >
                <span :style="{ color: 'var(--text-secondary)' }">Agents 配置</span>
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
                  配置输入
                </h2>
                <p :style="{ color: 'var(--text-muted)', fontSize: '14px' }">
                  粘贴配置内容或上传配置文件
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
                  加载示例
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
                    上传文件
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
              placeholder="在此粘贴配置内容，或点击上传文件按钮..."
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
              支持的文件格式: JSON, TOML, YAML, TXT
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
              {{ isConverting ? '转换中...' : '开始转换' }}
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
                转换统计
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
                    MCP 服务器
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
                    Slash 命令
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
                    Agents
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
                    Profiles
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
                    基础配置
                  </div>
                </div>
              </div>

              <div
                v-if="result.warnings && result.warnings.length > 0"
                class="rounded-lg p-4"
                :style="{ background: 'rgba(234, 179, 8, 0.1)', border: '1px solid rgb(234, 179, 8)' }"
              >
                <div
                  class="font-medium mb-2"
                  :style="{ color: 'rgb(234, 179, 8)' }"
                >
                  转换警告:
                </div>
                <ul class="list-disc list-inside space-y-1">
                  <li
                    v-for="(warning, index) in result.warnings"
                    :key="index"
                    class="text-sm"
                    :style="{ color: 'rgb(234, 179, 8)' }"
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
                    转换结果
                  </h2>
                  <p :style="{ color: 'var(--text-muted)', fontSize: '14px' }">
                    格式: {{ result.format?.toUpperCase() }}
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
                    复制
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
                    下载
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
              使用说明
            </h2>

            <div class="space-y-4">
              <div>
                <h4
                  class="font-medium mb-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  支持的转换路径
                </h4>
                <ul
                  class="list-disc list-inside space-y-1 text-sm"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  <li>Claude Code ↔ Codex (完全支持)</li>
                  <li>其他格式转换功能正在开发中...</li>
                </ul>
              </div>
              <div>
                <h4
                  class="font-medium mb-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  转换说明
                </h4>
                <ul
                  class="list-disc list-inside space-y-1 text-sm"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  <li>Claude Code 使用 JSON 格式 (settings.json)</li>
                  <li>Codex 使用 TOML 格式 (config.toml)</li>
                  <li>转换过程会保留所有支持的配置项</li>
                  <li>不支持的配置项会在警告中显示</li>
                </ul>
              </div>
              <div>
                <h4
                  class="font-medium mb-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  注意事项
                </h4>
                <ul
                  class="list-disc list-inside space-y-1 text-sm"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  <li>转换前请备份原始配置文件</li>
                  <li>API 密钥等敏感信息需要手动填写</li>
                  <li>建议转换后进行验证测试</li>
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
import { ref } from 'vue'
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

const CLI_TYPES: { value: CliType; label: string; description: string }[] = [
  { value: 'claude-code', label: 'Claude Code', description: 'Claude Code CLI configuration (JSON)' },
  { value: 'codex', label: 'Codex', description: 'OpenAI Codex CLI configuration (TOML)' },
  { value: 'gemini', label: 'Gemini', description: 'Google Gemini CLI configuration' },
  { value: 'qwen', label: 'Qwen', description: 'Alibaba Qwen CLI configuration' },
  { value: 'iflow', label: 'iFlow', description: 'iFlow CLI configuration' }
]

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
      successMessage.value = `已加载文件: ${file.name}`
      setTimeout(() => (successMessage.value = null), 3000)
    }
    reader.onerror = () => {
      error.value = '读取文件失败'
    }
    reader.readAsText(file)
  }
}

const handleConvert = async () => {
  error.value = null
  successMessage.value = null
  result.value = null

  if (!configData.value.trim()) {
    error.value = '请输入或上传配置内容'
    return
  }

  if (sourceFormat.value === targetFormat.value) {
    error.value = '源格式和目标格式不能相同'
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
    successMessage.value = '配置转换成功！'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '转换失败'
  } finally {
    isConverting.value = false
  }
}

const handleCopyResult = () => {
  if (result.value?.converted_data) {
    navigator.clipboard.writeText(result.value.converted_data)
    successMessage.value = '已复制到剪贴板'
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
    const sourceLabel =
      CLI_TYPES.find((t) => t.value === sourceFormat.value)?.label || sourceFormat.value
    const targetLabel =
      CLI_TYPES.find((t) => t.value === targetFormat.value)?.label || targetFormat.value
    a.download = `${sourceLabel}-to-${targetLabel}.${extension}`

    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)

    successMessage.value = '文件已下载'
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
  successMessage.value = '已加载示例配置'
  setTimeout(() => (successMessage.value = null), 3000)
}
</script>
