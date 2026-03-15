<template>
  <div
    class="glass-effect flex min-h-[300px] flex-col items-center justify-center p-12 text-center"
    role="status"
    aria-live="polite"
  >
    <div
      class="glass-surface mb-4 flex h-20 w-20 items-center justify-center rounded-full text-text-muted"
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
      class="inline-flex min-h-[44px] items-center justify-center gap-2 rounded-lg border border-accent-primary/20 bg-accent-primary/90 px-4 py-2 text-base font-medium text-text-inverted transition-[background-color,transform,box-shadow] duration-200 ease-out hover:-translate-y-px hover:bg-accent-primary hover:shadow-glow-primary active:translate-y-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30"
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
