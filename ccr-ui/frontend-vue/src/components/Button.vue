<template>
  <button
    class="px-4 py-2 rounded-lg font-semibold transition-all flex items-center justify-center space-x-2 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
    :class="buttonClasses"
    :style="buttonStyle"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
interface Props {
  variant?: 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  fullWidth?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  fullWidth: false,
})

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

const buttonClasses = [
  variantClasses[props.variant],
  sizeClasses[props.size],
  props.fullWidth ? 'w-full' : '',
]

const buttonStyle = {
  boxShadow: props.variant === 'primary' ? '0 0 20px var(--glow-primary)' : 'none',
}
</script>