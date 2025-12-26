<template>
  <div class="flex h-screen bg-bg-primary">
    <!-- Sidebar -->
    <div class="w-64 bg-bg-secondary border-r border-border-color flex flex-col">
      <!-- Logo and Title -->
      <div class="p-4 border-b border-border-color/50">
        <div class="flex items-center space-x-3">
          <div class="p-2 rounded-xl bg-gradient-to-br from-accent-primary/10 to-accent-secondary/10 animate-pulse-subtle shadow-sm">
            <Zap class="w-6 h-6 text-accent-primary animate-sidebar-item-enter" />
          </div>
          <div class="animate-sidebar-item-enter">
            <h1 class="text-xl font-black brand-gradient-text brand-gradient-text-hover tracking-tight">
              CCR UI
            </h1>
            <p class="text-xs text-text-secondary font-medium">
              {{ $t('home.subtitle') }}
            </p>
          </div>
        </div>
      </div>

      <!-- Navigation Menu -->
      <nav class="flex-1 overflow-y-auto py-4">
        <!-- 首页链接 -->
        <div class="px-3 space-y-1">
          <RouterLink 
            to="/" 
            class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm"
            active-class="nav-item-active"
          >
            <Home
              class="w-5 h-5 mr-3"
              style="color: #1890ff;"
            />
            <span class="font-medium">{{ $t('nav.home') }}</span>
          </RouterLink>
        </div>

        <!-- 分隔线 -->
        <div class="px-3 mt-4">
          <div class="border-t border-border-color/30" />
        </div>

        <!-- 主要模块 -->
        <div class="px-3 mt-4">
          <h2 class="px-3 text-xs font-semibold text-text-muted uppercase tracking-wider mb-3 flex items-center">
            <span class="flex-1">{{ $t('nav.mainModules') }}</span>
            <span class="w-2 h-2 rounded-full bg-accent-primary/30 animate-pulse" />
          </h2>
          <div class="space-y-1 nav-group">
            <RouterLink 
              to="/ccr-control" 
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <Terminal
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #10b981;"
              />
              <span class="font-medium">{{ $t('nav.ccrControl') }}</span>
            </RouterLink>
            <RouterLink 
              to="/claude-code" 
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <Code2
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #8b5cf6;"
              />
              <span class="font-medium">{{ $t('nav.claudeCode') }}</span>
            </RouterLink>
            <RouterLink 
              to="/codex" 
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <Settings
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #10b981;"
              />
              <span class="font-medium">{{ $t('nav.codex') }}</span>
            </RouterLink>
            <RouterLink 
              to="/gemini-cli" 
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <Sparkles
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #1a73e8;"
              />
              <span class="font-medium">{{ $t('nav.gemini') }}</span>
            </RouterLink>
            <RouterLink 
              to="/qwen" 
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <Zap
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #00b5e2;"
              />
              <span class="font-medium">{{ $t('nav.qwen') }}</span>
            </RouterLink>
            <RouterLink 
              to="/iflow" 
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <Activity
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #faad14;"
              />
              <span class="font-medium">{{ $t('nav.iflow') }}</span>
            </RouterLink>
          </div>
        </div>

        <!-- 分隔线 -->
        <div class="px-3 mt-4">
          <div class="border-t border-border-color/30" />
        </div>

        <!-- 工具中心 -->
        <div class="px-3 mt-4">
          <h2 class="px-3 text-xs font-semibold text-text-muted uppercase tracking-wider mb-3 flex items-center">
            <span class="flex-1">{{ $t('nav.toolsCenter') }}</span>
            <span class="w-2 h-2 rounded-full bg-accent-warning/30 animate-pulse" />
          </h2>
          <div class="space-y-1 nav-group">
            <RouterLink 
              to="/commands" 
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <Terminal
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #3b82f6;"
              />
              <span class="font-medium">{{ $t('nav.commands') }}</span>
            </RouterLink>
            <RouterLink 
              to="/converter" 
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <TrendingUp
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #13c2c2;"
              />
              <span class="font-medium">{{ $t('nav.converter') }}</span>
            </RouterLink>
            <RouterLink
              to="/sync"
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <Cloud
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #1890ff;"
              />
              <span class="font-medium">{{ $t('nav.sync') }}</span>
            </RouterLink>
            <RouterLink
              to="/usage"
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <Activity
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #10b981;"
              />
              <span class="font-medium">{{ $t('nav.usage') }}</span>
            </RouterLink>
            <RouterLink
              to="/checkin"
              class="nav-link flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:scale-[1.02] hover:shadow-sm group"
              active-class="nav-item-active"
            >
              <ClipboardCheck
                class="w-5 h-5 mr-3 group-hover:animate-nav-hover"
                style="color: #f59e0b;"
              />
              <span class="font-medium">{{ $t('nav.checkin') }}</span>
            </RouterLink>
          </div>
        </div>
      </nav>

      <!-- Version Info, Theme Toggle & Language Switcher -->
      <div class="p-4 border-t border-border-color/50 bg-bg-secondary/50 backdrop-blur-sm">
        <div class="flex items-center justify-between gap-3 animate-sidebar-item-enter">
          <LanguageSwitcher />
          <div class="flex items-center gap-2">
            <!-- Theme Toggle Button -->
            <button
              class="p-2 rounded-lg transition-all duration-300 hover:bg-bg-tertiary hover:scale-110"
              :title="currentTheme === 'dark' ? '切换到明亮模式' : '切换到深色模式'"
              @click="toggleTheme"
            >
              <Moon
                v-if="currentTheme === 'dark'"
                class="w-4 h-4 text-text-secondary"
              />
              <Sun
                v-else
                class="w-4 h-4 text-text-secondary"
              />
            </button>
            <div class="text-xs text-text-muted flex items-center gap-2 font-medium">
              <span>CCR UI v3.15.2</span>
              <span class="w-2 h-2 rounded-full bg-accent-success animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.4)]" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex flex-col overflow-auto">
      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  Home,
  Code2,
  Settings,
  Sparkles,
  Zap,
  Activity,
  Terminal,
  TrendingUp,
  Cloud,
  Moon,
  Sun,
  ClipboardCheck
} from 'lucide-vue-next'
import LanguageSwitcher from '@/components/LanguageSwitcher.vue'
import { useThemeStore } from '@/store'

const themeStore = useThemeStore()
const currentTheme = computed(() => themeStore.currentTheme)

const toggleTheme = () => {
  themeStore.toggleTheme()
}
</script>