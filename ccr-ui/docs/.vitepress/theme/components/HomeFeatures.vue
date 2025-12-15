<script setup lang="ts">
import FeatureCard from './FeatureCard.vue'
import CategoryBadge from './CategoryBadge.vue'

defineProps<{
  badge?: string
  badgeType?: 'brand' | 'info' | 'success' | 'warning' | 'danger'
  title?: string
  features: Array<{
    icon: string
    title: string
    details: string
    link?: string
  }>
}>()
</script>

<template>
  <div class="home-features">
    <div class="home-features__header" v-if="badge || title">
      <CategoryBadge v-if="badge" :text="badge" :type="badgeType" />
      <h2 v-if="title" class="home-features__title">{{ title }}</h2>
    </div>
    <div class="home-features__grid">
      <FeatureCard
        v-for="(feature, index) in features"
        :key="index"
        :icon="feature.icon"
        :title="feature.title"
        :details="feature.details"
        :link="feature.link"
      />
    </div>
  </div>
</template>

<style scoped>
.home-features {
  margin: 48px 0;
}

.home-features__header {
  margin-bottom: 24px;
}

.home-features__title {
  margin: 12px 0 0 0;
  font-size: 28px;
  font-weight: 700;
  color: var(--vp-c-text-1);
}

.home-features__grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

@media (max-width: 768px) {
  .home-features__grid {
    grid-template-columns: 1fr;
  }
  
  .home-features__title {
    font-size: 24px;
  }
}
</style>
