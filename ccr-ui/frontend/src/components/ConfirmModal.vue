<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    @click.self="handleCancel"
  >
    <!-- 背景遮罩 -->
    <div
      class="absolute inset-0 bg-black/50 backdrop-blur-sm transition-opacity duration-300"
      @click="handleCancel"
    />

    <!-- 弹窗内容 -->
    <div
      class="relative w-full max-w-md overflow-hidden rounded-2xl p-6 transition-all duration-300 transform scale-100"
      :style="{
        background: 'rgba(255, 255, 255, 0.95)',
        backdropFilter: 'blur(20px) saturate(180%)',
        WebkitBackdropFilter: 'blur(20px) saturate(180%)',
        border: '1px solid rgba(255, 255, 255, 0.3)',
        boxShadow: '0 20px 60px rgba(0, 0, 0, 0.2), inset 0 1px 0 0 rgba(255, 255, 255, 0.5)'
      }"
    >
      <!-- 图标 -->
      <div class="flex justify-center mb-4">
        <div
          class="p-3 rounded-full"
          :style="{
            background: type === 'danger' ? 'rgba(239, 68, 68, 0.1)' : 'rgba(99, 102, 241, 0.1)',
            color: type === 'danger' ? 'var(--accent-danger)' : 'var(--accent-primary)'
          }"
        >
          <slot name="icon">
            <component
              :is="iconComponent"
              class="w-8 h-8"
            />
          </slot>
        </div>
      </div>

      <!-- 标题和内容 -->
      <div class="text-center mb-6">
        <h3
          class="text-xl font-bold mb-2"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ title }}
        </h3>
        <p
          class="text-sm"
          :style="{ color: 'var(--text-secondary)' }"
        >
          {{ message }}
        </p>
      </div>

      <!-- 按钮组 -->
      <div class="flex gap-3">
        <button
          class="flex-1 px-4 py-2.5 rounded-xl font-semibold transition-all hover:scale-105"
          :style="{
            background: 'rgba(0, 0, 0, 0.05)',
            color: 'var(--text-secondary)'
          }"
          @click="handleCancel"
        >
          {{ cancelText || $t('common.cancel') }}
        </button>
        <button
          class="flex-1 px-4 py-2.5 rounded-xl font-semibold text-white transition-all hover:scale-105 shadow-lg"
          :style="{
            background: type === 'danger' 
              ? 'linear-gradient(135deg, #ef4444, #dc2626)' 
              : 'linear-gradient(135deg, #6366f1, #8b5cf6)',
            boxShadow: type === 'danger' 
              ? '0 4px 12px rgba(239, 68, 68, 0.3)' 
              : '0 4px 12px rgba(99, 102, 241, 0.3)'
          }"
          @click="handleConfirm"
        >
          {{ confirmText || $t('common.confirm') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { AlertTriangle, Info } from 'lucide-vue-next'

interface Props {
  isOpen: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: 'danger' | 'info' | 'warning'
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  confirmText: '',
  cancelText: ''
})

const emit = defineEmits<{
  confirm: []
  cancel: []
  'update:isOpen': [value: boolean]
}>()

const iconComponent = computed(() => {
  switch (props.type) {
    case 'danger':
      return AlertTriangle
    case 'warning':
      return AlertTriangle
    default:
      return Info
  }
})

const handleConfirm = () => {
  emit('confirm')
}

const handleCancel = () => {
  emit('cancel')
  emit('update:isOpen', false)
}
</script>
