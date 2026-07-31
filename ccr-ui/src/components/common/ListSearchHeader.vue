<template>
  <div class="list-search-header">
    <div class="list-search-header__search">
      <SIcon
        name="Search"
        size="w-4 h-4"
        class="text-text-muted"
      />
      <input
        :value="searchValue"
        type="text"
        class="list-search-header__input"
        :placeholder="placeholder"
        :aria-label="label || placeholder"
        @input="$emit('update:searchValue', ($event.target as HTMLInputElement).value)"
      >
    </div>

    <div class="list-search-header__actions">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

withDefaults(defineProps<{
  /** 搜索关键词 (v-model:searchValue) */
  searchValue: string
  /** 占位文本 */
  placeholder?: string
  /** 无障碍标签 */
  label?: string
}>(), {
  placeholder: 'Search...',
  label: undefined,
})

defineEmits<{
  'update:searchValue': [value: string]
}>()
</script>

<style scoped>
.list-search-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  border-bottom: 1px solid var(--surface-workspace-border, rgb(var(--color-border-default-rgb) / 45%));
}

.list-search-header__search {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
  min-width: 0;
  padding: 0.375rem 0.75rem;
  border-radius: 0.75rem;
  border: 1px solid var(--surface-status-border, rgb(var(--color-border-default-rgb) / 55%));
  background: var(--surface-status-bg, rgb(var(--color-bg-elevated-rgb) / 72%));
  backdrop-filter: var(--surface-status-blur, blur(14px));
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.list-search-header__search:focus-within {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.list-search-header__input {
  flex: 1;
  min-width: 0;
  font-size: 0.8125rem;
  line-height: 1.5;
  color: var(--color-text-primary);
  background: transparent;
  border: none;
  outline: none;
}

.list-search-header__input::placeholder {
  color: var(--color-text-ghost);
}

.list-search-header__actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  flex-shrink: 0;
}
</style>
