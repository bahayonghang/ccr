<!-- -->
<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #10b981 0%, #3b82f6 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #8b5cf6 0%, #ec4899 100%)',
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
              :style="{ background: 'rgba(16, 185, 129, 0.1)' }"
            >
              <ArrowLeft
                class="w-6 h-6"
                :style="{ color: '#10b981' }"
              />
            </RouterLink>
            <div>
              <h1
                class="text-3xl md:text-4xl font-bold mb-2 bg-gradient-to-r from-[#10b981] via-[#3b82f6] to-[#8b5cf6] bg-clip-text text-transparent"
              >
                Custom Models 管理
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                管理 Droid 的自定义模型配置
              </p>
            </div>
          </div>
          <button
            class="glass-card flex items-center gap-2 px-5 py-3 hover:scale-105 transition-transform duration-300"
            :style="{ background: 'rgba(16, 185, 129, 0.1)', color: '#10b981' }"
            @click="showAddModal = true"
          >
            <Plus class="w-5 h-5" />
            <span class="font-medium">添加模型</span>
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
          :style="{ borderColor: '#10b981' }"
        />
      </div>

      <!-- 模型列表 -->
      <div
        v-else-if="models.length > 0"
        class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
      >
        <div
          v-for="model in models"
          :key="model.model"
          class="glass-card p-6 hover:scale-105 transition-transform duration-300"
        >
          <!-- 模型头部 -->
          <div class="flex items-start justify-between mb-4">
            <div class="flex-1">
              <h3
                class="text-xl font-bold mb-1"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ model.displayName || model.model }}
              </h3>
              <p
                class="text-sm font-mono"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ model.model }}
              </p>
            </div>
            <div class="flex gap-2">
              <button
                class="p-2 rounded-lg hover:bg-blue-500/10 transition-colors"
                :style="{ color: '#3b82f6' }"
                @click="editModel(model)"
              >
                <Edit2 class="w-4 h-4" />
              </button>
              <button
                class="p-2 rounded-lg hover:bg-red-500/10 transition-colors"
                :style="{ color: '#ef4444' }"
                @click="deleteModel(model.model)"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- 模型信息 -->
          <div class="space-y-2">
            <div class="flex items-center gap-2">
              <Server
                class="w-4 h-4"
                :style="{ color: '#64748b' }"
              />
              <span
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ model.provider }}
              </span>
            </div>
            <div class="flex items-center gap-2">
              <Globe
                class="w-4 h-4"
                :style="{ color: '#64748b' }"
              />
              <span
                class="text-sm truncate"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ model.baseUrl }}
              </span>
            </div>
            <div
              v-if="model.maxOutputTokens"
              class="flex items-center gap-2"
            >
              <Zap
                class="w-4 h-4"
                :style="{ color: '#64748b' }"
              />
              <span
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                Max Tokens: {{ model.maxOutputTokens }}
              </span>
            </div>
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
          :style="{ background: 'rgba(16, 185, 129, 0.1)' }"
        >
          <Inbox
            class="w-16 h-16"
            :style="{ color: '#10b981' }"
          />
        </div>
        <h3
          class="text-2xl font-bold mb-2"
          :style="{ color: 'var(--text-primary)' }"
        >
          还没有自定义模型
        </h3>
        <p
          class="text-lg mb-6"
          :style="{ color: 'var(--text-secondary)' }"
        >
          点击"添加模型"按钮创建第一个自定义模型
        </p>
      </div>

      <!-- 添加/编辑模型弹窗 -->
      <div
        v-if="showAddModal"
        class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
        @click.self="closeModal"
      >
        <div class="glass-card p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto">
          <div class="flex items-center justify-between mb-6">
            <h2
              class="text-2xl font-bold"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ editingModel ? '编辑模型' : '添加模型' }}
            </h2>
            <button
              class="p-2 hover:bg-gray-500/10 rounded-lg transition-colors"
              @click="closeModal"
            >
              <X
                class="w-5 h-5"
                :style="{ color: 'var(--text-secondary)' }"
              />
            </button>
          </div>

          <form
            class="space-y-4"
            @submit.prevent="saveModel"
          >
            <!-- 模型 ID -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                模型 ID *
              </label>
              <input
                v-model="formData.model"
                type="text"
                required
                :disabled="!!editingModel"
                placeholder="claude-sonnet-4-5"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
            </div>

            <!-- 显示名称 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                显示名称
              </label>
              <input
                v-model="formData.displayName"
                type="text"
                placeholder="Claude Sonnet 4.5"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
            </div>

            <!-- API 端点 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                API 端点 *
              </label>
              <input
                v-model="formData.baseUrl"
                type="url"
                required
                placeholder="https://api.anthropic.com/v1"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
            </div>

            <!-- API Key -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                API Key *
              </label>
              <input
                v-model="formData.apiKey"
                type="password"
                required
                placeholder="sk-ant-..."
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
            </div>

            <!-- 提供商 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                提供商 *
              </label>
              <select
                v-model="formData.provider"
                required
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
                <option value="anthropic">
                  Anthropic
                </option>
                <option value="openai">
                  OpenAI
                </option>
                <option value="generic-chat-completion-api">
                  Generic Chat Completion API
                </option>
              </select>
            </div>

            <!-- Max Output Tokens -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                最大输出 Tokens
              </label>
              <input
                v-model.number="formData.maxOutputTokens"
                type="number"
                placeholder="8192"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
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
                :style="{ background: 'rgba(16, 185, 129, 0.2)', color: '#10b981' }"
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
import { ref, onMounted } from 'vue'
import { ArrowLeft, Plus, Edit2, Trash2, Server, Globe, Zap, Inbox, X } from 'lucide-vue-next'
import { listDroidModels, addDroidModel, updateDroidModel, deleteDroidModel } from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'

