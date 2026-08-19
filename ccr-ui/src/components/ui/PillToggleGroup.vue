<template>
  <div
    class="pill-toggle-group"
    role="radiogroup"
    :aria-label="ariaLabel"
  >
    <button
      v-for="option in options"
      :key="String(option.value)"
      type="button"
      class="pill-toggle-group__item"
      :class="{ 'pill-toggle-group__item--active': isActive(option.value) }"
      role="radio"
      :aria-checked="isActive(option.value)"
      :disabled="option.disabled"
      @click="select(option.value)"
    >
      {{ option.label }}
    </button>
  </div>
</template>

<script setup lang="ts" generic="T extends string | number">
export interface PillToggleOption<TValue extends string | number = string> {
  value: TValue
  label: string
  disabled?: boolean
}

interface Props {
  options: PillToggleOption<T>[]
  modelValue?: T
  ariaLabel?: string
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: undefined,
  ariaLabel: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: T]
}>()

const isActive = (value: T) => props.modelValue === value

const select = (value: T) => {
  if (props.modelValue === value) return
  emit('update:modelValue', value)
}
</script>

<style scoped>
.pill-toggle-group {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-full);
  background: var(--color-bg-elevated);
}

.pill-toggle-group__item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 2rem;
  padding: 0.25rem 0.75rem;
  border: 1px solid transparent;
  border-radius: var(--radius-full);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.24;
  letter-spacing: 0;
  cursor: pointer;
  transition:
    color 150ms ease,
    background-color 150ms ease,
    border-color 150ms ease;
}

.pill-toggle-group__item:hover:not(:disabled) {
  color: var(--color-text-primary);
  background: rgb(var(--color-bg-surface-rgb) / 72%);
}

.pill-toggle-group__item:focus-visible {
  outline: 2px solid var(--color-accent-primary);
  outline-offset: 2px;
}

.pill-toggle-group__item:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.pill-toggle-group__item--active {
  color: var(--color-text-primary);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
}
</style>
