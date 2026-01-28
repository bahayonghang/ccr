<template>
  <div
    class="flex flex-col items-center justify-center p-12 text-center min-h-[300px] glass-effect"
    role="status"
    aria-live="polite"
  >
    <div
      class="flex items-center justify-center w-20 h-20 mb-4 rounded-full bg-bg-surface text-text-muted"
      aria-hidden="true"
    >
      <component
        :is="icon"
        :size="48"
        :stroke-width="1.5"
      />
    </div>
    <h3 class="text-xl font-semibold text-text-primary mb-2">
      {{ title }}
    </h3>
    <p
      v-if="description"
      class="text-base text-text-secondary max-w-[480px] mb-4"
    >
      {{ description }}
    </p>
    <button
      v-if="actionText && onAction"
      class="inline-flex items-center justify-center gap-2 px-4 py-2 text-base font-medium text-white bg-accent-primary border-none rounded-md cursor-pointer transition-all duration-200 ease-out hover:bg-accent-primary-hover hover:-translate-y-px hover:shadow-glow-primary active:translate-y-0 min-h-[44px]"
      @click="onAction"
    >
      <component
        :is="actionIcon"
        v-if="actionIcon"
        class="w-[18px] h-[18px]"
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
