<template>
  <button
    :type="type"
    :class="classes"
    :data-variant="variant"
    :data-size="size"
    :disabled="disabled || loading"
    :aria-busy="loading || undefined"
    @click="handleClick"
  >
    <!-- Loading Spinner -->
    <svg 
      v-if="loading" 
      class="animate-spin -ml-1 mr-2 h-4 w-4" 
      xmlns="http://www.w3.org/2000/svg" 
      fill="none" 
      viewBox="0 0 24 24"
    >
      <circle
        class="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        stroke-width="4"
      />
      <path
        class="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      />
    </svg>

    <!-- Leading Icon -->
    <span
      v-if="$slots.leading && !loading"
      class="mr-2 flex items-center"
    >
      <slot name="leading" />
    </span>

    <!-- Content -->
    <slot />

    <!-- Trailing Icon -->
    <span
      v-if="$slots.trailing"
      class="ml-2 flex items-center"
    >
      <slot name="trailing" />
    </span>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'primary' | 'secondary' | 'accent' | 'outline' | 'ghost' | 'glass' | 'danger'
  size?: 'sm' | 'md' | 'lg' | 'icon'
  type?: 'button' | 'submit' | 'reset'
  disabled?: boolean
  loading?: boolean
  block?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  type: 'button',
  disabled: false,
  loading: false,
  block: false,
})

const emit = defineEmits(['click'])

const handleClick = (e: MouseEvent) => {
  if (!props.disabled && !props.loading) {
    emit('click', e)
  }
}

const classes = computed(() => {
  return [
    'ui-button',
    `ui-button--${props.variant}`,
    `ui-button--${props.size}`,
    props.block ? 'ui-button--block' : '',
  ].join(' ')
})
</script>

<style scoped>
.ui-button {
  @apply inline-flex min-h-[44px] items-center justify-center rounded-xl font-medium;
  @apply transition-[color,background-color,border-color,transform,box-shadow] duration-300 ease-out;
  @apply focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-base;

  transform: translateZ(0);
}

.ui-button:active {
  transform: scale(0.95);
}

.ui-button:disabled {
  @apply cursor-not-allowed opacity-50;

  box-shadow: none;
}

.ui-button--block {
  @apply w-full;
}

.ui-button--sm {
  @apply px-3 py-2 text-xs;
}

.ui-button--md {
  @apply px-4 py-2.5 text-sm;
}

.ui-button--lg {
  @apply px-6 py-3 text-base;
}

.ui-button--icon {
  @apply min-w-[44px] p-2.5;
}

.ui-button--primary {
  @apply border border-accent-primary/20 bg-accent-primary/90 text-text-inverted;

  box-shadow: 0 0 20px rgb(var(--color-accent-primary-rgb) / 24%);
}

.ui-button--primary:hover:not(:disabled) {
  @apply bg-accent-primary;
}

.ui-button--secondary {
  @apply border border-border-default/70 bg-bg-elevated/75 text-text-primary shadow-sm backdrop-blur-md;
}

.ui-button--secondary:hover:not(:disabled) {
  @apply border-border-strong bg-bg-surface;
}

.ui-button--accent {
  @apply border border-accent-secondary/20 bg-accent-secondary/90 text-text-inverted;

  box-shadow: 0 0 20px rgb(var(--color-accent-secondary-rgb) / 24%);
}

.ui-button--accent:hover:not(:disabled) {
  @apply bg-accent-secondary;
}

.ui-button--outline {
  @apply border border-border-default/80 bg-transparent text-text-secondary;
}

.ui-button--outline:hover:not(:disabled) {
  @apply border-accent-primary/35 bg-bg-surface/70 text-text-primary;
}

.ui-button--ghost {
  @apply text-text-secondary;
}

.ui-button--ghost:hover:not(:disabled) {
  @apply bg-bg-surface/80 text-text-primary;
}

.ui-button--glass {
  @apply border border-border-default/60 text-text-primary shadow-sm;

  background: var(--surface-status-bg);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--surface-status-shadow);
}

.ui-button--glass:hover:not(:disabled) {
  @apply border-accent-primary/30 bg-bg-elevated/80;
}

.ui-button--danger {
  @apply border border-accent-danger/20 bg-accent-danger/90 text-text-inverted;

  box-shadow: 0 0 20px rgb(var(--color-danger-rgb) / 24%);
}

.ui-button--danger:hover:not(:disabled) {
  @apply bg-accent-danger;
}
</style>
