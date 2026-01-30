<template>
  <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10 bg-bg-base transition-colors duration-500">
    <!-- 1. Base Mesh Gradient (Subtle shifting colors) -->
    <div class="absolute inset-0 opacity-30 dark:opacity-20">
      <div class="absolute top-0 left-0 w-full h-full bg-[radial-gradient(circle_at_50%_0%,var(--accent-primary)_0%,transparent_50%)] animate-pulse-slow" />
      <div class="absolute bottom-0 right-0 w-full h-full bg-[radial-gradient(circle_at_100%_100%,var(--accent-secondary)_0%,transparent_50%)] animate-pulse-slow delay-1000" />
      <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full h-full bg-[radial-gradient(circle_at_50%_50%,var(--accent-info)_0%,transparent_60%)] opacity-50 animate-pulse-slower" />
    </div>

    <!-- 2. Cyber Grid Pattern -->
    <div class="absolute inset-0 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:4rem_4rem] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)]" />

    <!-- 3. Noise Texture (Film Grain Effect) -->
    <div
      class="absolute inset-0 opacity-[0.03] dark:opacity-[0.05] mix-blend-overlay" 
      style="background-image: url('data:image/svg+xml,%3Csvg viewBox=%220 0 200 200%22 xmlns=%22http://www.w3.org/2000/svg%22%3E%3Cfilter id=%22noise%22%3E%3CfeTurbulence type=%22fractalNoise%22 baseFrequency=%220.65%22 numOctaves=%223%22 stitchTiles=%22stitch%22/%3E%3C/filter%3E%3Crect width=%22100%25%22 height=%22100%25%22 filter=%22url(%23noise)%22 opacity=%221%22/%3E%3C/svg%3E')"
    />
  </div>
</template>

<script setup lang="ts">
defineProps<{
  complex?: boolean
}>()
</script>

<style scoped>
.animate-pulse-slow {
  animation: pulse-glow 8s ease-in-out infinite;
}

.animate-pulse-slower {
  animation: pulse-glow 12s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%,
  100% {
    opacity: 0.3;
    transform: scale(1);
  }

  50% {
    opacity: 0.5;
    transform: scale(1.1);
  }
}
</style>
