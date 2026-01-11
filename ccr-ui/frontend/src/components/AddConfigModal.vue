<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
  >
    <!-- 背景遮罩 -->
    <div
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
    />

    <!-- 弹窗内容 -->
    <div
      ref="modalRef"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="titleId"
      class="relative w-full max-w-2xl max-h-[90vh] overflow-y-auto rounded-2xl p-8 transition-all duration-300 glass-modal"
    >
      <!-- 标题栏 -->
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-3">
          <div
            class="p-3 rounded-xl"
            :style="{ background: 'rgba(var(--accent-success-rgb), 0.15)' }"
          >
            <Plus
              class="w-6 h-6"
              :style="{ color: 'var(--accent-success)' }"
            />
          </div>
          <div>
            <h2
              :id="titleId"
              class="text-2xl font-bold"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('configs.addConfig.title') }}
            </h2>
            <p
              class="text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              {{ $t('configs.addConfig.subtitle') }}
            </p>
          </div>
        </div>
        <button
          class="p-2 rounded-lg transition-all hover:scale-110"
          :style="{
            background: 'var(--color-bg-overlay)',
            color: 'var(--text-secondary)'
          }"
          :aria-label="$t('common.close')"
          @click="handleClose"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- 模板选择区域 -->
      <div class="mb-6">
        <label
          class="block text-sm font-semibold mb-3"
          :style="{ color: 'var(--text-primary)' }"
        >
          🚀 {{ $t('configs.addConfig.selectTemplate') }}
        </label>
        <div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
          <button
            v-for="template in templates"
            :key="template.id"
            type="button"
            class="p-3 rounded-xl text-left transition-all hover:scale-[1.02]"
            :style="{
              background: selectedTemplate === template.id
                ? `linear-gradient(135deg, ${template.color}20, ${template.color}10)`
                : 'var(--glass-bg)',
              border: selectedTemplate === template.id
                ? `2px solid ${template.color}`
                : '1px solid var(--glass-border)',
              boxShadow: selectedTemplate === template.id
                ? `0 4px 12px ${template.color}30`
                : 'none'
            }"
            @click="applyTemplate(template)"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="text-lg">{{ template.icon }}</span>
              <span
                class="font-semibold text-sm"
                :style="{ color: selectedTemplate === template.id ? template.color : 'var(--text-primary)' }"
              >
                {{ template.label }}
              </span>
            </div>
            <p
              class="text-xs truncate"
              :style="{ color: 'var(--text-secondary)' }"
            >
              {{ template.description }}
            </p>
          </button>
        </div>
      </div>

      <!-- 分割线 -->
      <div
        class="mb-6 h-px"
        :style="{ background: 'linear-gradient(90deg, transparent, var(--color-border-default), transparent)' }"
      />

      <!-- 表单内容 -->
      <form
        class="space-y-4"
        @submit.prevent="handleSave"
      >
        <!-- 配置名称 -->
        <div>
          <label
            class="block text-sm font-semibold mb-2"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('configs.addConfig.name') }} <span class="text-red-500">*</span>
          </label>
          <input
            v-model="formData.name"
            type="text"
            required
            class="w-full px-4 py-3 rounded-xl transition-all"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
            :placeholder="$t('configs.addConfig.namePlaceholder')"
          >
        </div>

        <!-- 描述 -->
        <div>
          <label
            class="block text-sm font-semibold mb-2"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('configs.addConfig.description') }}
          </label>
          <input
            v-model="formData.description"
            type="text"
            class="w-full px-4 py-3 rounded-xl transition-all"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
            :placeholder="$t('configs.addConfig.descriptionPlaceholder')"
          >
        </div>

        <!-- Base URL -->
        <div>
          <label
            class="block text-sm font-semibold mb-2"
            :style="{ color: 'var(--text-primary)' }"
          >
            Base URL <span class="text-red-500">*</span>
          </label>
          <input
            v-model="formData.base_url"
            type="url"
            required
            class="w-full px-4 py-3 rounded-xl transition-all"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
            placeholder="https://api.example.com"
          >
        </div>

        <!-- Auth Token -->
        <div>
          <label
            class="block text-sm font-semibold mb-2"
            :style="{ color: 'var(--text-primary)' }"
          >
            Auth Token <span class="text-red-500">*</span>
          </label>
          <input
            v-model="formData.auth_token"
            type="password"
            required
            class="w-full px-4 py-3 rounded-xl transition-all font-mono"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
            :placeholder="$t('configs.addConfig.tokenPlaceholder')"
          >
        </div>

        <!-- Model -->
        <div>
          <label
            class="block text-sm font-semibold mb-2"
            :style="{ color: 'var(--text-primary)' }"
          >
            Model
          </label>
          <input
            v-model="formData.model"
            type="text"
            class="w-full px-4 py-3 rounded-xl transition-all"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
            :placeholder="$t('configs.addConfig.modelPlaceholder')"
          >
        </div>

        <!-- Small Fast Model -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: 'var(--text-primary)' }"
          >
            ⚡ Small Fast Model
          </label>
          <input
            v-model="formData.small_fast_model"
            type="text"
            class="w-full px-4 py-3 rounded-xl transition-all"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
            :placeholder="$t('configs.addConfig.smallModelPlaceholder') || 'claude-3-haiku-20240307'"
          >
        </div>

        <!-- Provider Type -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: 'var(--text-primary)' }"
          >
            🏷️ {{ $t('configs.addConfig.providerType') || 'Provider Type' }}
          </label>
          <select
            v-model="formData.provider_type"
            class="w-full px-4 py-3 rounded-xl transition-all cursor-pointer"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
          >
            <option value="">
              ❓ {{ $t('configs.addConfig.providerUncategorized') }}
            </option>
            <option value="official_relay">
              🔄 {{ $t('configs.addConfig.providerOfficialRelay') }}
            </option>
            <option value="third_party_model">
              🤖 {{ $t('configs.addConfig.providerThirdParty') }}
            </option>
          </select>
        </div>

        <!-- Provider Name -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: 'var(--text-primary)' }"
          >
            🏢 {{ $t('configs.addConfig.providerName') || '提供商名称' }}
          </label>
          <input
            v-model="formData.provider"
            type="text"
            class="w-full px-4 py-3 rounded-xl transition-all"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
            :placeholder="$t('configs.addConfig.providerNamePlaceholder') || '如: anyrouter, glm, moonshot'"
          >
          <p
            class="text-xs mt-1"
            :style="{ color: 'var(--text-secondary)' }"
          >
            {{ $t('configs.addConfig.providerNameHint') || '用于标识同一提供商的不同配置' }}
          </p>
        </div>

        <!-- Account -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: 'var(--text-primary)' }"
          >
            👤 {{ $t('configs.addConfig.account') || '账号标识' }}
          </label>
          <input
            v-model="formData.account"
            type="text"
            class="w-full px-4 py-3 rounded-xl transition-all"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
            :placeholder="$t('configs.addConfig.accountPlaceholder') || '如: github_5953, personal, work'"
          >
          <p
            class="text-xs mt-1"
            :style="{ color: 'var(--text-secondary)' }"
          >
            {{ $t('configs.addConfig.accountHint') || '用于区分同一提供商的不同账号' }}
          </p>
        </div>

        <!-- Tags -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: 'var(--text-primary)' }"
          >
            🏷️ {{ $t('configs.addConfig.tags') || '标签' }}
          </label>
          <input
            v-model="tagsInput"
            type="text"
            class="w-full px-4 py-3 rounded-xl transition-all"
            :style="{
              background: 'var(--glass-bg)',
              border: '1px solid var(--glass-border)',
              color: 'var(--text-primary)'
            }"
            :placeholder="$t('configs.addConfig.tagsPlaceholder') || '用逗号分隔，如: free, stable, backup'"
          >
          <p
            class="text-xs mt-1"
            :style="{ color: 'var(--text-secondary)' }"
          >
            {{ $t('configs.addConfig.tagsHint') || '用于灵活分类和筛选，多个标签用逗号分隔' }}
          </p>
        </div>

        <!-- 按钮组 -->
        <div class="flex gap-3 pt-4">
          <button
            type="submit"
            :disabled="saving || !isFormValid"
            class="flex-1 px-6 py-3 rounded-xl font-semibold text-white transition-all hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
            :style="{
              background: 'var(--gradient-primary)',
              boxShadow: '0 4px 16px rgba(var(--color-success-rgb), 0.3)'
            }"
          >
            {{ saving ? $t('configs.addConfig.saving') : $t('configs.addConfig.save') }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Plus, X } from 'lucide-vue-next'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'
