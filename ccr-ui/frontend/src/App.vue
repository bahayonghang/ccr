<template>
  <RouterView v-slot="{ Component }">
    <keep-alive :include="['HomeView', 'ConfigsView', 'CommandsView']">
      <component :is="Component" />
    </keep-alive>
  </RouterView>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useThemeStore } from '@/store'

const themeStore = useThemeStore()

onMounted(() => {
  // Initialize theme from localStorage or default to light
  const savedTheme = localStorage.getItem('ccr-theme') || 'light'
  themeStore.setTheme(savedTheme)
})
</script>