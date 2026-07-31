<!-- Shared profile group container: heading, count, and content slot. -->
<template>
  <section class="cp-section">
    <div class="cp-section__head">
      <SIcon
        name="Folder"
        size="w-3.5 h-3.5"
        class="cp-section__icon"
      />
      <span class="cp-section__title">{{ title }}</span>
      <span class="cp-section__count">{{ count }}</span>
    </div>
    <div class="cp-section__body">
      <slot />
    </div>
  </section>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

interface Props {
  title: string
  count: number
}

defineProps<Props>()
</script>

<style scoped>
.cp-section {
  margin-bottom: 18px;
}

.cp-section__head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.cp-section__icon {
  color: var(--cp-ink-3);
}

.cp-section__title {
  font-size: 0.8125rem;
  font-weight: 600;
  letter-spacing: 0.0375rem;
  text-transform: uppercase;
  color: var(--cp-ink-2);
}

.cp-section__count {
  padding: 0.0625rem 0.4375rem;
  border-radius: 999px;
  background: var(--cp-bg-3);
  border: 1px solid var(--cp-line-2);
  color: var(--cp-ink-3);
  font-family: var(--cp-mono);
  font-size: 0.75rem;
}

.cp-section__body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
</style>
