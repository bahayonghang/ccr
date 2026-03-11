<template>
  <button
    type="button"
    class="min-h-[44px] min-w-[44px] rounded-xl border border-border-default/70 bg-bg-elevated/70 text-text-secondary shadow-sm backdrop-blur-md transition-[color,background-color,border-color,box-shadow,transform] duration-200 hover:border-accent-primary/40 hover:bg-bg-surface hover:text-accent-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-base active:scale-95"
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
