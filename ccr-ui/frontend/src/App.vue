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
  <RouterView />
  <ToastContainer />
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useThemeStore } from '@/store'
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
