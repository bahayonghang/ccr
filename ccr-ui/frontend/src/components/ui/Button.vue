<template>
  <button
    :class="classes"
    :disabled="disabled || loading"
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
  disabled?: boolean
  loading?: boolean
  block?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
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
    'inline-flex items-center justify-center rounded-xl font-medium transition-all duration-300 ease-out',
    'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-bg-base',
    'disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none',
    'transform active:scale-95 will-change-transform', // Micro-interaction
    props.block ? 'w-full' : '',
  ]

  const variants = {
    primary: 'bg-accent-primary text-text-inverted hover:bg-accent-primary/90 shadow-glow-primary',
    secondary: 'bg-bg-overlay text-text-primary hover:bg-bg-overlay/80',
    accent: 'bg-accent-secondary text-text-inverted hover:bg-accent-secondary/90 shadow-glow-primary',
    outline: 'border border-border-default text-text-primary hover:border-accent-primary hover:text-accent-primary hover:bg-accent-primary/5',
    ghost: 'text-text-secondary hover:text-text-primary hover:bg-bg-overlay/50',
    glass: 'bg-white/5 backdrop-blur-md border border-white/10 text-text-primary hover:bg-white/10 shadow-lg',
    danger: 'bg-accent-danger text-text-inverted hover:bg-accent-danger/90 shadow-glow-danger',
  }

  const sizes = {
    sm: 'text-xs px-2.5 py-1.5',
    md: 'text-sm px-4 py-2',
    lg: 'text-base px-6 py-3',
    icon: 'p-2',
  }

  return [...base, variants[props.variant], sizes[props.size]].join(' ')
})
</script>
