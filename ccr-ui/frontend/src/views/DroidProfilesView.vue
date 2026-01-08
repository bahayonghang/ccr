<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #ec4899 0%, #f59e0b 100%)',
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
              class="p-3 rounded-2xl glass-card hover:scale-105 transition-all duration-300"
              :style="{ background: 'rgba(59, 130, 246, 0.1)' }"
            >
              <ArrowLeft
                class="w-6 h-6"
                :style="{ color: '#3b82f6' }"
              />
            </RouterLink>
            <div>
              <h1
                class="text-3xl md:text-4xl font-bold mb-2 bg-gradient-to-r from-[#3b82f6] via-[#8b5cf6] to-[#ec4899] bg-clip-text text-transparent"
              >
                Profiles 管理
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                管理 Droid 的配置文件 (Profiles)
              </p>
            </div>
          </div>
          <button
            class="glass-card flex items-center gap-2 px-5 py-3 hover:scale-105 transition-all duration-300"
            :style="{ background: 'rgba(59, 130, 246, 0.1)', color: '#3b82f6' }"
            @click="showAddModal = true"
          >
            <Plus class="w-5 h-5" />
            <span class="font-medium">添加 Profile</span>
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
          :style="{ borderColor: '#3b82f6' }"
        />
      </div>

      <!-- Profile 列表 -->
      <div
        v-else-if="profiles.length > 0"
        class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
      >
        <div
          v-for="profile in profiles"
          :key="profile.name"
          class="glass-card p-6 hover:scale-105 transition-all duration-300"
          :class="{ 'ring-2 ring-blue-500': profile.enabled }"
        >
          <!-- Profile 头部 -->
          <div class="flex items-start justify-between mb-4">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3
                  class="text-xl font-bold"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ profile.name }}
                </h3>
                <span
                  v-if="profile.enabled"
                  class="px-2 py-1 text-xs font-medium rounded-full"
                  :style="{ background: 'rgba(59, 130, 246, 0.2)', color: '#3b82f6' }"
                >
                  当前激活
                </span>
              </div>
              <p
                v-if="profile.description"
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ profile.description }}
              </p>
            </div>
            <div class="flex gap-2">
              <button
                class="p-2 rounded-lg hover:bg-blue-500/10 transition-colors"
                :style="{ color: '#3b82f6' }"
                @click="editProfile(profile)"
              >
                <Edit2 class="w-4 h-4" />
              </button>
              <button
                class="p-2 rounded-lg hover:bg-red-500/10 transition-colors"
                :style="{ color: '#ef4444' }"
                @click="deleteProfile(profile.name)"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- Profile 信息 -->
          <div class="space-y-2 mb-4">
            <div class="flex items-center gap-2">
              <Server
                class="w-4 h-4"
                :style="{ color: '#64748b' }"
              />
              <span
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ profile.model || 'N/A' }}
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
                {{ profile.base_url || 'N/A' }}
              </span>
            </div>
            <div class="flex items-center gap-2">
              <Zap
                class="w-4 h-4"
                :style="{ color: '#64748b' }"
              />
              <span
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ profile.provider || 'N/A' }}
              </span>
            </div>
          </div>

          <!-- 切换按钮 -->
          <button
            v-if="!profile.enabled"
            class="w-full px-4 py-2 rounded-lg glass-card hover:scale-105 transition-all duration-300"
            :style="{ background: 'rgba(59, 130, 246, 0.1)', color: '#3b82f6' }"
            @click="switchProfile(profile.name)"
          >
            切换到此 Profile
          </button>
        </div>
      </div>

      <!-- 空状态 -->
      <div
        v-else
        class="text-center py-20"
      >
        <div
          class="inline-block p-6 rounded-3xl glass-card mb-6"
          :style="{ background: 'rgba(59, 130, 246, 0.1)' }"
        >
          <Inbox
            class="w-16 h-16"
            :style="{ color: '#3b82f6' }"
          />
        </div>
        <h3
          class="text-2xl font-bold mb-2"
          :style="{ color: 'var(--text-primary)' }"
        >
          还没有 Profile
        </h3>
        <p
          class="text-lg mb-6"
          :style="{ color: 'var(--text-secondary)' }"
        >
          点击"添加 Profile"按钮创建第一个配置文件
        </p>
      </div>

      <!-- 添加/编辑 Profile 弹窗 -->
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
              {{ editingProfile ? '编辑 Profile' : '添加 Profile' }}
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
            @submit.prevent="saveProfile"
          >
            <!-- Profile 名称 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                Profile 名称 *
              </label>
              <input
                v-model="formData.name"
                type="text"
                required
                :disabled="!!editingProfile"
                placeholder="my-profile"
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
                描述
              </label>
              <input
                v-model="formData.description"
                type="text"
                placeholder="我的自定义配置"
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
                API 端点
              </label>
              <input
                v-model="formData.base_url"
                type="url"
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
                API Key
              </label>
              <input
                v-model="formData.api_key"
                type="password"
                placeholder="sk-ant-..."
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
                模型
              </label>
              <input
                v-model="formData.model"
                type="text"
                placeholder="claude-sonnet-4-5"
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
                提供商
              </label>
              <select
                v-model="formData.provider"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
                <option value="">
                  选择提供商
                </option>
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

            <!-- 提供商类型 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                提供商类型
              </label>
              <input
                v-model="formData.provider_type"
                type="text"
                placeholder="anthropic"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
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
                v-model.number="formData.max_output_tokens"
                type="number"
                placeholder="8192"
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
                v-model="formData.display_name"
                type="text"
                placeholder="Claude Sonnet 4.5"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
            </div>

            <!-- 标签 -->
            <div>
              <label
                class="block text-sm font-medium mb-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                标签 (逗号分隔)
              </label>
              <input
                v-model="tagsInput"
                type="text"
                placeholder="production, fast, reliable"
                class="w-full px-4 py-2 rounded-lg glass-card"
                :style="{ color: 'var(--text-primary)', background: 'var(--glass-bg)' }"
              >
            </div>

            <!-- 启用状态 -->
            <div class="flex items-center gap-2">
              <input
                id="enabled"
                v-model="formData.enabled"
                type="checkbox"
                class="w-4 h-4 rounded"
              >
              <label
                for="enabled"
                class="text-sm font-medium"
                :style="{ color: 'var(--text-primary)' }"
              >
                启用此 Profile
              </label>
            </div>

            <!-- 按钮 -->
            <div class="flex gap-3 pt-4">
              <button
                type="button"
                class="flex-1 px-4 py-2 rounded-lg glass-card hover:scale-105 transition-all duration-300"
                :style="{ color: 'var(--text-secondary)' }"
                @click="closeModal"
              >
                取消
              </button>
              <button
                type="submit"
                :disabled="saving"
                class="flex-1 px-4 py-2 rounded-lg glass-card hover:scale-105 transition-all duration-300"
                :style="{ background: 'rgba(59, 130, 246, 0.2)', color: '#3b82f6' }"
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
import axios from 'axios'

