<template>
  <AnimeBackground v-if="showGlobalBackground" />
  <Titlebar />
  <div class="flex flex-col h-screen w-screen overflow-hidden">
    <RouterView class="flex-1 overflow-hidden" />
  </div>
  <ToastContainer />
  <GlobalConfirmDialog />
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import { useRoute } from 'vue-router'

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

const showGlobalBackground = computed(() => {
  return !route.meta.hideGlobalBackground
})
</script>
