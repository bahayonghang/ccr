<template>
  <div class="fixed inset-0 w-full h-full -z-50 overflow-hidden transition-colors duration-1000 bg-black">
    <!-- Image layer -->
    <div
      v-show="isLoaded && bgUrl"
      class="absolute inset-0 w-full h-full bg-cover bg-center bg-no-repeat transition-opacity duration-1000"
      :style="{ backgroundImage: `url(${bgUrl})`, opacity: isLoaded ? 1 : 0 }"
    />
    
    <!-- Gradient fallback -->
    <div
      v-show="!isLoaded || hasError"
      class="absolute inset-0 w-full h-full bg-gradient-to-br from-gray-900 via-slate-900 to-black transition-opacity duration-1000"
      :class="{ 'opacity-100': !isLoaded || hasError, 'opacity-0': isLoaded && !hasError }"
    />

    <!-- Dark Overlay for readablity -->
    <div class="absolute inset-0 w-full h-full bg-black/60 z-0 pointer-events-none" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { logger } from '@/utils/logger'

const bgUrl = ref('')
const isLoaded = ref(false)
const hasError = ref(false)

const SESSION_KEY = 'anime_bg_cache'
const CACHE_EXPIRE_MS = 15 * 60 * 1000 // 15 minutes

interface BgCache {
  url: string
  timestamp: number
}

const loadBackground = async () => {
  try {
    // 1. Check cache
    const cachedData = sessionStorage.getItem(SESSION_KEY)
    if (cachedData) {
      const cache: BgCache = JSON.parse(cachedData)
      const now = Date.now()
      if (now - cache.timestamp < CACHE_EXPIRE_MS) {
        preloadImage(cache.url)
        return
      }
    }

    // 2. Fetch new API
    // type=json returns { "code": 200, "url": "...", "width": ..., "height": ... }
    const res = await fetch('https://www.loliapi.com/acg/?type=json')
    if (!res.ok) throw new Error('API request failed')
    const data = await res.json()
    
    if (data && data.url) {
      // 3. Update cache
      sessionStorage.setItem(SESSION_KEY, JSON.stringify({
        url: data.url,
        timestamp: Date.now()
      }))
      preloadImage(data.url)
    } else {
      throw new Error('Invalid API response format')
    }
  } catch (err) {
    logger.error('[AnimeBackground] failed to load', err)
    hasError.value = true
    isLoaded.value = true // stop loading state
  }
}

const preloadImage = (url: string) => {
  const img = new Image()
  img.src = url
  img.onload = () => {
    bgUrl.value = url
    isLoaded.value = true
  }
  img.onerror = () => {
    hasError.value = true
    isLoaded.value = true
  }
}

onMounted(() => {
  loadBackground()
})
</script>

<style scoped>
/* 确保图层不参与主页面重排，且能享用 GPU 加速 */
.bg-cover {
  transform: translateZ(0);
  will-change: opacity;
}
</style>
