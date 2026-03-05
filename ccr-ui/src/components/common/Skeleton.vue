<template>
  <div
    :class="['skeleton', `skeleton-${shape}`, animated && 'animate-pulse']"
    :style="{ width, height }"
    role="status"
    :aria-label="ariaLabel"
  >
    <span class="sr-only">{{ ariaLabel }}</span>
  </div>
</template>

<script setup lang="ts">
interface Props {
  shape?: 'rectangle' | 'circle' | 'text'
  width?: string
  height?: string
  animated?: boolean
  ariaLabel?: string
}

withDefaults(defineProps<Props>(), {
  shape: 'rectangle',
  width: '100%',
  height: '1rem',
  animated: true,
  ariaLabel: 'Loading...',
})
</script>

<style scoped>
.skeleton {
  background: var(--color-bg-surface);
  overflow: hidden;
}

.skeleton-rectangle {
  border-radius: var(--radius-md);
}

.skeleton-circle {
  border-radius: var(--radius-full);
  aspect-ratio: 1;
}

.skeleton-text {
  border-radius: var(--radius-sm);
  height: 1rem;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
  border-width: 0;
}
</style>
