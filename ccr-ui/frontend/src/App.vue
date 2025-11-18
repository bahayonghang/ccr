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
  themeStore.initializeTheme()
})
</script>