// 类型定义
interface DroidProfile {
  name: string
  description?: string
  base_url?: string
  api_key?: string
  model?: string
  provider?: string
  provider_type?: string
  max_output_tokens?: number
  display_name?: string
  tags?: string[]
  enabled: boolean
}

// 状态
const profiles = ref<DroidProfile[]>([])
const loading = ref(false)
const saving = ref(false)
const showAddModal = ref(false)
const editingProfile = ref<DroidProfile | null>(null)

// 表单数据
const formData = ref<DroidProfile>({
  name: '',
  description: '',
  base_url: '',
  api_key: '',
  model: '',
  provider: '',
  provider_type: '',
  max_output_tokens: undefined,
  display_name: '',
  tags: [],
  enabled: false
})

// 标签输入 (逗号分隔字符串)
const tagsInput = ref('')

// API 基础 URL
const API_BASE = 'http://localhost:8081/api/droid'

// 加载 Profile 列表
const loadProfiles = async () => {
  loading.value = true
  try {
    const response = await axios.get(`${API_BASE}/profiles`)
    if (response.data.success) {
      profiles.value = response.data.data
    }
  } catch (error) {
    console.error('加载 Profiles 失败:', error)
    alert('加载 Profiles 失败，请检查后端服务是否运行')
  } finally {
    loading.value = false
  }
}

// 编辑 Profile
const editProfile = (profile: DroidProfile) => {
  editingProfile.value = profile
  formData.value = { ...profile }
  tagsInput.value = profile.tags?.join(', ') || ''
  showAddModal.value = true
}

// 保存 Profile
const saveProfile = async () => {
  saving.value = true
  try {
    // 处理标签
    const tags = tagsInput.value
      .split(',')
      .map(tag => tag.trim())
      .filter(tag => tag.length > 0)
    
    const profileData = {
      ...formData.value,
      tags: tags.length > 0 ? tags : undefined
    }

    if (editingProfile.value) {
      // 更新
      await axios.put(`${API_BASE}/profiles/${editingProfile.value.name}`, profileData)
      alert('Profile 更新成功！')
    } else {
      // 添加
      await axios.post(`${API_BASE}/profiles`, profileData)
      alert('Profile 添加成功！')
    }
    closeModal()
    await loadProfiles()
  } catch (error: any) {
    console.error('保存 Profile 失败:', error)
    alert(error.response?.data?.message || '保存 Profile 失败')
  } finally {
    saving.value = false
  }
}

// 删除 Profile
const deleteProfile = async (name: string) => {
  if (!confirm(`确定要删除 Profile "${name}" 吗？`)) {
    return
  }

  try {
    await axios.delete(`${API_BASE}/profiles/${name}`)
    alert('Profile 删除成功！')
    await loadProfiles()
  } catch (error: any) {
    console.error('删除 Profile 失败:', error)
    alert(error.response?.data?.message || '删除 Profile 失败')
  }
}

// 切换 Profile
const switchProfile = async (name: string) => {
  try {
    await axios.post(`${API_BASE}/profiles/${name}/switch`)
    alert(`已切换到 Profile "${name}"！`)
    await loadProfiles()
  } catch (error: any) {
    console.error('切换 Profile 失败:', error)
    alert(error.response?.data?.message || '切换 Profile 失败')
  }
}

// 关闭弹窗
const closeModal = () => {
  showAddModal.value = false
  editingProfile.value = null
  formData.value = {
    name: '',
    description: '',
    base_url: '',
    api_key: '',
    model: '',
    provider: '',
    provider_type: '',
    max_output_tokens: undefined,
    display_name: '',
    tags: [],
    enabled: false
  }
  tagsInput.value = ''
}

// 页面加载时获取数据
onMounted(() => {
  loadProfiles()
})
</script>

<style scoped>
.glass-card {
  background: var(--glass-bg);
  backdrop-filter: blur(10px);
  border: 1px solid var(--glass-border);
  box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.1);
}

input,
select {
  border: 1px solid var(--glass-border);
}

input:focus,
select:focus {
  outline: none;
  border-color: #3b82f6;
}

input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
