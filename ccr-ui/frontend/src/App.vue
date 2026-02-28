<template>
  <div
    v-if="showGlobalBackground"
    class="premium-background"
  >
    <div class="premium-bg-orb orb-1" />
    <div class="premium-bg-orb orb-2" />
    <div class="premium-bg-orb orb-3" />
    <div class="premium-bg-pattern" />
  </div>
  <Titlebar />
  <div class="pt-9 h-screen overflow-hidden">
    <RouterView />
  </div>
  <ToastContainer />
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useThemeStore } from '@/store'
import Titlebar from '@/components/layout/Titlebar.vue'
import ToastContainer from '@/components/common/ToastContainer.vue'

const themeStore = useThemeStore()
const route = useRoute()

const showGlobalBackground = computed(() => {
  return !route.meta.hideGlobalBackground
})

onMounted(() => {
  themeStore.initializeTheme()
})
</script>
