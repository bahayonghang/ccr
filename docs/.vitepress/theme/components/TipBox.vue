<script setup lang="ts">
defineProps<{
  type?: 'tip' | 'info' | 'warning' | 'danger'
  title?: string
}>()

const icons = {
  tip: '💡',
  info: 'ℹ️',
  warning: '⚠️',
  danger: '🚨'
}
</script>

<template>
  <div class="tip-box" :class="`tip-box--${type || 'tip'}`">
    <div class="tip-box__header" v-if="title">
      <span class="tip-box__icon">{{ icons[type || 'tip'] }}</span>
      <span class="tip-box__title">{{ title }}</span>
    </div>
    <div class="tip-box__content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.tip-box {
  padding: 16px 20px;
  border-radius: 10px;
  margin: 16px 0;
  border-left: 4px solid;
}

.tip-box--tip {
  background: var(--vp-c-brand-soft);
  border-color: var(--vp-c-brand-1);
}

.tip-box--info {
  background: var(--vp-c-default-soft);
  border-color: var(--vp-c-text-3);
}

.tip-box--warning {
  background: rgba(245, 158, 11, 0.1);
  border-color: #f59e0b;
}

.tip-box--danger {
  background: rgba(239, 68, 68, 0.1);
  border-color: #ef4444;
}

.tip-box__header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.tip-box__icon {
  font-size: 16px;
}

.tip-box__title {
  font-weight: 600;
  font-size: 14px;
  color: var(--vp-c-text-1);
}

.tip-box__content {
  font-size: 14px;
  line-height: 1.6;
  color: var(--vp-c-text-2);
}

.tip-box__content :deep(p) {
  margin: 0;
}

.tip-box__content :deep(a) {
  color: var(--vp-c-brand-1);
}
</style>
