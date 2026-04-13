<template>
  <div class="min-h-screen relative">
    <AnimatedBackground variant="minimal" />

    <div class="relative z-10 container mx-auto px-4 py-6">
      <!-- Navbar -->
      <div class="glass-card p-4 mb-6">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <div class="flex items-center space-x-2">
              <SIcon
                name="Zap"
                size="w-6 h-6"
                class="text-accent-primary"
              />
              <h1
                class="text-xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                CCR UI
              </h1>
            </div>
            <span
              class="text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >Claude Code Configuration Switcher</span>
          </div>
          
          <div class="flex items-center space-x-2">
            <!-- Theme Toggle Button -->
            <button
              class="w-10 h-10 rounded-full transition-[color,background-color,border-color,transform] hover:rotate-180 hover:scale-110 flex items-center justify-center"
              :style="{
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-secondary)',
              }"
              :title="`切换到${currentTheme === 'dark' ? '明亮' : '深色'}模式`"
              :aria-label="`切换到${currentTheme === 'dark' ? '明亮' : '深色'}模式`"
              @click="toggleTheme"
            >
              <SIcon
                v-if="currentTheme === 'dark'"
                name="Moon"
                size="w-5 h-5"
              />
              <SIcon
                v-else
                name="Sun"
                size="w-5 h-5"
              />
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
        <p
          class="text-sm"
          :style="{ color: 'var(--text-muted)' }"
        >
          现代化的配置管理解决方案 · 支持多种 AI CLI 工具
        </p>
        <p
          class="text-xs"
          :style="{ color: 'var(--text-muted)' }"
        >
          Claude Code • Codex • Gemini
        </p>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import { computed } from 'vue'
import { useThemeStore } from '@/stores/theme'

const themeStore = useThemeStore()

const currentTheme = computed(() => themeStore.currentTheme)

const toggleTheme = () => {
  themeStore.toggleTheme()
}
</script>