import { addConfig } from '@/api'
import type { UpdateConfigRequest } from '@/types'

interface Props {
  isOpen: boolean
}

interface ConfigTemplate {
  id: string
  label: string
  description: string
  icon: string
  color: string
  base_url: string
  model: string
  provider_type: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
  saved: []
}>()

const { t } = useI18n({ useScope: 'global' })

// Accessibility enhancements
const titleId = useUniqueId('add-config-modal-title')
const modalRef = ref<HTMLElement | null>(null)
const isOpenRef = ref(props.isOpen)

watch(() => props.isOpen, (newValue) => {
  isOpenRef.value = newValue
})

// Close handler
const handleClose = () => {
  emit('close')
}

const { focusFirstElement } = useFocusTrap(modalRef, isOpenRef)
useEscapeKey(handleClose, isOpenRef)

watch(isOpenRef, (isOpen) => {
  if (isOpen) {
    setTimeout(() => focusFirstElement(), 100)
  }
})

const saving = ref(false)
const selectedTemplate = ref<string | null>(null)
const tagsInput = ref('')

const formData = ref<UpdateConfigRequest>({
  name: '',
  description: '',
  base_url: '',
  auth_token: '',
  model: '',
  small_fast_model: '',
  provider_type: '',
  provider: '',
  account: '',
  tags: []
})

