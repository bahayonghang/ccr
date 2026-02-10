<template>
  <div 
    class="relative rounded-xl overflow-hidden transition-all duration-500 will-change-transform"
    :class="[
      variantClasses, 
      hover ? 'hover:scale-[1.01] hover:shadow-glow-primary/20' : '',
      className
    ]"
  >
    <!-- Neo Breathing Border Effect (Optional) -->
    <div 
      v-if="glow" 
      class="absolute inset-0 bg-gradient-to-tr from-accent-primary/20 via-transparent to-accent-secondary/20 opacity-0 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none"
    />
    
    <div class="relative z-10 h-full">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'base' | 'elevated' | 'glass' | 'outline'
  hover?: boolean
  glow?: boolean // Enable the inner glow effect
  className?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'elevated',
  hover: true,
  glow: false,
  className: '',
})

const variantClasses = computed(() => {
  const map = {
    base: 'bg-bg-base border border-border-subtle',
    elevated: 'bg-bg-elevated border border-border-subtle shadow-lg',
    glass: 'liquid-glass',
    outline: 'bg-transparent border border-border-default',
  }
  return map[props.variant]
})
</script>
