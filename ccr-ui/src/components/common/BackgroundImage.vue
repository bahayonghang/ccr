<template>
  <div class="fixed inset-0 -z-20 overflow-hidden">
    <!-- Dark mode - multi-point radial gradient mesh -->
    <template v-if="isDarkMode">
      <div class="absolute inset-0 bg-gradient-to-r from-slate-900/95 via-slate-900/85 to-slate-900/70" />
      <div class="absolute inset-0 background-mesh background-mesh--dark-primary" />
      <div class="absolute inset-0 background-mesh background-mesh--dark-secondary" />
      <div class="absolute inset-0 background-mesh background-mesh--dark-center" />
      <div class="absolute inset-x-0 top-0 h-32 bg-gradient-to-b from-slate-900/90 to-transparent" />
      <div class="absolute inset-x-0 bottom-0 h-32 bg-gradient-to-t from-slate-900/90 to-transparent" />
    </template>

    <!-- Light mode - subtle gradient background -->
    <template v-else>
      <div class="absolute inset-0 bg-gradient-to-br from-slate-50 via-white to-indigo-50/30" />
      <div class="absolute inset-0 background-mesh background-mesh--light-accent" />
    </template>

    <!-- Noise texture overlay (inline SVG, no external resources) -->
    <div
      class="absolute inset-0 mix-blend-overlay pointer-events-none"
      :class="isDarkMode ? 'background-noise background-noise--dark' : 'background-noise background-noise--light'"
      style="background-image: url('data:image/svg+xml,%3Csvg viewBox=%220 0 200 200%22 xmlns=%22http://www.w3.org/2000/svg%22%3E%3Cfilter id=%22noise%22%3E%3CfeTurbulence type=%22fractalNoise%22 baseFrequency=%220.65%22 numOctaves=%223%22 stitchTiles=%22stitch%22/%3E%3C/filter%3E%3Crect width=%22100%25%22 height=%22100%25%22 filter=%22url(%23noise)%22/%3E%3C/svg%3E')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const isDarkMode = ref(false)

const checkDarkMode = () => {
  isDarkMode.value = document.documentElement.getAttribute('data-theme') === 'dark' ||
    document.documentElement.classList.contains('dark')
}

let observer: MutationObserver | null = null

onMounted(() => {
  checkDarkMode()
  observer = new MutationObserver(checkDarkMode)
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['class', 'data-theme']
  })
})

onUnmounted(() => {
  observer?.disconnect()
})
</script>

<style scoped>
.background-mesh--dark-primary {
  background: radial-gradient(
    ellipse 40% 50% at 15% 20%,
    rgb(var(--color-accent-primary-rgb) / 12%),
    transparent
  );
}

.background-mesh--dark-secondary {
  background: radial-gradient(
    ellipse 35% 40% at 80% 75%,
    rgb(var(--color-accent-secondary-rgb) / 10%),
    transparent
  );
}

.background-mesh--dark-center {
  background: radial-gradient(
    ellipse 50% 30% at 50% 50%,
    rgb(var(--color-accent-primary-rgb) / 5%),
    transparent
  );
}

.background-mesh--light-accent {
  background: radial-gradient(
    ellipse at top right,
    rgb(99 102 241 / 8%),
    transparent 50%
  );
}

.background-noise {
  opacity: 0.02;
}

.background-noise--dark {
  opacity: 0.03;
}

.background-noise--light {
  opacity: 0.02;
}
</style>