// 配置模板
const templates: ConfigTemplate[] = [
  {
    id: 'cc_relay',
    label: 'CC 中转',
    description: 'Claude 官方中转服务',
    icon: '🔄',
    color: 'var(--platform-gemini)',
    base_url: 'https://api.claudecc.com',
    model: 'claude-sonnet-4-20250514',
    provider_type: 'official_relay'
  },
  {
    id: 'kimi',
    label: 'Kimi',
    description: '月之暗面 Moonshot',
    icon: '🌙',
    color: 'var(--platform-claude)',
    base_url: 'https://api.moonshot.cn/v1',
    model: 'moonshot-v1-128k',
    provider_type: 'third_party_model'
  },
  {
    id: 'zhipu',
    label: '智谱 GLM',
    description: '智谱 AI ChatGLM',
    icon: '🧠',
    color: 'var(--accent-info)',
    base_url: 'https://open.bigmodel.cn/api/paas/v4',
    model: 'glm-4.6',
    provider_type: 'third_party_model'
  },
  {
    id: 'deepseek',
    label: 'DeepSeek',
    description: 'DeepSeek Chat',
    icon: '🔍',
    color: 'var(--accent-success)',
    base_url: 'https://api.deepseek.com/v1',
    model: 'deepseek-chat',
    provider_type: 'third_party_model'
  },
  {
    id: 'qwen',
    label: '通义千问',
    description: '阿里通义千问',
    icon: '☁️',
    color: 'var(--platform-qwen)',
    base_url: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    model: 'qwen-max',
    provider_type: 'third_party_model'
  }
]

// 表单验证
const isFormValid = computed(() => {
  return formData.value.name.trim() !== '' &&
         formData.value.base_url.trim() !== '' &&
         formData.value.auth_token.trim() !== ''
})

// 应用模板
const applyTemplate = (template: ConfigTemplate) => {
  selectedTemplate.value = template.id
  formData.value.base_url = template.base_url
  formData.value.model = template.model
  formData.value.provider_type = template.provider_type
  formData.value.description = template.description
  
  // 自动生成配置名（如果为空）
  if (!formData.value.name) {
    formData.value.name = template.id.replace(/_/g, '-')
  }
}

// 解析标签输入
const parseTags = (input: string): string[] => {
  return input.split(',').map(tag => tag.trim()).filter(tag => tag.length > 0)
}

// 保存配置
const handleSave = async () => {
  if (!isFormValid.value) return

  try {
    saving.value = true
    const tags = parseTags(tagsInput.value)
    const payload: UpdateConfigRequest = {
      name: formData.value.name,
      description: formData.value.description,
      base_url: formData.value.base_url,
      auth_token: formData.value.auth_token,
      model: (formData.value.model ?? '').trim() || undefined,
      small_fast_model: (formData.value.small_fast_model ?? '').trim() || undefined,
      provider: (formData.value.provider ?? '').trim() || undefined,
      provider_type: formData.value.provider_type || undefined,
      account: (formData.value.account ?? '').trim() || undefined,
      tags: tags.length > 0 ? tags : undefined
    }
    await addConfig(payload)
    alert(t('configs.addConfig.success', { name: formData.value.name }))
    emit('saved')
    handleClose()
  } catch (err) {
    console.error('添加配置失败:', err)
    alert(`${t('configs.addConfig.failed')}: ${err instanceof Error ? err.message : 'Unknown error'}`)
  } finally {
    saving.value = false
  }
}

// 重置表单
const resetForm = () => {
  formData.value = {
    name: '',
    description: '',
    base_url: '',
    auth_token: '',
    model: '',
    small_fast_model: '',
    provider_type: '',
    provider: '',
    account: '',
    tags: []
  }
  tagsInput.value = ''
  selectedTemplate.value = null
}

// 监听弹窗打开，重置表单
watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    resetForm()
  }
})
</script>
