<template>
  <MainLayout />
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useThemeStore } from '@/store/theme'
import MainLayout from '@/components/MainLayout.vue'

const themeStore = useThemeStore()

onMounted(() => {
  // Initialize theme from localStorage or default to light
  const savedTheme = localStorage.getItem('ccr-theme') || 'light'
  themeStore.setTheme(savedTheme)
})
</script>