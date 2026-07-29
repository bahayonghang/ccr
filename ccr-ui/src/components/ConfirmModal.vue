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
  --confirm-shell-bg: linear-gradient(180deg, rgb(255 253 252 / 98%), rgb(247 244 255 / 94%));
  --confirm-shell-border: rgb(var(--color-border-default-rgb) / 82%);
  --confirm-shell-shadow: 0 28px 72px rgb(31 17 58 / 16%), 0 12px 28px rgb(31 17 58 / 10%);
  --confirm-shell-highlight:
    radial-gradient(circle at top center, rgb(var(--color-accent-primary-rgb) / 10%), transparent 42%),
    radial-gradient(circle at bottom right, rgb(var(--color-warning-rgb) / 8%), transparent 36%);
  --confirm-text-primary: var(--color-text-primary);
  --confirm-text-secondary: var(--color-text-secondary);
  --confirm-hairline: rgb(var(--color-border-default-rgb) / 58%);
  --confirm-muted-bg: rgb(var(--color-bg-surface-rgb) / 72%);
  --confirm-muted-hover: rgb(var(--color-bg-overlay-rgb) / 80%);

  position: relative;
  isolation: isolate;
  overflow: hidden;
  background: var(--confirm-shell-bg) !important;
  border: 1px solid var(--confirm-shell-border) !important;
  box-shadow: var(--confirm-shell-shadow) !important;
}

:root[class~='dark'] .confirm-modal,
[data-theme='dark'] .confirm-modal {
  --confirm-shell-bg: linear-gradient(180deg, rgb(23 26 43 / 98%), rgb(16 18 31 / 96%));
  --confirm-shell-border: rgb(78 84 115 / 78%);
  --confirm-shell-shadow: 0 32px 84px rgb(0 0 0 / 58%), 0 18px 36px rgb(0 0 0 / 42%);
  --confirm-shell-highlight:
    radial-gradient(circle at top center, rgb(var(--color-accent-primary-rgb) / 16%), transparent 42%),
    radial-gradient(circle at bottom right, rgb(var(--color-warning-rgb) / 12%), transparent 38%);
  --confirm-text-primary: var(--color-text-primary);
  --confirm-text-secondary: var(--color-text-secondary);
  --confirm-hairline: rgb(78 84 115 / 74%);
  --confirm-muted-bg: rgb(42 47 74 / 56%);
  --confirm-muted-hover: rgb(53 60 92 / 72%);
}

.confirm-modal::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--confirm-shell-highlight);
  pointer-events: none;
  z-index: 0;
}

.confirm-modal > * {
  position: relative;
  z-index: 1;
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
