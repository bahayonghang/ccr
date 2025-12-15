<template>
  <button
    class="btn-enhanced relative font-semibold transition-all flex items-center justify-center space-x-2 touch-optimized"
    :class="[
      ...buttonClasses,
      {
        'animate-gpu-accelerated': !disabled,
        'focus:outline-none focus:ring-2 focus:ring-offset-2': true
      }
    ]"
    :style="{
      ...buttonStyle,
      borderRadius: 'var(--radius-lg)',
      transition: `all var(--duration-normal) var(--ease-out-cubic)`,
      '--tw-ring-offset-color': 'var(--bg-primary)',
      '--tw-ring-color': 'var(--accent-primary)'
    }"
    :disabled="disabled"
    :aria-label="ariaLabel"
    @click="handleClick"
  >
    <slot />

    <!-- Loading spinner -->
    <div
      v-if="loading"
      class="absolute inset-0 flex items-center justify-center"
      :style="{ background: 'inherit', borderRadius: 'inherit' }"
    >
      <div
        class="w-4 h-4 border-2 border-current border-t-transparent rounded-full"
        style="animation: spin 1s linear infinite;"
      />
    </div>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
interface Props {
  variant?: 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  fullWidth?: boolean;
  loading?: boolean;
  ariaLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  fullWidth: false,
  loading: false,
  ariaLabel: '',
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

const variantClasses = {
  primary: 'bg-gradient-to-r from-accent-primary to-accent-secondary text-white shadow-lg hover:shadow-xl',
  secondary: 'bg-bg-tertiary text-text-primary border border-border-color hover:bg-bg-secondary',
  success: 'bg-gradient-to-r from-accent-success to-accent-info text-white shadow-lg',
  warning: 'bg-gradient-to-r from-accent-warning to-orange-500 text-white shadow-lg',
  danger: 'bg-gradient-to-r from-accent-danger to-rose-600 text-white shadow-lg',
  ghost: 'bg-transparent text-accent-primary hover:bg-bg-secondary',
}

const sizeClasses = {
  sm: 'text-sm px-3 py-1.5',
  md: 'text-base px-4 py-2',
  lg: 'text-lg px-6 py-3',
}

const buttonClasses = computed(() => [
  variantClasses[props.variant],
  sizeClasses[props.size],
  props.fullWidth ? 'w-full' : '',
  {
    'opacity-50 cursor-not-allowed': props.disabled || props.loading,
    'cursor-wait': props.loading,
  }
])

const buttonStyle = computed(() => ({
  boxShadow: props.variant === 'primary' ? '0 0 20px var(--glow-primary)' : 'none',
  padding: props.size === 'sm' ? 'var(--space-sm) var(--space-md)' :
          props.size === 'md' ? 'var(--space-md) var(--space-lg)' :
          'var(--space-lg) var(--space-xl)',
}))

const handleClick = (event: MouseEvent) => {
  if (!props.disabled && !props.loading) {
    emit('click', event)
  }
}
</script>