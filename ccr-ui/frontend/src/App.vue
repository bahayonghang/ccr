<template>
  <RouterView v-slot="{ Component }">
    <keep-alive :include="['HomeView', 'ConfigsView', 'CommandsView']">
      <component :is="Component" />
    </keep-alive>
  </RouterView>
  <ToastContainer />
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useThemeStore } from '@/store'
import ToastContainer from '@/components/common/ToastContainer.vue'

const themeStore = useThemeStore()

onMounted(() => {
  themeStore.initializeTheme()
})
</script>

