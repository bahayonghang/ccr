<template>
  <div
    class="rounded-md p-2"
    :style="{
      background: 'var(--bg-secondary)',
      border: '1px solid var(--border-color)'
    }"
  >
    <div
      class="text-[10px] uppercase tracking-wide font-semibold mb-0.5"
      :style="{ color: 'var(--text-muted)' }"
    >
      {{ label }}
    </div>
    <div
      class="text-xs font-mono truncate"
      :style="{ color: 'var(--text-primary)' }"
      :title="value"
    >
      {{ value }}
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  label: string
  value: string
}

defineProps<Props>()
</script>
