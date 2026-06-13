<template>
  <Transition
    enter-active-class="transition-all duration-200 ease-out"
    enter-from-class="translate-y-4 opacity-0"
    enter-to-class="translate-y-0 opacity-100"
    leave-active-class="transition-all duration-150 ease-in"
    leave-from-class="translate-y-0 opacity-100"
    leave-to-class="translate-y-4 opacity-0"
  >
    <div
      v-if="selectedCount > 0"
      class="multi-select-bar"
    >
      <span class="multi-select-bar__count">
        {{ countLabel || `${selectedCount} / ${totalCount} selected` }}
      </span>

      <div class="multi-select-bar__actions">
        <slot />
        <button
          v-if="showDelete"
          type="button"
          class="multi-select-bar__btn multi-select-bar__btn--danger"
          :aria-label="deleteAriaLabel || `Delete ${selectedCount} items`"
          @click="$emit('delete')"
        >
          <SIcon
            name="Trash2"
            size="w-4 h-4"
          />
          <span>{{ deleteLabel || 'Delete' }}</span>
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

withDefaults(defineProps<{
  /** 已选数量 */
  selectedCount: number
  /** 总数量 */
  totalCount: number
  /** 是否显示删除按钮 */
  showDelete?: boolean
  countLabel?: string
  deleteLabel?: string
  deleteAriaLabel?: string
}>(), {
  showDelete: true,
  countLabel: undefined,
  deleteLabel: undefined,
  deleteAriaLabel: undefined,
})

defineEmits<{
  delete: []
}>()
</script>

<style scoped>
.multi-select-bar {
  position: absolute;
  bottom: 0.75rem;
  left: 0.75rem;
  right: 0.75rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.625rem 1rem;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 20%);
  background: var(--surface-card-bg, rgb(var(--color-bg-elevated-rgb) / 92%));
  box-shadow: var(--elevation-2);
  z-index: var(--layer-sticky);
}

.multi-select-bar__count {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.multi-select-bar__actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.multi-select-bar__btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  border-radius: 0.625rem;
  font-size: 0.8125rem;
  font-weight: 500;
  border: 1px solid var(--surface-status-border, rgb(var(--color-border-default-rgb) / 55%));
  background: var(--surface-status-bg);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.multi-select-bar__btn:hover {
  color: var(--color-text-primary);
  background: var(--surface-card-bg);
}

.multi-select-bar__btn--danger {
  color: rgb(239 68 68 / 85%);
  border-color: rgb(239 68 68 / 20%);
}

.multi-select-bar__btn--danger:hover {
  color: rgb(239 68 68);
  background: rgb(239 68 68 / 8%);
  border-color: rgb(239 68 68 / 30%);
}
</style>
