<script setup lang="ts">
defineProps<{
  text: string
  type?: 'brand' | 'info' | 'success' | 'warning' | 'danger'
}>()
</script>

<template>
  <span class="category-badge" :class="`category-badge--${type || 'brand'}`">
    {{ text }}
  </span>
</template>

<style scoped>
.category-badge {
  display: inline-block;
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.02em;
  border-radius: 6px;
  text-transform: uppercase;
}

.category-badge--brand {
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
}

.category-badge--info {
  background: var(--vp-c-default-soft);
  color: var(--vp-c-text-2);
}

.category-badge--success {
  background: rgba(16, 185, 129, 0.16);
  color: #10b981;
}

.category-badge--warning {
  background: rgba(245, 158, 11, 0.16);
  color: #f59e0b;
}

.category-badge--danger {
  background: rgba(239, 68, 68, 0.16);
  color: #ef4444;
}
</style>
