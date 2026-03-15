<!-- -->
<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #8b5cf6 0%, #ec4899 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #10b981 0%, #3b82f6 100%)',
          animationDelay: '1s'
        }"
      />
    </div>

    <div class="relative z-10 p-6 max-w-7xl mx-auto">
      <!-- 页面标题 -->
      <div class="mb-8">
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center gap-4">
            <RouterLink
              to="/droid"
              class="p-3 rounded-2xl glass-card hover:scale-105 transition-transform duration-300"
              :style="{ background: 'rgba(139, 92, 246, 0.1)' }"
            >
              <SIcon
                name="ArrowLeft"
                size="w-6 h-6"
                :style="{ color: '#8b5cf6' }"
              />
            </RouterLink>
            <div>
              <h1
                class="text-3xl md:text-4xl font-bold mb-2 bg-gradient-to-r from-[#8b5cf6] via-[#ec4899] to-[#f59e0b] bg-clip-text text-transparent"
              >
                Droids 管理
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                管理自定义 AI Subagents (Droids)
              </p>
            </div>
          </div>
          <button
            class="glass-card flex items-center gap-2 px-5 py-3 hover:scale-105 transition-transform duration-300"
            :style="{ background: 'rgba(139, 92, 246, 0.1)', color: '#8b5cf6' }"
            @click="showAddModal = true"
          >
            <SIcon
              name="Plus"
              size="w-5 h-5"
            />
            <span class="font-medium">添加 Droid</span>
          </button>
        </div>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading"
        class="flex justify-center items-center py-20"
      >
        <div
          class="animate-spin rounded-full h-12 w-12 border-b-2"
          :style="{ borderColor: '#8b5cf6' }"
        />
      </div>

      <!-- Droids 列表 -->
      <div
        v-else-if="droids.length > 0"
        class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
      >
        <div
          v-for="droid in droids"
          :key="droid.name"
          class="glass-card p-6 hover:scale-105 transition-transform duration-300"
        >
          <!-- Droid 头部 -->
          <div class="flex items-start justify-between mb-4">
            <div class="flex-1">
              <h3
                class="text-xl font-bold mb-1"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ droid.name }}
              </h3>
              <p
                v-if="droid.description"
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ droid.description }}
              </p>
            </div>
            <div class="flex gap-2">
              <button
                class="p-2 rounded-lg hover:bg-blue-500/10 transition-colors"
                :style="{ color: '#3b82f6' }"
                @click="editDroid(droid)"
              >
                <SIcon
                  name="Edit2"
                  size="w-4 h-4"
                />
              </button>
              <button
                class="p-2 rounded-lg hover:bg-red-500/10 transition-colors"
                :style="{ color: '#ef4444' }"
                @click="deleteDroid(droid.name)"
              >
                <SIcon
                  name="Trash2"
                  size="w-4 h-4"
                />
              </button>
            </div>
          </div>

          <!-- Droid 信息 -->
          <div class="space-y-2 mb-4">
            <div class="flex items-center gap-2">
              <SIcon
                name="Cpu"
                size="w-4 h-4"
                :style="{ color: '#64748b' }"
              />
              <span
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ droid.model }}
              </span>
            </div>
            <div
              v-if="droid.reasoningEffort"
              class="flex items-center gap-2"
            >
              <SIcon
                name="Zap"
                size="w-4 h-4"
                :style="{ color: '#64748b' }"
              />
              <span
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                Reasoning: {{ droid.reasoningEffort }}
              </span>
            </div>
            <div
              v-if="droid.tools"
              class="flex items-center gap-2"
            >
              <SIcon
                name="Wrench"
                size="w-4 h-4"
                :style="{ color: '#64748b' }"
              />
              <span
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                Tools: {{ formatTools(droid.tools) }}
              </span>
            </div>
          </div>

          <!-- 系统提示预览 -->
          <div
            class="mt-4 p-3 rounded-lg"
            :style="{ background: 'rgba(139, 92, 246, 0.05)' }"
          >
            <p
              class="text-xs font-mono line-clamp-3"
              :style="{ color: 'var(--text-secondary)' }"
            >
              {{ droid.systemPrompt }}
            </p>
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div
        v-else
        class="text-center py-20"
      >
        <div
          class="inline-block p-6 rounded-3xl glass-card mb-6"
          :style="{ background: 'rgba(139, 92, 246, 0.1)' }"
        >
          <SIcon
            name="Inbox"
            size="w-16 h-16"
            :style="{ color: '#8b5cf6' }"
          />
        </div>
        <h3
          class="text-2xl font-bold mb-2"
          :style="{ color: 'var(--text-primary)' }"
        >
          还没有 Droid
        </h3>
        <p
          class="text-lg mb-6"
          :style="{ color: 'var(--text-secondary)' }"
        >
          点击"添加 Droid"按钮创建第一个自定义 Subagent
        </p>
      </div>

      <!-- 添加/编辑 Droid 弹窗 -->
      <div
        v-if="showAddModal"
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
        @click.self="closeModal"
      >
        <div class="glass-card p-6 max-w-4xl w-full max-h-[90vh] overflow-y-auto">
          <div class="flex items-center justify-between mb-6">
            <h2
              class="text-2xl font-bold"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ editingDroid ? '编辑 Droid' : '添加 Droid' }}
            </h2>
            <button
              class="p-2 hover:bg-gray-500/10 rounded-lg transition-colors"
              @click="closeModal"
            >
              <SIcon
                name="X"
                size="w-5 h-5"
                :style="{ color: 'var(--text-secondary)' }"
              />
            </button>
          </div>

          <form
            class="space-y-4"
            @submit.prevent="saveDroid"
          >
            <!-- Droid 名称 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                Droid 名称 * (小写字母、数字、-、_)
              </label>
              <input
                v-model="formData.name"
                type="text"
                required
                :disabled="!!editingDroid"
                placeholder="code-reviewer"
                pattern="[a-z0-9_-]+"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
            </div>

            <!-- 描述 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                描述 (最多500字符)
              </label>
              <input
                v-model="formData.description"
                type="text"
                maxlength="500"
                placeholder="Reviews diffs for correctness, tests, and migration fallout"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
            </div>

            <!-- 模型 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                模型 *
              </label>
              <input
                v-model="formData.model"
                type="text"
                required
                placeholder="inherit (或模型ID，如 claude-sonnet-4-5)"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
              <p
                class="text-xs mt-1"
                :style="{ color: 'var(--text-secondary)' }"
              >
                使用 "inherit" 继承主模型，或指定具体模型ID
              </p>
            </div>

            <!-- 推理努力程度 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                推理努力程度
              </label>
              <select
                v-model="formData.reasoningEffort"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
                <option value="">
                  不指定
                </option>
                <option value="low">
                  Low
                </option>
                <option value="medium">
                  Medium
                </option>
                <option value="high">
                  High
                </option>
              </select>
            </div>

            <!-- 工具 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                工具配置
              </label>
              <select
                v-model="toolsMode"
                class="w-full px-4 py-2 rounded-lg glass-card mb-2"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
                <option value="all">
                  所有工具
                </option>
                <option value="category">
                  工具类别
                </option>
                <option value="custom">
                  自定义工具列表
                </option>
              </select>

              <!-- 工具类别选择 -->
              <select
                v-if="toolsMode === 'category'"
                v-model="toolsCategory"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
                <option value="read-only">
                  read-only (Read, LS, Grep, Glob)
                </option>
                <option value="edit">
                  edit (Create, Edit, ApplyPatch)
                </option>
                <option value="execute">
                  execute (Execute)
                </option>
                <option value="web">
                  web (WebSearch, FetchUrl)
                </option>
                <option value="mcp">
                  mcp (MCP 工具)
                </option>
              </select>

              <!-- 自定义工具列表 -->
              <textarea
                v-if="toolsMode === 'custom'"
                v-model="toolsCustom"
                rows="3"
                placeholder="[&quot;Read&quot;, &quot;Write&quot;, &quot;Grep&quot;, &quot;Glob&quot;]"
                class="w-full px-4 py-2 rounded-lg glass-card font-mono text-sm"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              />
            </div>

            <!-- 系统提示 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                系统提示 * (Markdown)
              </label>
              <textarea
                v-model="formData.systemPrompt"
                required
                rows="10"
                placeholder="You are a helpful AI assistant..."
                class="w-full px-4 py-2 rounded-lg glass-card font-mono text-sm"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              />
            </div>

            <!-- 按钮 -->
            <div class="flex gap-3 pt-4">
              <button
                type="button"
                class="flex-1 px-4 py-2 rounded-lg glass-card hover:scale-105 transition-transform duration-300"
                :style="{ color: 'var(--text-secondary)' }"
                @click="closeModal"
              >
                取消
              </button>
              <button
                type="submit"
                :disabled="saving"
                class="flex-1 px-4 py-2 rounded-lg glass-card hover:scale-105 transition-transform duration-300"
                :style="{ background: 'rgba(139, 92, 246, 0.2)', color: '#8b5cf6' }"
              >
                {{ saving ? '保存中...' : '保存' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import { listDroidAgents, addDroidAgent, updateDroidAgent, deleteDroidAgent } from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'

// 类型定义
interface Droid {
  name: string
  description?: string
  model: string
  reasoningEffort?: string
  tools?: unknown
  systemPrompt: string
}

// 状态
const droids = ref<Droid[]>([])
const loading = ref(false)
const saving = ref(false)
const showAddModal = ref(false)
const editingDroid = ref<Droid | null>(null)

// 表单数据
const formData = ref<Droid>({
  name: '',
  description: '',
  model: 'inherit',
  reasoningEffort: '',
  tools: undefined,
  systemPrompt: ''
})

// 工具配置模式
const toolsMode = ref<'all' | 'category' | 'custom'>('all')
const toolsCategory = ref('read-only')
const toolsCustom = ref('')

const normalizeDroids = (data: unknown): Droid[] => {
  if (Array.isArray(data)) {
    return data as Droid[]
  }

  if (data && typeof data === 'object') {
    return Object.entries(data).map(([name, config]) => ({
      name,
      ...(config as Omit<Droid, 'name'>)
    }))
  }

  return []
}

// 加载 Droids 列表
const loadDroids = async () => {
  loading.value = true
  try {
    const data = await listDroidAgents()
    droids.value = normalizeDroids(data)
  } catch (error) {
    logger.error('加载 Droids 失败:', error)
    alert('加载 Droids 失败，请检查配置文件是否可访问')
  } finally {
    loading.value = false
  }
}

// 编辑 Droid
const editDroid = (droid: Droid) => {
  editingDroid.value = droid
  formData.value = { ...droid }

  // 设置工具模式
  if (!droid.tools) {
    toolsMode.value = 'all'
  } else if (typeof droid.tools === 'string') {
    toolsMode.value = 'category'
    toolsCategory.value = droid.tools
  } else {
    toolsMode.value = 'custom'
    toolsCustom.value = JSON.stringify(droid.tools)
  }

  showAddModal.value = true
}

// 保存 Droid
const saveDroid = async () => {
  saving.value = true
  try {
    // 处理工具配置
    let tools: unknown = undefined
    if (toolsMode.value === 'category') {
      tools = toolsCategory.value
    } else if (toolsMode.value === 'custom') {
      try {
        tools = JSON.parse(toolsCustom.value)
      } catch (e) {
        alert('工具列表 JSON 格式错误')
        saving.value = false
        return
      }
    }

    const droidData = {
      ...formData.value,
      tools,
      reasoningEffort: formData.value.reasoningEffort || undefined
    }

    if (editingDroid.value) {
      // 更新
      await updateDroidAgent(editingDroid.value.name, droidData)
      alert('Droid 更新成功！')
    } else {
      // 添加
      await addDroidAgent(formData.value.name, droidData)
      alert('Droid 添加成功！')
    }
    closeModal()
    await loadDroids()
  } catch (error: unknown) {
    logger.error('保存 Droid 失败:', error)
    alert(getErrorMessage(error) || '保存 Droid 失败')
  } finally {
    saving.value = false
  }
}

// 删除 Droid
const deleteDroid = async (name: string) => {
  if (!confirm(`确定要删除 Droid "${name}" 吗？`)) {
    return
  }

  try {
    await deleteDroidAgent(name)
    alert('Droid 删除成功！')
    await loadDroids()
  } catch (error: unknown) {
    logger.error('删除 Droid 失败:', error)
    alert(getErrorMessage(error) || '删除 Droid 失败')
  }
}

// 格式化工具显示
const formatTools = (tools: unknown): string => {
  if (!tools) return '所有工具'
  if (typeof tools === 'string') return tools
  if (Array.isArray(tools)) return tools.join(', ')
  return JSON.stringify(tools)
}

// 关闭弹窗
const closeModal = () => {
  showAddModal.value = false
  editingDroid.value = null
  formData.value = {
    name: '',
    description: '',
    model: 'inherit',
    reasoningEffort: '',
    tools: undefined,
    systemPrompt: ''
  }
  toolsMode.value = 'all'
  toolsCategory.value = 'read-only'
  toolsCustom.value = ''
}

// 页面加载时获取数据
onMounted(() => {
  loadDroids()
})
</script>

<style scoped>
.glass-card {
  background: var(--glass-bg);
  backdrop-filter: blur(10px);
  border: 1px solid var(--glass-border);
  box-shadow: 0 8px 32px 0 rgb(0 0 0 / 10%);
}

input,
select,
textarea {
  border: 1px solid var(--glass-border);
}

input:focus,
select:focus,
textarea:focus {
  outline: none;
  border-color: #8b5cf6;
}

input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.line-clamp-3 {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