// 类型定义
interface DroidCustomModel {
  model: string
  displayName?: string
  baseUrl: string
  apiKey: string
  provider: string
  maxOutputTokens?: number
}

// 状态
const models = ref<DroidCustomModel[]>([])
const loading = ref(false)
const saving = ref(false)
const showAddModal = ref(false)
const editingModel = ref<DroidCustomModel | null>(null)

// 表单数据
const formData = ref<DroidCustomModel>({
  model: '',
  displayName: '',
  baseUrl: '',
  apiKey: '',
  provider: 'anthropic',
  maxOutputTokens: undefined
})

const loadModels = async () => {
  loading.value = true
  try {
    const data = await listDroidModels()
    models.value = Array.isArray(data) ? (data as DroidCustomModel[]) : []
  } catch (error) {
    logger.error('加载模型失败:', error)
    alert('加载模型失败，请检查配置文件是否可访问')
  } finally {
    loading.value = false
  }
}

// 编辑模型
const editModel = (model: DroidCustomModel) => {
  editingModel.value = model
  formData.value = { ...model }
  showAddModal.value = true
}

// 保存模型
const saveModel = async () => {
  saving.value = true
  try {
    if (editingModel.value) {
      await updateDroidModel(editingModel.value.model, formData.value)
      alert('模型更新成功！')
    } else {
      await addDroidModel(formData.value)
      alert('模型添加成功！')
    }

    closeModal()
    await loadModels()
  } catch (error: unknown) {
    logger.error('保存模型失败:', error)
    alert(getErrorMessage(error) || '保存模型失败')
  } finally {
    saving.value = false
  }
}

// 删除模型
const deleteModel = async (modelId: string) => {
  if (!confirm(`确定要删除模型 "${modelId}" 吗？`)) {
    return
  }

  try {
    await deleteDroidModel(modelId)
    alert('模型删除成功！')
    await loadModels()
  } catch (error: unknown) {
    logger.error('删除模型失败:', error)
    alert(getErrorMessage(error) || '删除模型失败')
  }
}

// 关闭弹窗
const closeModal = () => {
  showAddModal.value = false
  editingModel.value = null
  formData.value = {
    model: '',
    displayName: '',
    baseUrl: '',
    apiKey: '',
    provider: 'anthropic',
    maxOutputTokens: undefined
  }
}

// 页面加载时获取数据
onMounted(() => {
  loadModels()
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
select {
  border: 1px solid var(--glass-border);
}

input:focus,
select:focus {
  outline: none;
  border-color: #10b981;
}

input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
