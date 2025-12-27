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
      ref="modalRef"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="titleId"
      :aria-describedby="descId"
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
          :id="titleId"
          class="text-xl font-bold mb-2"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ title }}
        </h3>
        <p
          :id="descId"
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
import { computed, ref, watch } from 'vue'
import { AlertTriangle, Info } from 'lucide-vue-next'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'

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

// Generate unique IDs for ARIA labels
const titleId = useUniqueId('confirm-modal-title')
const descId = useUniqueId('confirm-modal-desc')

// Modal container ref for focus trap
const modalRef = ref<HTMLElement | null>(null)
const isOpenRef = ref(props.isOpen)

// Update isOpenRef when props.isOpen changes
watch(() => props.isOpen, (newValue) => {
  isOpenRef.value = newValue
})

// Cancel handler
const handleCancel = () => {
  emit('cancel')
  emit('update:isOpen', false)
}

// Focus trap and escape key handling
const { focusFirstElement } = useFocusTrap(modalRef, isOpenRef)
useEscapeKey(handleCancel, isOpenRef)

// Focus first element when modal opens
watch(isOpenRef, (isOpen) => {
  if (isOpen) {
    setTimeout(() => focusFirstElement(), 100)
  }
})

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
</script>
