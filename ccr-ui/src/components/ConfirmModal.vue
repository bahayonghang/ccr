<template>
  <BaseModal
    :model-value="isOpen"
    :title="title"
    :description="message"
    :close-on-backdrop="false"
    :close-on-escape="true"
    :show-close="false"
    :surface="surface"
    size="sm"
    @update:model-value="handleModalChange"
    @close="handleCancel"
  >
    <template #header="{ titleId }">
      <h2
        :id="titleId"
        class="w-full text-center text-lg font-semibold text-text-primary"
      >
        {{ title }}
      </h2>
    </template>

    <div class="flex flex-col items-center text-center pb-1">
      <div :class="iconContainerClasses">
        <slot name="icon">
          <SIcon
            :name="iconComponent"
            :class="iconClasses"
          />
        </slot>
      </div>

      <p class="mt-4 text-sm leading-relaxed text-text-secondary">
        {{ message }}
      </p>
    </div>

    <template #footer>
      <div class="flex w-full gap-3">
        <button
          type="button"
          class="flex-1 rounded-xl border border-border-default bg-bg-surface px-4 py-2.5 text-sm font-medium text-text-secondary transition-colors duration-150 hover:bg-bg-overlay hover:text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/30"
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
import SIcon from '@/components/ui/SIcon.vue'
import { computed } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'

interface Props {
  isOpen: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: 'danger' | 'info' | 'warning'
  surface?: 'glass' | 'solid'
}

const emit = defineEmits<{
  confirm: []
  cancel: []
  'update:isOpen': [value: boolean]
}>()

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  surface: 'glass',
  confirmText: '',
  cancelText: '',
})

const iconComponent = computed(() => {
  const icons = {
    danger: 'AlertTriangle',
    warning: 'AlertCircle',
    info: 'Info',
  }

  return icons[props.type]
})

const iconContainerClasses = computed(() => {
  const baseClasses = 'flex h-14 w-14 items-center justify-center rounded-full border shadow-sm'
  const typeClasses = {
    danger: 'border-accent-danger/20 bg-accent-danger/10',
    warning: 'border-accent-warning/20 bg-accent-warning/10',
    info: 'border-accent-info/20 bg-accent-info/10',
  }

  return [baseClasses, typeClasses[props.type]]
})

const iconClasses = computed(() => {
  const baseClasses = 'h-7 w-7'
  const typeClasses = {
    danger: 'text-accent-danger',
    warning: 'text-accent-warning',
    info: 'text-accent-info',
  }

  return [baseClasses, typeClasses[props.type]]
})

const confirmButtonClasses = computed(() => {
  const baseClasses = [
    'flex-1 rounded-xl px-4 py-2.5 text-sm font-medium text-white shadow-sm',
    'focus:outline-none focus:ring-2',
    'transition-colors duration-150',
  ]

  const typeClasses: Record<string, string[]> = {
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