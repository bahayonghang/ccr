<script setup lang="ts">
defineProps<{
  icon: string
  title: string
  details: string
  link?: string
}>()
</script>

<template>
  <component
    :is="link ? 'a' : 'div'"
    :href="link"
    class="feature-card"
    :class="{ 'feature-card--link': link }"
  >
    <div class="feature-card__icon">
      <span v-html="icon"></span>
    </div>
    <div class="feature-card__content">
      <h3 class="feature-card__title">{{ title }}</h3>
      <p class="feature-card__details">{{ details }}</p>
    </div>
  </component>
</template>

<style scoped>
.feature-card {
  display: flex;
  gap: 16px;
  padding: 20px;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-border);
  border-radius: 12px;
  transition: all 0.25s ease;
  text-decoration: none;
  color: inherit;
}

.feature-card--link:hover {
  border-color: var(--vp-c-brand-1);
  transform: translateY(-2px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.1);
}

.dark .feature-card--link:hover {
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.3);
}

.feature-card__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  background: var(--vp-c-brand-soft);
  border-radius: 10px;
  font-size: 24px;
  flex-shrink: 0;
  color: var(--vp-c-brand-1);
}

.feature-card__content {
  flex: 1;
  min-width: 0;
}

.feature-card__title {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  line-height: 1.4;
  color: var(--vp-c-text-1);
}

.feature-card__details {
  margin: 0;
  font-size: 14px;
  line-height: 1.5;
  color: var(--vp-c-text-2);
}
</style>
