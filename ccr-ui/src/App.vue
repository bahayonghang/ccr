<template>
  <AnimeBackground v-if="showGlobalBackground" />
  <Titlebar v-if="showCustomTitlebar" />
  <div
    class="flex h-screen w-screen flex-col overflow-hidden bg-bg-base"
    :style="appShellStyle"
  >
    <RouterView class="flex-1 overflow-hidden" />
  </div>
  <ToastContainer />
  <GlobalConfirmDialog />
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getCurrentWindowSafe } from '@/utils/tauriWindow'
import { getWindowChromeTopInset, shouldUseCustomTitlebar } from '@/utils/windowChrome'

const Titlebar = defineAsyncComponent({
  loader: () => import('@/components/layout/Titlebar.vue'),
  suspensible: false,
})

const ToastContainer = defineAsyncComponent({
  loader: () => import('@/components/common/ToastContainer.vue'),
  suspensible: false,
})

const AnimeBackground = defineAsyncComponent({
  loader: () => import('@/components/common/AnimeBackground.vue'),
  suspensible: false,
})

const GlobalConfirmDialog = defineAsyncComponent({
  loader: () => import('@/components/common/GlobalConfirmDialog.vue'),
  suspensible: false,
})

const route = useRoute()
const router = useRouter()
const showCustomTitlebar = shouldUseCustomTitlebar()
const windowChromeTopInset = getWindowChromeTopInset()
let stopShellNavigate: (() => void) | null = null

const showGlobalBackground = computed(() => {
  return !route.meta.hideGlobalBackground
})

const appShellStyle = computed(() => {
  return windowChromeTopInset > 0
    ? { paddingTop: `${windowChromeTopInset}px` }
    : undefined
})

onMounted(async () => {
  const win = await getCurrentWindowSafe()
  if (!win) {
    return
  }

  stopShellNavigate = await win.listen<string>('shell:navigate', async (event) => {
    if (!event.payload || router.currentRoute.value.fullPath === event.payload) {
      return
    }

    await router.push(event.payload)
  })
})

onUnmounted(() => {
  stopShellNavigate?.()
  stopShellNavigate = null
})
</script>
