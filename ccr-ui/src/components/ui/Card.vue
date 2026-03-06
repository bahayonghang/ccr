<template>
  <div
    class="relative overflow-hidden transition-[transform,box-shadow] duration-500"
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
    base: 'rounded-xl bg-black/20 dark:bg-black/40 border border-white/10 backdrop-blur-xl',
    elevated: 'rounded-2xl bg-black/20 dark:bg-black/40 border border-white/10 backdrop-blur-xl shadow-2xl',
    glass: 'rounded-2xl bg-white/10 border border-white/20 backdrop-blur-md shadow-xl text-white',
    outline: 'rounded-xl bg-transparent border border-white/20 backdrop-blur-md',
    neko: 'rounded-2xl bg-black/30 dark:bg-black/50 border border-accent-primary/20 backdrop-blur-xl shadow-lg overflow-visible mt-4 neko-border-glow',
  }
  return map[props.variant]
})
</script>
