<template>
  <div
    class="glass-effect empty-state"
    role="status"
    aria-live="polite"
  >
    <div
      class="icon-wrapper"
      aria-hidden="true"
    >
      <component
        :is="icon"
        :size="48"
        :stroke-width="1.5"
      />
    </div>
    <h3 class="title">
      {{ title }}
    </h3>
    <p
      v-if="description"
      class="description"
    >
      {{ description }}
    </p>
    <button
      v-if="actionText && onAction"
      class="action-btn min-h-[44px]"
      @click="onAction"
    >
      <component
        :is="actionIcon"
        v-if="actionIcon"
        class="action-icon"
        aria-hidden="true"
      />
      {{ actionText }}
    </button>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { FileX } from 'lucide-vue-next'

interface Props {
  icon?: Component
  title: string
  description?: string
  actionText?: string
  actionIcon?: Component
  onAction?: () => void
}

withDefaults(defineProps<Props>(), {
  icon: () => FileX,
})
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-12);
  text-align: center;
  min-height: 300px;
}

.icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 80px;
  margin-bottom: var(--space-4);
  border-radius: var(--radius-full);
  background: var(--color-bg-surface);
  color: var(--color-text-muted);
}

.title {
  font-size: var(--text-xl);
  font-weight: var(--font-semibold);
  color: var(--color-text-primary);
  margin-bottom: var(--space-2);
}

.description {
  font-size: var(--text-base);
  color: var(--color-text-secondary);
  max-width: 480px;
  margin-bottom: var(--space-4);
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  font-size: var(--text-base);
  font-weight: var(--font-medium);
  color: white;
  background: var(--color-accent-primary);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-normal) var(--ease-out);
}

.action-icon {
  width: 18px;
  height: 18px;
}

.action-btn:hover {
  background: var(--color-accent-primary-hover);
  transform: translateY(-1px);
  box-shadow: var(--glow-primary);
}

.action-btn:active {
  transform: translateY(0);
}
</style>
