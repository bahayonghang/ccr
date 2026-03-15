<template>
  <div class="fixed inset-0 w-full h-full -z-50 overflow-hidden transition-colors duration-1000 bg-white dark:bg-black">
    <div class="absolute inset-0 w-full h-full bg-gradient-to-br from-pink-50 via-white to-purple-50 dark:from-gray-900 dark:via-slate-900 dark:to-black" />

    <div
      v-if="bgUrl"
      class="absolute inset-0 w-full h-full bg-cover bg-center bg-no-repeat transition-opacity duration-1000"
      :style="{ backgroundImage: `url(${bgUrl})`, opacity: isLoaded ? 1 : 0 }"
    />

    <div class="absolute inset-0 w-full h-full bg-white/60 dark:bg-black/40 z-0 pointer-events-none" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import fallbackBackgroundUrl from '@/assets/anime-background-fallback.svg'
import {
  clearBackgroundCache,
  consumeBackgroundRefreshAttempt,
  createBackgroundObjectUrl,
  isBackgroundCacheExpired,
  loadBackgroundCache,
  revokeBackgroundObjectUrl,
  saveBackgroundCache,
  type BackgroundCacheRecord,
} from '@/utils/backgroundCache'
import { logger } from '@/utils/logger'

interface LoliApiResponse {
  url?: string
}

const LOLI_API_URL = 'https://www.loliapi.com/acg/?type=json'

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

const fetchBackgroundRecord = async (): Promise<BackgroundCacheRecord> => {
  const apiResponse = await fetch(LOLI_API_URL, { cache: 'no-store' })
  if (!apiResponse.ok) {
    throw new Error(`Background API request failed with status ${apiResponse.status}`)
  }

  const data = await apiResponse.json() as LoliApiResponse
  if (!data.url || typeof data.url !== 'string') {
    throw new Error('Background API returned an invalid payload')
  }

  const imageResponse = await fetch(data.url, { cache: 'no-store' })
  if (!imageResponse.ok) {
    throw new Error(`Background image request failed with status ${imageResponse.status}`)
  }

  const blob = await imageResponse.blob()
  const contentType = imageResponse.headers.get('content-type') ?? blob.type ?? 'image/*'

  return {
    sourceUrl: data.url,
    contentType,
    fetchedAt: Date.now(),
    blob: blob.type === contentType ? blob : new Blob([blob], { type: contentType }),
  }
}

const refreshBackground = async (): Promise<void> => {
  if (!consumeBackgroundRefreshAttempt()) {
    return
  }

  try {
    const record = await fetchBackgroundRecord()

    try {
      await saveBackgroundCache(record)
    } catch (error) {
      logger.warn('[AnimeBackground] failed to persist background cache', error)
    }

    const displayed = await displayCachedBackground(record)
    if (!displayed) {
      await clearBackgroundCache().catch(() => undefined)
      throw new Error('Downloaded background could not be displayed')
    }
  } catch (error) {
    logger.warn('[AnimeBackground] failed to refresh background image', error)
  }
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
        if (isBackgroundCacheExpired(cached)) {
          void refreshBackground()
        }

        return
      }
    }
  } catch (error) {
    logger.warn('[AnimeBackground] failed to load background cache', error)
  }

  await displayFallbackBackground()
  void refreshBackground()
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
