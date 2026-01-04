<template>
  <BaseModal
    :model-value="isOpen"
    :title="title"
    :description="message"
    :close-on-backdrop="false"
    :close-on-escape="true"
    :show-close="false"
    size="sm"
    @update:model-value="handleModalChange"
    @close="handleCancel"
  >
    <!-- 图标和内容 -->
    <div class="flex flex-col items-center text-center">
      <!-- 图标 -->
      <div :class="iconContainerClasses">
        <slot name="icon">
          <component
            :is="iconComponent"
            :class="iconClasses"
          />
        </slot>
      </div>

      <!-- 标题 -->
      <h3 class="mt-4 text-lg font-semibold text-text-primary">
        {{ title }}
      </h3>

      <!-- 消息 -->
      <p class="mt-2 text-sm text-text-secondary leading-relaxed">
        {{ message }}
      </p>
    </div>

    <!-- 按钮 -->
    <template #footer>
      <div class="flex w-full gap-3">
        <button
          type="button"
          class="flex-1 px-4 py-2.5 rounded-xl text-sm font-medium
                 bg-bg-tertiary text-text-secondary
                 hover:bg-bg-hover hover:text-text-primary
                 focus:outline-none focus:ring-2 focus:ring-accent-primary/30
                 transition-colors duration-150"
          @click="handleCancel"
        >
          {{ cancelText || '取消' }}
        </button>
        <button
          type="button"
          :class="confirmButtonClasses"
          @click="handleConfirm"
        >
          {{ confirmText || '确认' }}
        </button>
      </div>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { AlertTriangle, Info, AlertCircle } from 'lucide-vue-next'
import BaseModal from '@/components/common/BaseModal.vue'

// Props interface
interface Props {
  /** 是否显示 */
  isOpen: boolean
  /** 标题 */
  title: string
  /** 消息内容 */
  message: string
  /** 确认按钮文本 */
  confirmText?: string
  /** 取消按钮文本 */
  cancelText?: string
  /** 类型变体 */
  type?: 'danger' | 'info' | 'warning'
}

// Emits
const emit = defineEmits<{
  confirm: []
  cancel: []
  'update:isOpen': [value: boolean]
}>()

// Props with defaults
const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  confirmText: '',
  cancelText: '',
})

// 图标组件映射
const iconComponent = computed(() => {
  const icons = {
    danger: AlertTriangle,
    warning: AlertCircle,
    info: Info,
  }
  return icons[props.type]
})

// 图标容器样式
const iconContainerClasses = computed(() => {
  const baseClasses = 'w-14 h-14 rounded-full flex items-center justify-center'
  const typeClasses = {
    danger: 'bg-accent-danger/10',
    warning: 'bg-accent-warning/10',
    info: 'bg-accent-info/10',
  }
  return [baseClasses, typeClasses[props.type]]
})

// 图标样式
const iconClasses = computed(() => {
  const baseClasses = 'w-7 h-7'
  const typeClasses = {
    danger: 'text-accent-danger',
    warning: 'text-accent-warning',
    info: 'text-accent-info',
  }
  return [baseClasses, typeClasses[props.type]]
})

// 确认按钮样式
const confirmButtonClasses = computed(() => {
  const baseClasses = [
    'flex-1 px-4 py-2.5 rounded-xl text-sm font-medium text-white',
    'focus:outline-none focus:ring-2 focus:ring-offset-2',
    'transition-colors duration-150',
  ]

  const typeClasses = {
    danger: [
      'bg-accent-danger',
      'hover:bg-accent-danger/90',
      'focus:ring-accent-danger/30',
    ],
    warning: [
      'bg-accent-warning',
      'hover:bg-accent-warning/90',
      'focus:ring-accent-warning/30',
    ],
    info: [
      'bg-accent-primary',
      'hover:bg-accent-primary/90',
      'focus:ring-accent-primary/30',
    ],
  }

  return [...baseClasses, ...typeClasses[props.type]]
})

// 事件处理
function handleModalChange(value: boolean) {
  emit('update:isOpen', value)
}

function handleConfirm() {
  emit('confirm')
  emit('update:isOpen', false)
}

function handleCancel() {
  emit('cancel')
  emit('update:isOpen', false)
}
</script>
