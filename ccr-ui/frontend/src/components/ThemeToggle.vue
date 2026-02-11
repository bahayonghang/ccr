<template>
  <button
    class="w-8 h-8 rounded-lg flex items-center justify-center transition-all bg-bg-surface hover:bg-bg-elevated border border-border-default hover:border-accent-primary/30 text-text-secondary hover:text-accent-primary"
    :title="`切换到${currentTheme === 'dark' ? '明亮' : '深色'}模式`"
    :aria-label="`切换到${currentTheme === 'dark' ? '明亮' : '深色'}模式`"
    @click.stop="toggleTheme"
  >
    <Moon
      v-if="currentTheme === 'dark'"
      class="w-4 h-4"
    />
    <Sun
      v-else
      class="w-4 h-4"
    />
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