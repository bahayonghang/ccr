<template>
  <BaseModal
    :model-value="isOpen"
    :title="title"
    :description="message"
    :close-on-backdrop="false"
    :close-on-escape="true"
    :show-close="false"
    :surface="resolvedSurface"
    :content-class="modalClasses"
    size="sm"
    @update:model-value="handleModalChange"
    @close="handleCancel"
  >
    <template #header="{ titleId }">
      <h2
        :id="titleId"
        class="confirm-modal__title w-full text-center text-lg font-semibold"
      >
        {{ title }}
      </h2>
    </template>

    <div class="confirm-modal__body flex flex-col items-center text-center pb-1">
      <div :class="iconContainerClasses">
        <slot name="icon">
          <SIcon
            :name="iconComponent"
            :class="iconClasses"
          />
        </slot>
      </div>

      <p class="confirm-modal__message mt-4 text-sm leading-relaxed">
        {{ message }}
      </p>

      <!-- 结构化附加内容（如 Apply diff 行）：缺省不渲染，保持旧行为 -->
      <div
        v-if="$slots.details"
        class="confirm-modal__details mt-3 w-full"
      >
        <slot name="details" />
      </div>

      <p
        v-if="footnote"
        class="confirm-modal__footnote mt-3 text-xs leading-relaxed"
      >
        {{ footnote }}
      </p>
    </div>

    <template #footer>
      <div class="confirm-modal__footer flex w-full gap-3">
        <button
          type="button"
          class="confirm-modal__button confirm-modal__button--cancel flex-1 rounded-xl px-4 py-2.5 text-sm font-medium transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-accent-primary/30"
          @click="handleCancel"
        >
          {{ cancelText || '取消' }}
        </button>
        <button
          type="button"
          :class="['confirm-modal__button flex-1 rounded-xl px-4 py-2.5 text-sm font-medium focus:outline-none focus:ring-2 transition-colors duration-150', confirmButtonClasses]"
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
  /** 底部补充说明行（如 delete 备份提示）；缺省不渲染 */
  footnote?: string
}

const emit = defineEmits<{
  confirm: []
  cancel: []
  'update:isOpen': [value: boolean]
}>()

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  surface: 'solid',
  confirmText: '',
  cancelText: '',
  footnote: '',
})

const resolvedSurface = computed(() => props.surface)
const modalClasses = computed(() => `confirm-modal confirm-modal--${props.type}`)

const iconComponent = computed(() => {
  const icons = {
    danger: 'AlertTriangle',
    warning: 'AlertCircle',
    info: 'Info',
  }

  return icons[props.type]
})

const iconContainerClasses = computed(() => {
  const baseClasses = 'confirm-modal__icon-wrap flex h-14 w-14 items-center justify-center rounded-full border shadow-sm'
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
    'shadow-sm',
  ]

  const typeClasses: Record<string, string[]> = {
    danger: [
      'bg-accent-danger',
      'text-[color:var(--color-danger-contrast)]',
      'hover:bg-accent-danger/90',
      'focus:ring-accent-danger/30',
    ],
    warning: [
      'bg-accent-warning',
      'text-[color:var(--color-warning-contrast)]',
      'hover:bg-accent-warning/90',
      'focus:ring-accent-warning/30',
    ],
    info: [
      'bg-accent-primary',
      'text-[color:var(--color-accent-primary-contrast)]',
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

<style>
.confirm-modal {
  --confirm-shell-bg: var(--surface-modal-bg);
  --confirm-shell-border: var(--surface-modal-border);
  --confirm-shell-shadow: var(--surface-modal-shadow);
  --confirm-text-primary: var(--color-text-primary);
  --confirm-text-secondary: var(--color-text-secondary);
  --confirm-hairline: var(--color-border-subtle);
  --confirm-muted-bg: var(--color-bg-surface);
  --confirm-muted-hover: var(--color-bg-overlay);

  background: var(--confirm-shell-bg) !important;
  border: 1px solid var(--confirm-shell-border) !important;
  box-shadow: var(--confirm-shell-shadow) !important;
}

.confirm-modal__title {
  color: var(--confirm-text-primary);
}

.confirm-modal__message {
  color: var(--confirm-text-secondary);
}

.confirm-modal__details {
  color: var(--confirm-text-primary);
  text-align: left;
}

.confirm-modal__footnote {
  color: var(--confirm-text-secondary);
  opacity: 0.82;
}

.confirm-modal__footer {
  padding-top: 0.25rem;
}

.confirm-modal__button {
  border: 1px solid transparent;
}

.confirm-modal__button--cancel {
  border-color: var(--confirm-hairline);
  background: var(--confirm-muted-bg);
  color: var(--confirm-text-primary);
}

.confirm-modal__button--cancel:hover {
  background: var(--confirm-muted-hover);
}
</style>
