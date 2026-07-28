<template>
  <div
    class="empty-state flex min-h-[300px] flex-col items-center justify-center p-12 text-center"
    role="status"
    aria-live="polite"
  >
    <div
      class="empty-state__icon mb-4 flex h-20 w-20 items-center justify-center rounded-full text-text-muted"
      aria-hidden="true"
    >
      <SIcon :name="icon" />
    </div>
    <h3 class="mb-2 text-xl font-semibold text-text-primary">
      {{ title }}
    </h3>
    <p
      v-if="description"
      class="mb-4 max-w-[480px] text-base text-text-secondary"
    >
      {{ description }}
    </p>
    <button
      v-if="actionText && onAction"
      class="inline-flex min-h-[44px] items-center justify-center gap-2 rounded-full border border-accent-primary/15 bg-accent-primary px-5 py-2.5 text-base font-medium text-text-inverted transition-[background-color,transform] duration-200 ease-out hover:-translate-y-px hover:bg-accent-primary-hover active:translate-y-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30"
      @click="onAction"
    >
      <SIcon
        v-if="actionIcon"
        :name="actionIcon"
        size="w-[18px] h-[18px]"
      />
      {{ actionText }}
    </button>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

interface Props {
  icon?: string
  title: string
  description?: string
  actionText?: string
  actionIcon?: string
  onAction?: () => void
}

withDefaults(defineProps<Props>(), {
  icon: () => 'FileX',
})
</script>

<style scoped>
.empty-state {
  border: 1px solid var(--surface-card-border);
  border-radius: var(--radius-2xl);
  background: var(--surface-card-bg);
  box-shadow: var(--surface-card-shadow), var(--glass-inner-glow);
  backdrop-filter: var(--surface-card-blur);
}

.empty-state__icon {
  border: 1px solid rgb(var(--color-border-default-rgb) / 10%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 100%), rgb(var(--color-bg-surface-rgb) / 88%));
  box-shadow: var(--inner-glow);
}
</style>
