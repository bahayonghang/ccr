<template>
  <div class="fixed inset-0 -z-20 overflow-hidden bg-bg-base">
    <!-- Dark mode - multi-point radial gradient mesh -->
    <template v-if="isDarkMode">
      <div class="absolute inset-0 bg-gradient-to-r from-slate-900/95 via-slate-900/85 to-slate-900/70" />
      <div class="absolute inset-0 bg-[radial-gradient(ellipse_40%_50%_at_15%_20%,rgb(var(--color-accent-primary-rgb)/0.12),transparent)]" />
      <div class="absolute inset-0 bg-[radial-gradient(ellipse_35%_40%_at_80%_75%,rgb(var(--color-accent-secondary-rgb)/0.10),transparent)]" />
      <div class="absolute inset-0 bg-[radial-gradient(ellipse_50%_30%_at_50%_50%,rgb(var(--color-accent-primary-rgb)/0.05),transparent)]" />
      <div class="absolute inset-x-0 top-0 h-32 bg-gradient-to-b from-slate-900/90 to-transparent" />
      <div class="absolute inset-x-0 bottom-0 h-32 bg-gradient-to-t from-slate-900/90 to-transparent" />
    </template>

    <!-- Light mode - subtle gradient background -->
    <template v-else>
      <div class="absolute inset-0 bg-gradient-to-br from-slate-50 via-white to-indigo-50/30" />
      <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,rgba(99,102,241,0.08),transparent_50%)]" />
    </template>

    <!-- Noise texture overlay (inline SVG, no external resources) -->
    <div
      class="absolute inset-0 mix-blend-overlay pointer-events-none"
      :class="isDarkMode ? 'opacity-[0.03]' : 'opacity-[0.02]'"
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
