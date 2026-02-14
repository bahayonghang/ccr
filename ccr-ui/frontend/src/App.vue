<template>
  <div class="premium-background">
    <div class="premium-bg-orb orb-1" />
    <div class="premium-bg-orb orb-2" />
    <div class="premium-bg-orb orb-3" />
    <div class="premium-bg-pattern" />
  </div>
  <RouterView v-slot="{ Component }">
    <keep-alive
      :include="cachedViews"
      :max="5"
    >
      <component :is="Component" />
    </keep-alive>
  </RouterView>
  <ToastContainer />
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useThemeStore } from '@/store'
import ToastContainer from '@/components/common/ToastContainer.vue'

// 扩展的 keep-alive 缓存列表
// 包含频繁访问的页面以提升性能
const cachedViews = [
  'HomeView',
  'ConfigsView', 
  'CommandsView',
  'ClaudeCodeView',
  'CodexView',
  'GeminiCliView',
  'QwenView',
  'IflowView',
  'CheckinView',

]

const themeStore = useThemeStore()

onMounted(() => {
  themeStore.initializeTheme()
})
</script>
