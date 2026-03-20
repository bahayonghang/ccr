<template>
  <div class="fixed inset-0 w-full h-full -z-50 overflow-hidden transition-colors duration-1000 bg-white dark:bg-black">
    <div class="absolute inset-0 w-full h-full bg-gradient-to-br from-pink-50 via-white to-purple-50 dark:from-gray-900 dark:via-slate-900 dark:to-black" />

    <div
      v-if="bgUrl"
      class="absolute inset-0 w-full h-full bg-cover bg-center bg-no-repeat transition-opacity duration-1000"
      :style="{ backgroundImage: `url(${bgUrl})`, opacity: isLoaded ? 1 : 0 }"
    />

    <div class="absolute inset-0 w-full h-full bg-white/78 dark:bg-black/40 z-0 pointer-events-none" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import fallbackBackgroundUrl from '@/assets/anime-background-fallback.svg'
import {
  clearBackgroundCache,
  createBackgroundObjectUrl,
  loadBackgroundCache,
  revokeBackgroundObjectUrl,
  type BackgroundCacheRecord
} from '@/utils/backgroundCache'
import { logger } from '@/utils/logger'

const bgUrl = ref('')
const isLoaded = ref(false)

let currentObjectUrl: string | null = null
let isUnmounted = false

const releaseCurrentObjectUrl = () => {
  revokeBackgroundObjectUrl(currentObjectUrl)
  currentObjectUrl = null
}

const preloadBackground = (url: string, nextObjectUrl: string | null = null): Promise<boolean> => {
  return new Promise((resolve) => {
    const img = new Image()

    img.onload = () => {
      if (isUnmounted) {
        revokeBackgroundObjectUrl(nextObjectUrl)
        resolve(false)
        return
      }

      releaseCurrentObjectUrl()
      currentObjectUrl = nextObjectUrl
      bgUrl.value = url
      isLoaded.value = true
      resolve(true)
    }

    img.onerror = () => {
      revokeBackgroundObjectUrl(nextObjectUrl)
      resolve(false)
    }

    img.src = url
  })
}

const displayFallbackBackground = async (): Promise<void> => {
  const displayed = await preloadBackground(fallbackBackgroundUrl)

  if (!displayed && !isUnmounted) {
    bgUrl.value = fallbackBackgroundUrl
    isLoaded.value = true
  }
}

const displayCachedBackground = async (record: Pick<BackgroundCacheRecord, 'blob'>): Promise<boolean> => {
  const objectUrl = createBackgroundObjectUrl(record)
  return preloadBackground(objectUrl, objectUrl)
}

const initializeBackground = async () => {
  try {
    const cached = await loadBackgroundCache()

    if (cached) {
      const displayed = await displayCachedBackground(cached)

      if (!displayed) {
        logger.warn('[AnimeBackground] cached background is invalid, clearing cache')
        await clearBackgroundCache().catch(() => undefined)
      } else {
        return
      }
    }
  } catch (error) {
    logger.warn('[AnimeBackground] failed to load background cache', error)
  }

  await displayFallbackBackground()
}

onMounted(() => {
  void initializeBackground()
})

onUnmounted(() => {
  isUnmounted = true
  releaseCurrentObjectUrl()
})
</script>

<style scoped>
.bg-cover {
  transform: translateZ(0);
  will-change: opacity;
}
</style>
