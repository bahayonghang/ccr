<script setup lang="ts">
import FeatureCard from './FeatureCard.vue'

defineProps<{
  features: Array<{
    icon: string
    title: string
    details: string
    link?: string
  }>
}>()
</script>

<template>
  <div class="feature-grid">
    <FeatureCard
      v-for="(feature, index) in features"
      :key="index"
      :icon="feature.icon"
      :title="feature.title"
      :details="feature.details"
      :link="feature.link"
    />
  </div>
</template>

<style scoped>
.feature-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin: 24px 0;
}

@media (max-width: 768px) {
  .feature-grid {
    grid-template-columns: 1fr;
  }
}
</style>
