<!-- -->
<template>
  <div class="min-h-full relative overflow-hidden">
    <!-- 🎨 彩色背景装饰 -->
    <div class="absolute inset-0 overflow-hidden pointer-events-none -z-10">
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
              class="glass-card rounded-2xl bg-accent-primary/10 p-3 text-accent-primary transition-colors duration-300 hover:bg-bg-overlay/70"
            >
              <SIcon
                name="ArrowLeft"
                size="w-6 h-6"
              />
            </RouterLink>
            <div>
              <h1 class="mb-2 text-3xl font-bold text-text-primary md:text-4xl">
                Profiles 管理
              </h1>
              <p class="text-lg text-text-secondary">
                管理 Droid 的配置文件 (Profiles)
              </p>
            </div>
          </div>
          <button
            class="glass-card flex min-h-[44px] items-center gap-2 bg-accent-primary/10 px-5 py-3 text-accent-primary transition-colors duration-300 hover:bg-accent-primary/15"
            @click="showAddModal = true"
          >
            <SIcon
              name="Plus"
              size="w-5 h-5"
            />
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
          class="h-12 w-12 animate-spin rounded-full border-b-2 border-accent-primary"
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
          class="glass-card p-6 transition-[transform,box-shadow,border-color] duration-300 hover:-translate-y-1 hover:border-white/30"
          :class="{ 'ring-2 ring-accent-primary/60': profile.enabled }"
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
                  class="rounded-full bg-accent-primary/15 px-2 py-1 text-xs font-medium text-accent-primary"
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
                class="rounded-lg p-2 text-accent-primary transition-colors hover:bg-accent-primary/10"
                @click="editProfile(profile)"
              >
                <SIcon
                  name="Edit2"
                  size="w-4 h-4"
                />
              </button>
              <button
                class="rounded-lg p-2 text-accent-danger transition-colors hover:bg-accent-danger/10"
                @click="deleteProfile(profile.name)"
              >
                <SIcon
                  name="Trash2"
                  size="w-4 h-4"
                />
              </button>
            </div>
          </div>

          <!-- Profile 信息 -->
          <div class="space-y-2 mb-4">
            <div class="flex items-center gap-2">
              <SIcon
                name="Server"
                size="w-4 h-4"
                class="text-text-muted"
              />
              <span
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ profile.model || 'N/A' }}
              </span>
            </div>
            <div class="flex items-center gap-2">
              <SIcon
                name="Globe"
                size="w-4 h-4"
                class="text-text-muted"
              />
              <span
                class="text-sm truncate"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ profile.base_url || 'N/A' }}
              </span>
            </div>
            <div class="flex items-center gap-2">
              <SIcon
                name="Zap"
                size="w-4 h-4"
                class="text-text-muted"
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
            class="glass-card w-full min-h-[44px] rounded-lg bg-accent-primary/10 px-4 py-2 text-accent-primary transition-colors duration-300 hover:bg-accent-primary/15"
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
          class="glass-card mb-6 inline-block rounded-3xl bg-accent-primary/10 p-6"
        >
          <SIcon
            name="Inbox"
            size="w-16 h-16"
            class="text-accent-primary"
          />
        </div>
        <h3 class="mb-2 text-2xl font-bold text-text-primary">
          还没有 Profile
        </h3>
        <p class="mb-6 text-lg text-text-secondary">
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
              <SIcon
                name="X"
                size="w-5 h-5"
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
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import {
  listDroidProfiles,
  addDroidProfile,
  updateDroidProfile,
  deleteDroidProfile,
  switchDroidProfile,
} from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'

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

// 加载 Profile 列表
const loadProfiles = async () => {
  loading.value = true
  try {
    const data = await listDroidProfiles()
    profiles.value = Array.isArray(data) ? (data as DroidProfile[]) : []
  } catch (error) {
    logger.error('加载 Profiles 失败:', error)
    alert('加载 Profiles 失败，请检查配置文件是否可访问')
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
      await updateDroidProfile(editingProfile.value.name, profileData)
      alert('Profile 更新成功！')
    } else {
      // 添加
      await addDroidProfile(formData.value.name, profileData)
      alert('Profile 添加成功！')
    }
    closeModal()
    await loadProfiles()
  } catch (error: unknown) {
    logger.error('保存 Profile 失败:', error)
    alert(getErrorMessage(error) || '保存 Profile 失败')
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
    await deleteDroidProfile(name)
    alert('Profile 删除成功！')
    await loadProfiles()
  } catch (error: unknown) {
    logger.error('删除 Profile 失败:', error)
    alert(getErrorMessage(error) || '删除 Profile 失败')
  }
}

// 切换 Profile
const switchProfile = async (name: string) => {
  try {
    await switchDroidProfile(name)
    alert(`已切换到 Profile "${name}"！`)
    await loadProfiles()
  } catch (error: unknown) {
    logger.error('切换 Profile 失败:', error)
    alert(getErrorMessage(error) || '切换 Profile 失败')
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
  box-shadow: 0 8px 32px 0 rgb(0 0 0 / 10%);
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

input:focus-visible,
select:focus-visible {
  outline: 2px solid rgb(var(--color-accent-primary-rgb) / 50%);
  outline-offset: 2px;
}

input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
