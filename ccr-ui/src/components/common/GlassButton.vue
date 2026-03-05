<template>
  <button
    class="glass-btn relative inline-flex items-center justify-center gap-2 rounded-full font-medium transition-all duration-300 active:scale-95 focus:outline-none focus-visible:ring-2 focus-visible:ring-white/50 disabled:opacity-50 disabled:cursor-not-allowed disabled:active:scale-100 group overflow-hidden"
    :class="[variantClasses, sizeClasses]"
    :type="type"
    :disabled="disabled || loading"
  >
    <!-- Background overlay for hover effects without reflow -->
    <div 
      class="absolute inset-0 rounded-full transition-opacity duration-300 pointer-events-none opacity-0 group-hover:opacity-100"
      :class="hoverOverlayClasses"
    ></div>

    <span v-if="loading" class="absolute inset-0 flex items-center justify-center">
      <svg class="animate-spin h-5 w-5 text-current" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
    </span>
    
    <span class="relative z-10 flex items-center gap-2" :class="{ 'opacity-0': loading }">
      <slot name="icon-left" />
      <slot />
      <slot name="icon-right" />
    </span>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  variant: {
    type: String,
    default: 'primary' // primary, secondary, danger, ghost
  },
  size: {
    type: String,
    default: 'md' // sm, md, lg
  },
  type: {
    type: String as () => 'button' | 'submit' | 'reset',
    default: 'button'
  },
  disabled: {
    type: Boolean,
    default: false
  },
  loading: {
    type: Boolean,
    default: false
  }
})

const sizeClasses = computed(() => {
  switch (props.size) {
    case 'sm': return 'text-sm px-4 py-2'
    case 'lg': return 'text-lg px-8 py-4'
    case 'md':
    default: return 'text-base px-6 py-3'
  }
})

const variantClasses = computed(() => {
  switch (props.variant) {
    case 'primary':
      return 'bg-accent-primary text-white shadow-lg shadow-accent-primary/20 border border-white/10'
    case 'danger':
      return 'bg-accent-danger text-white shadow-lg shadow-accent-danger/20 border border-white/10'
    case 'ghost':
      return 'bg-transparent text-white border border-transparent'
    case 'secondary':
    default:
      // Glassmorphism secondary
      return 'bg-white/10 text-white border border-white/10 backdrop-blur-md shadow-sm'
  }
})

const hoverOverlayClasses = computed(() => {
  switch (props.variant) {
    case 'primary':
    case 'danger':
      return 'bg-white/10'
    case 'secondary':
      return 'bg-white/20'
    case 'ghost':
      return 'bg-white/10'
    default:
      return 'bg-white/20'
  }
})
</script>

<style scoped>
.glass-btn {
  will-change: transform, opacity;
  transform: translateZ(0); /* Force GPU acceleration */
}
</style>
