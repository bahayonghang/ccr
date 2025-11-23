<template>
  <div
    class="guofeng-card group relative overflow-hidden rounded-xl transition-all duration-300"
    :class="[
      interactive ? 'hover:-translate-y-1 hover:shadow-lg cursor-pointer' : '',
      variant === 'glass' ? 'glass-effect' : 'bg-[var(--bg-secondary)]',
      className
    ]"
    :style="{
      border: '1px solid var(--border-color)',
      ...style
    }"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <!-- Decorative background pattern (optional) -->
    <div
      v-if="pattern"
      class="absolute inset-0 opacity-5 pointer-events-none"
      :style="{
        backgroundImage: BACKGROUND_PATTERN
      }"
    />

    <!-- Content -->
    <div class="relative z-10 p-5">
      <slot />
    </div>

    <!-- Hover glow effect -->
    <div
      v-if="interactive"
      class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"
      :style="{
        background: 'radial-gradient(circle at center, var(--glow-primary) 0%, transparent 70%)'
      }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'default' | 'glass'
  interactive?: boolean
  pattern?: boolean
  className?: string
  style?: Record<string, any>
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  interactive: false,
  pattern: false,
  className: '',
  style: () => ({})
})

const emit = defineEmits(['mouseenter', 'mouseleave'])

const BACKGROUND_PATTERN = `url("data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cg fill='%23000000' fill-opacity='1'%3E%3Cpath d='M36 34v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zm0-30V0h-2v4h-4v2h4v4h2V6h4V4h-4zM6 34v-4H4v4H0v2h4v4h2v-4h4v-2H6zM6 4V0H4v4H0v2h4v4h2V6h4V4H6z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E")`

const onMouseEnter = (e: MouseEvent) => {
  if (props.interactive) {
    emit('mouseenter', e)
  }
}

const onMouseLeave = (e: MouseEvent) => {
  if (props.interactive) {
    emit('mouseleave', e)
  }
}
</script>

<style scoped>
.guofeng-card {
  position: relative;
}

.guofeng-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--border-color), transparent);
  opacity: 0.5;
}
</style>
