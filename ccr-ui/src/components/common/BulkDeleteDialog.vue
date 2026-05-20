<template>
  <BaseModal
    :model-value="isOpen"
    :title="title"
    :close-on-backdrop="true"
    :close-on-escape="true"
    :show-close="true"
    surface="solid"
    size="sm"
    @update:model-value="handleModalChange"
    @close="$emit('cancel')"
  >
    <div class="bulk-delete__body">
      <div class="bulk-delete__icon-wrap">
        <SIcon
          name="AlertTriangle"
          class="w-6 h-6 text-amber-500"
        />
      </div>

      <p class="bulk-delete__message">
        {{ message || `确认删除选中的 ${items.length} 个${resourceLabel}？此操作不可撤销。` }}
      </p>

      <div
        v-if="items.length > 0 && items.length <= 10"
        class="bulk-delete__list"
      >
        <div
          v-for="item in items"
          :key="item.key"
          class="bulk-delete__item"
        >
          <span class="truncate">{{ item.label }}</span>
          <span
            v-if="item.badge"
            class="bulk-delete__badge"
          >{{ item.badge }}</span>
        </div>
      </div>
      <p
        v-else-if="items.length > 10"
        class="bulk-delete__overflow"
      >
        {{ overflowMessage || `... 以及其他 ${items.length - 10} 项` }}
      </p>
    </div>

    <template #footer>
      <button
        type="button"
        class="bulk-delete__btn bulk-delete__btn--cancel"
        @click="$emit('cancel')"
      >
        {{ cancelLabel || 'Cancel' }}
      </button>
      <button
        type="button"
        class="bulk-delete__btn bulk-delete__btn--confirm"
        :disabled="loading"
        @click="$emit('confirm')"
      >
        <SIcon
          v-if="loading"
          name="Loader2"
          size="w-4 h-4"
          class="animate-spin"
        />
        <SIcon
          v-else
          name="Trash2"
          size="w-4 h-4"
        />
        <span>{{ confirmLabel || `Delete ${items.length}` }}</span>
      </button>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import BaseModal from '@/components/common/BaseModal.vue'
import SIcon from '@/components/ui/SIcon.vue'

export interface BulkDeleteItem {
  key: string
  label: string
  badge?: string
}

withDefaults(defineProps<{
  /** 对话框是否打开 */
  isOpen: boolean
  title?: string
  /** 待删除项列表 */
  items: BulkDeleteItem[]
  /** 资源类型标签 */
  resourceLabel?: string
  /** 自定义消息 */
  message?: string
  overflowMessage?: string
  cancelLabel?: string
  confirmLabel?: string
  /** 加载状态 */
  loading?: boolean
}>(), {
  title: 'Confirm Delete',
  resourceLabel: '项',
  message: undefined,
  overflowMessage: undefined,
  cancelLabel: undefined,
  confirmLabel: undefined,
  loading: false,
})

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

function handleModalChange(value: boolean) {
  if (!value) emit('cancel')
}
</script>

<style scoped>
.bulk-delete__body {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  text-align: center;
  padding: 0.5rem 0;
}

.bulk-delete__icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.75rem;
  height: 2.75rem;
  border-radius: 50%;
  background: rgb(245 158 11 / 10%);
}

.bulk-delete__message {
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--color-text-secondary);
}

.bulk-delete__list {
  width: 100%;
  max-height: 12rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.5rem;
  border-radius: 0.75rem;
  background: rgb(var(--color-bg-base-rgb) / 55%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 45%);
}

.bulk-delete__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 0.375rem 0.5rem;
  border-radius: 0.5rem;
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}

.bulk-delete__badge {
  flex-shrink: 0;
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  font-size: 0.6875rem;
  background: var(--surface-status-bg);
  border: 1px solid var(--surface-status-border);
  color: var(--color-text-secondary);
}

.bulk-delete__overflow {
  font-size: 0.75rem;
  color: var(--color-text-muted);
}

.bulk-delete__btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 1rem;
  border-radius: 0.75rem;
  font-size: 0.8125rem;
  font-weight: 500;
  cursor: pointer;
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.bulk-delete__btn--cancel {
  border: 1px solid var(--surface-status-border);
  background: var(--surface-status-bg);
  color: var(--color-text-secondary);
}

.bulk-delete__btn--cancel:hover {
  background: var(--surface-card-bg);
  color: var(--color-text-primary);
}

.bulk-delete__btn--confirm {
  border: 1px solid rgb(239 68 68 / 30%);
  background: rgb(239 68 68 / 10%);
  color: rgb(239 68 68);
}

.bulk-delete__btn--confirm:hover:not(:disabled) {
  background: rgb(239 68 68 / 18%);
  border-color: rgb(239 68 68 / 45%);
}

.bulk-delete__btn--confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
