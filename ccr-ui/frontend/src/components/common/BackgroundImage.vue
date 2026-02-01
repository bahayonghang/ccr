<template>
  <div class="fixed inset-0 -z-20 overflow-hidden bg-bg-base">
    <!-- Background Image - Only show in dark mode -->
    <div
      v-if="isDarkMode"
      class="absolute inset-0 bg-cover bg-center bg-no-repeat transition-opacity duration-1000"
      :style="{
        backgroundImage: `url(${currentImage})`,
        opacity: imageLoaded ? 1 : 0
      }"
    />

    <!-- Dark mode overlays -->
    <template v-if="isDarkMode">
      <!-- Gradient Overlay - 从左到右渐变遮罩 -->
      <div class="absolute inset-0 bg-gradient-to-r from-slate-900/95 via-slate-900/85 to-slate-900/70" />

      <!-- Top gradient for better header visibility -->
      <div class="absolute inset-x-0 top-0 h-32 bg-gradient-to-b from-slate-900/90 to-transparent" />

      <!-- Bottom gradient -->
      <div class="absolute inset-x-0 bottom-0 h-32 bg-gradient-to-t from-slate-900/90 to-transparent" />
    </template>

    <!-- Light mode - subtle gradient background -->
    <template v-else>
      <div class="absolute inset-0 bg-gradient-to-br from-slate-50 via-white to-indigo-50/30" />
      <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,rgba(99,102,241,0.08),transparent_50%)]" />
    </template>

    <!-- Noise texture overlay (both modes, lighter in light mode) -->
    <div
      class="absolute inset-0 mix-blend-overlay pointer-events-none"
      :class="isDarkMode ? 'opacity-[0.03]' : 'opacity-[0.02]'"
      style="background-image: url('data:image/svg+xml,%3Csvg viewBox=%220 0 200 200%22 xmlns=%22http://www.w3.org/2000/svg%22%3E%3Cfilter id=%22noise%22%3E%3CfeTurbulence type=%22fractalNoise%22 baseFrequency=%220.65%22 numOctaves=%223%22 stitchTiles=%22stitch%22/%3E%3C/filter%3E%3Crect width=%22100%25%22 height=%22100%25%22 filter=%22url(%23noise)%22/%3E%3C/svg%3E')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

// 检测暗色模式
const isDarkMode = ref(false)

const checkDarkMode = () => {
  isDarkMode.value = document.documentElement.getAttribute('data-theme') === 'dark' ||
    document.documentElement.classList.contains('dark')
}

// 监听主题变化
let observer: MutationObserver | null = null

onMounted(() => {
  checkDarkMode()

  // 监听 html 元素的 class 和 data-theme 变化
  observer = new MutationObserver(checkDarkMode)
  observer.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ['class', 'data-theme']
  })

  // 只在暗色模式下预加载图片
  if (isDarkMode.value) {
    preloadImage()
  }
})

onUnmounted(() => {
  observer?.disconnect()
})

// 二次元风格背景图片列表
const backgroundImages = [
  'https://images.unsplash.com/photo-1578632767115-351597cf2477?w=1920&q=80',
  'https://images.unsplash.com/photo-1534796636912-3b95b3ab5986?w=1920&q=80',
  'https://images.unsplash.com/photo-1519681393784-d120267933ba?w=1920&q=80',
]

const currentImage = ref(backgroundImages[0])
const imageLoaded = ref(false)

const preloadImage = () => {
  const randomIndex = Math.floor(Math.random() * backgroundImages.length)
  currentImage.value = backgroundImages[randomIndex]

  const img = new Image()
  img.onload = () => {
    imageLoaded.value = true
  }
  img.src = currentImage.value
}
</script>
