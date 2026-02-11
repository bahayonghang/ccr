<template>
  <div
    class="relative overflow-hidden transition-all duration-500 will-change-transform"
    :class="[
      variantClasses,
      hover ? 'hover:scale-[1.01] hover:shadow-glow-primary/20' : '',
      className
    ]"
  >
    <!-- Neko Breathing Border Effect (Optional) -->
    <div
      v-if="glow"
      class="absolute inset-0 bg-gradient-to-tr from-accent-primary/20 via-transparent to-accent-secondary/20 opacity-0 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none"
    />

    <!-- Neko Ears for neko variant -->
    <template v-if="variant === 'neko'">
      <div
        class="absolute -top-3 left-5 w-6 h-6 bg-accent-primary z-10 transition-transform duration-300 hover:scale-110"
        style="clip-path: polygon(50% 0%, 0% 100%, 100% 100%); transform: rotate(-15deg);"
      />
      <div
        class="absolute -top-3 right-5 w-6 h-6 bg-accent-primary z-10 transition-transform duration-300 hover:scale-110"
        style="clip-path: polygon(50% 0%, 0% 100%, 100% 100%); transform: rotate(15deg);"
      />
    </template>

    <div class="relative z-10 h-full">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'base' | 'elevated' | 'glass' | 'outline' | 'neko'
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
    base: 'rounded-xl bg-bg-base border border-border-subtle',
    elevated: 'rounded-2xl bg-bg-elevated border border-border-subtle shadow-lg',
    glass: 'rounded-2xl liquid-glass',
    outline: 'rounded-xl bg-transparent border border-border-default',
    neko: 'rounded-2xl bg-bg-elevated border border-accent-primary/20 shadow-lg overflow-visible mt-4 neko-border-glow',
  }
  return map[props.variant]
})
</script>
