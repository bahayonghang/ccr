<template>
  <div class="min-h-screen relative">
    <!-- 🎨 动态背景装饰 - 液态玻璃风格 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #ec4899 0%, #f59e0b 100%)',
          animationDelay: '1s'
        }"
      />
      <div
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #10b981 0%, #3b82f6 100%)',
          animationDelay: '2s'
        }"
      />
    </div>

    <div class="relative z-10 container mx-auto px-4 py-6">
      <!-- Navbar -->
      <div class="glass-card p-4 mb-6">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <div class="flex items-center space-x-2">
              <Zap class="w-6 h-6 text-accent-primary" />
              <h1 class="text-xl font-bold" :style="{ color: 'var(--text-primary)' }">CCR UI</h1>
            </div>
            <span class="text-sm" :style="{ color: 'var(--text-secondary)' }">Claude Code Configuration Switcher</span>
          </div>
          
          <div class="flex items-center space-x-2">
            <!-- Theme Toggle Button -->
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
          </div>
        </div>
      </div>

      <!-- Main Content -->
      <main class="mb-8">
        <slot />
      </main>

      <!-- Footer -->
      <footer class="text-center py-6">
        <p class="text-sm" :style="{ color: 'var(--text-muted)' }">
          现代化的配置管理解决方案 · 支持多种 AI CLI 工具
        </p>
        <p class="text-xs" :style="{ color: 'var(--text-muted)' }">
          Claude Code • Codex • Gemini • Qwen • IFLOW
        </p>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Zap, Moon, Sun } from 'lucide-vue-next'
import { useThemeStore } from '@/store/theme'

const themeStore = useThemeStore()

const currentTheme = computed(() => themeStore.currentTheme)

const toggleTheme = () => {
  themeStore.toggleTheme()
}
</script>