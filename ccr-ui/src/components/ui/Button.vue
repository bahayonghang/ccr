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
    'inline-flex items-center justify-center rounded-xl font-medium transition-[color,background-color,border-color,transform,box-shadow] duration-300 ease-out',
    'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-bg-base',
    'disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none',
    'transform active:scale-95', // Micro-interaction
    props.block ? 'w-full' : '',
  ]

  const variants = {
    primary: 'bg-accent-primary/90 text-white hover:bg-accent-primary shadow-glow-primary border border-white/10 backdrop-blur-sm',
    secondary: 'bg-white/10 text-white border border-white/10 hover:bg-white/20 backdrop-blur-md shadow-sm',
    accent: 'bg-accent-secondary/90 text-white hover:bg-accent-secondary shadow-glow-primary border border-white/10 backdrop-blur-sm',
    outline: 'border border-white/30 text-white hover:border-white/60 hover:bg-white/10 backdrop-blur-sm',
    ghost: 'text-white/70 hover:text-white hover:bg-white/10',
    glass: 'bg-white/10 backdrop-blur-md border border-white/20 text-white hover:bg-white/20 shadow-lg',
    danger: 'bg-accent-danger/90 text-white hover:bg-accent-danger shadow-glow-danger border border-white/10 backdrop-blur-sm',
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
