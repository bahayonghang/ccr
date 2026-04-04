<template>
  <AnimeBackground v-if="showGlobalBackground" />
  <Titlebar v-if="showCustomTitlebar" />
  <div
    class="flex h-screen w-screen flex-col overflow-hidden bg-bg-base"
    :class="{ 'pt-9': showCustomTitlebar }"
  >
    <RouterView class="flex-1 overflow-hidden" />
  </div>
  <ToastContainer />
  <GlobalConfirmDialog />
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import { useRoute } from 'vue-router'
import { shouldUseCustomTitlebar } from '@/utils/windowChrome'

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
const showCustomTitlebar = shouldUseCustomTitlebar()

const showGlobalBackground = computed(() => {
  return !route.meta.hideGlobalBackground
})
</script>
