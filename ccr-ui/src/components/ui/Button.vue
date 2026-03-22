<template>
  <button
    :type="type"
    :class="classes"
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
  const base = [
    'inline-flex min-h-[44px] items-center justify-center rounded-xl font-medium transition-[color,background-color,border-color,transform,box-shadow] duration-300 ease-out',
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-base',
    'disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none',
    'transform active:scale-95',
    props.block ? 'w-full' : '',
  ]

  const variants = {
    primary: 'border border-accent-primary/20 bg-accent-primary/90 text-text-inverted shadow-glow-primary hover:bg-accent-primary',
    secondary: 'border border-border-default/70 bg-bg-elevated/75 text-text-primary shadow-sm backdrop-blur-md hover:border-border-strong hover:bg-bg-surface',
    accent: 'border border-accent-secondary/20 bg-accent-secondary/90 text-text-inverted shadow-glow-primary hover:bg-accent-secondary',
    outline: 'border border-border-default/80 bg-transparent text-text-secondary hover:border-accent-primary/35 hover:bg-bg-surface/70 hover:text-text-primary',
    ghost: 'text-text-secondary hover:bg-bg-surface/80 hover:text-text-primary',
    glass: 'surface-status border border-border-default/60 text-text-primary shadow-sm hover:border-accent-primary/30 hover:bg-bg-elevated/80',
    danger: 'border border-accent-danger/20 bg-accent-danger/90 text-text-inverted shadow-glow-danger hover:bg-accent-danger',
  }

  const sizes = {
    sm: 'px-3 py-2 text-xs',
    md: 'px-4 py-2.5 text-sm',
    lg: 'px-6 py-3 text-base',
    icon: 'min-w-[44px] p-2.5',
  }

  return [...base, variants[props.variant], sizes[props.size]].join(' ')
})
</script>
