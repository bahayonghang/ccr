<template>
  <button
    @click="toggleTheme"
    class="w-10 h-10 rounded-full transition-all hover:rotate-180 hover:scale-110 flex items-center justify-center"
    :style="{
      background: 'var(--bg-tertiary)',
      border: '1px solid var(--border-color)',
      color: 'var(--text-secondary)',
    }"
    :title="`切换到${currentTheme === 'dark' ? '明亮' : '深色'}模式`"
    :aria-label="`切换到${currentTheme === 'dark' ? '明亮' : '深色'}模式`"
  >
    <Moon v-if="currentTheme === 'dark'" class="w-5 h-5" />
    <Sun v-else class="w-5 h-5" />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useThemeStore } from '@/store'
import { Moon, Sun } from 'lucide-vue-next'

const themeStore = useThemeStore()

const currentTheme = computed(() => themeStore.currentTheme)

const toggleTheme = () => {
  themeStore.toggleTheme()
}
</script>