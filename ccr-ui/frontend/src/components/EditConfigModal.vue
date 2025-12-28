<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    @click.self="handleClose"
  >
    <!-- 背景遮罩 -->
    <div
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
      @click="handleClose"
    />

    <!-- 弹窗内容 -->
    <div
      ref="modalRef"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="titleId"
      class="relative w-full max-w-2xl max-h-[90vh] overflow-y-auto rounded-2xl p-8 transition-all duration-300"
      :style="{
        background: 'rgba(255, 255, 255, 0.95)',
        backdropFilter: 'blur(20px) saturate(180%)',
        WebkitBackdropFilter: 'blur(20px) saturate(180%)',
        border: '1px solid rgba(255, 255, 255, 0.3)',
        boxShadow: '0 20px 60px rgba(0, 0, 0, 0.2), inset 0 1px 0 0 rgba(255, 255, 255, 0.5)'
      }"
    >
      <!-- 标题栏 -->
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-3">
          <div
            class="p-3 rounded-xl"
            :style="{ background: 'linear-gradient(135deg, rgba(99, 102, 241, 0.2), rgba(139, 92, 246, 0.2))' }"
          >
            <Settings
              class="w-6 h-6"
              :style="{ color: '#6366f1' }"
            />
          </div>
          <div>
            <h2
              :id="titleId"
              class="text-2xl font-bold"
              :style="{ 
                background: 'linear-gradient(135deg, #6366f1, #8b5cf6)',
                WebkitBackgroundClip: 'text',
                WebkitTextFillColor: 'transparent',
                backgroundClip: 'text'
              }"
            >
              ⚙️ 编辑配置
            </h2>
            <p
              class="text-sm flex items-center gap-1"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <span>📋</span> {{ configName }}
            </p>
          </div>
        </div>
        <button
          class="p-2 rounded-lg transition-all hover:scale-110"
          :style="{
            background: 'rgba(0, 0, 0, 0.05)',
            color: 'var(--text-secondary)'
          }"
          aria-label="关闭"
          @click="handleClose"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading"
        class="flex justify-center py-20"
      >
        <div
          class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
          :style="{
            borderTopColor: '#6366f1',
            borderRightColor: '#8b5cf6'
          }"
        />
      </div>

      <!-- 表单内容 -->
      <form
        v-else
        class="space-y-4"
        @submit.prevent="handleSave"
      >
        <!-- 描述 -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: '#10b981' }"
          >
            📝 描述
          </label>
          <input
            v-model="formData.description"
            type="text"
            class="w-full px-4 py-3 rounded-xl transition-all focus:ring-2 focus:ring-indigo-500/50"
            :style="{
              background: 'rgba(255, 255, 255, 0.5)',
              border: '1px solid rgba(0, 0, 0, 0.1)',
              color: 'var(--text-primary)'
            }"
            placeholder="配置描述"
          >
        </div>

        <!-- Base URL -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: '#3b82f6' }"
          >
            🌐 Base URL
          </label>
          <input
            v-model="formData.base_url"
            type="url"
            required
            class="w-full px-4 py-3 rounded-xl transition-all focus:ring-2 focus:ring-indigo-500/50"
            :style="{
              background: 'rgba(255, 255, 255, 0.5)',
              border: '1px solid rgba(0, 0, 0, 0.1)',
              color: 'var(--text-primary)'
            }"
            placeholder="https://api.claude.ai"
          >
        </div>

        <!-- Auth Token -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: '#f59e0b' }"
          >
            🔑 Auth Token
          </label>
          <input
            v-model="formData.auth_token"
            type="text"
            required
            class="w-full px-4 py-3 rounded-xl transition-all font-mono text-sm focus:ring-2 focus:ring-indigo-500/50"
            :style="{
              background: 'rgba(255, 255, 255, 0.5)',
              border: '1px solid rgba(0, 0, 0, 0.1)',
              color: 'var(--text-primary)'
            }"
            placeholder="sk-ant-..."
          >
        </div>

        <!-- Model -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: '#8b5cf6' }"
          >
            🤖 Model
          </label>
          <input
            v-model="formData.model"
            type="text"
            class="w-full px-4 py-3 rounded-xl transition-all focus:ring-2 focus:ring-indigo-500/50"
            :style="{
              background: 'rgba(255, 255, 255, 0.5)',
              border: '1px solid rgba(0, 0, 0, 0.1)',
              color: 'var(--text-primary)'
            }"
            placeholder="claude-sonnet-4-5-20250929"
          >
        </div>

        <!-- Provider Type -->
        <div>
          <label
            class="block text-sm font-semibold mb-2 flex items-center gap-1"
            :style="{ color: '#ec4899' }"
          >
            🏷️ Provider Type
          </label>
          <select
            v-model="formData.provider_type"
            class="w-full px-4 py-3 rounded-xl transition-all focus:ring-2 focus:ring-indigo-500/50"
            :style="{
              background: 'rgba(255, 255, 255, 0.5)',
              border: '1px solid rgba(0, 0, 0, 0.1)',
              color: 'var(--text-primary)'
            }"
          >
            <option value="">
              未分类
            </option>
            <option value="official_relay">
              官方中转
            </option>
            <option value="third_party_model">
              第三方模型
            </option>
          </select>
        </div>

        <!-- 按钮组 -->
        <div class="flex gap-3 pt-4">
          <button
            type="button"
            class="flex-1 px-6 py-3 rounded-xl font-semibold transition-all hover:scale-105 flex items-center justify-center gap-2"
            :style="{
              background: 'rgba(0, 0, 0, 0.05)',
              color: 'var(--text-secondary)'
            }"
            @click="handleClose"
          >
            ❌ 取消
          </button>
          <button
            type="submit"
            :disabled="saving"
            class="flex-1 px-6 py-3 rounded-xl font-semibold text-white transition-all hover:scale-105 disabled:opacity-50 flex items-center justify-center gap-2"
            :style="{
              background: 'linear-gradient(135deg, #6366f1, #8b5cf6)',
              boxShadow: '0 4px 16px rgba(99, 102, 241, 0.3)'
            }"
          >
            {{ saving ? '⏳ 保存中...' : '💾 保存' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Settings, X } from 'lucide-vue-next'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'
import { getConfig, updateConfig } from '@/api'

interface Props {
  isOpen: boolean
  configName: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
  saved: []
}>()

// Accessibility enhancements
const titleId = useUniqueId('edit-config-modal-title')
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

const loading = ref(false)
const saving = ref(false)
const formData = ref<any>({
  description: '',
  base_url: '',
  auth_token: '',
  model: '',
  provider_type: ''
})

// 加载配置数据
const loadConfig = async () => {
  if (!props.configName) return

  try {
    loading.value = true
    // 从API获取配置数据
    const data = await getConfig(props.configName)
    formData.value = {
      description: data.description || '',
      base_url: data.base_url || '',
      auth_token: data.auth_token || '',
      model: data.model || '',
      provider_type: data.provider_type || ''
    }
  } catch (err) {
    console.error('加载配置失败:', err)
    alert(`加载配置失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  } finally {
    loading.value = false
  }
}

// 保存配置
const handleSave = async () => {
  try {
    saving.value = true
    // 构造符合后端 UpdateConfigRequest 结构的请求数据
    const payload = {
      name: props.configName,  // ✅ 添加必填的 name 字段
      ...formData.value,
      model: (formData.value.model ?? '').trim() || undefined
    }
    await updateConfig(props.configName, payload)
    alert(`✓ 成功保存配置 "${props.configName}"`)
    emit('saved')
    handleClose()
  } catch (err) {
    console.error('保存配置失败:', err)
    alert(`保存失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  } finally {
    saving.value = false
  }
}

// 监听弹窗打开
watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    loadConfig()
  }
})
</script>
