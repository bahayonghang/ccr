<template>
  <div class="flex h-screen bg-bg-primary">
    <!-- 跳过导航链接 - Accessibility Enhancement -->
    <a
      href="#main-content"
      class="skip-to-content"
    >
      跳至主内容
    </a>
    <!-- Sidebar -->
    <div 
      class="bg-bg-secondary border-r border-border-color flex flex-col relative flex-shrink-0"
      :style="{ width: sidebarWidth + 'px', transition: isResizing ? 'none' : 'width 0.1s ease-out' }"
    >
      <!-- Resize Handle -->
      <div 
        class="absolute right-0 top-0 w-1 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors z-50 group"
        :class="{ 'bg-accent-primary': isResizing }"
        @mousedown.prevent="startResize"
      >
        <!-- Visual indicator on hover/active -->
        <div
          class="absolute right-0 top-1/2 -translate-y-1/2 w-1 h-8 bg-border-color rounded-full opacity-0 group-hover:opacity-100 transition-opacity"
          :class="{ 'opacity-100 bg-accent-primary': isResizing }"
        />
      </div>
      <!-- Logo and Title -->
      <div class="p-4 border-b border-border-color">
        <div class="flex items-center space-x-3">
          <div class="p-1.5 rounded-lg bg-bg-tertiary animate-pulse-subtle">
            <img
              src="@/assets/logo.png"
              alt="CCR Logo"
              class="w-8 h-8 object-contain"
            >
          </div>
          <div>
            <h1 class="text-xl font-bold text-text-primary">
              CCR UI
            </h1>
            <p class="text-xs text-text-secondary">
              AI CLI 配置管理
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
            class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
            active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
          >
            <Home class="w-5 h-5 mr-3" />
            <span class="font-medium">首页</span>
          </RouterLink>
        </div>

        <!-- 分隔线 -->
        <div class="px-3 mt-4">
          <div class="border-t border-border-color/50" />
        </div>

        <!-- 主要模块 -->
        <div class="px-3 mt-4">
          <h2 class="px-3 text-xs font-semibold text-text-muted uppercase tracking-wider mb-3 flex items-center">
            <span class="flex-1">主要模块</span>
            <span class="w-2 h-2 rounded-full bg-accent-primary/30 animate-pulse" />
          </h2>
          <div class="space-y-1">
            <RouterLink 
              to="/claude-code" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Code2 class="w-5 h-5 mr-3" />
              <span class="font-medium">Claude Code</span>
            </RouterLink>
            <RouterLink 
              to="/codex" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Settings class="w-5 h-5 mr-3" />
              <span class="font-medium">Codex</span>
            </RouterLink>
            <RouterLink 
              to="/gemini-cli" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Sparkles class="w-5 h-5 mr-3" />
              <span class="font-medium">Gemini CLI</span>
            </RouterLink>
            <RouterLink 
              to="/qwen" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Zap class="w-5 h-5 mr-3" />
              <span class="font-medium">Qwen</span>
            </RouterLink>
            <RouterLink 
              to="/iflow" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Activity class="w-5 h-5 mr-3" />
              <span class="font-medium">IFLOW</span>
            </RouterLink>
          </div>
        </div>

        <!-- 分隔线 -->
        <div class="px-3 mt-4">
          <div class="border-t border-border-color/50" />
        </div>

        <!-- 工具中心 -->
        <div class="px-3 mt-4">
          <h2 class="px-3 text-xs font-semibold text-text-muted uppercase tracking-wider mb-3 flex items-center">
            <span class="flex-1">工具中心</span>
            <span class="w-2 h-2 rounded-full bg-accent-warning/30 animate-pulse" />
          </h2>
          <div class="space-y-1">
            <RouterLink 
              to="/commands" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Terminal class="w-5 h-5 mr-3" />
              <span class="font-medium">命令执行</span>
            </RouterLink>
            <RouterLink 
              to="/converter" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <TrendingUp class="w-5 h-5 mr-3" />
              <span class="font-medium">配置转换</span>
            </RouterLink>
            <RouterLink 
              to="/sync" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Cloud class="w-5 h-5 mr-3" />
              <span class="font-medium">云同步</span>
            </RouterLink>
          </div>
        </div>
      </nav>

      <!-- Footer Actions -->
      <div class="p-4 border-t border-border-color space-y-3">
        <!-- Language Switcher -->
        <LanguageSwitcher />

        <!-- Exit Confirm Toggle (Tauri only) -->
        <div
          v-if="isTauri"
          class="flex items-center justify-between text-xs"
        >
          <span class="text-text-secondary">退出时确认</span>
          <button
            class="relative w-10 h-5 rounded-full transition-colors"
            :class="showExitConfirm ? 'bg-accent-primary' : 'bg-bg-tertiary'"
            @click="toggleExitConfirm"
          >
            <span
              class="absolute top-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform"
              :class="showExitConfirm ? 'translate-x-5' : 'translate-x-0.5'"
            />
          </button>
        </div>

        <!-- Version Info -->
        <div class="text-xs text-text-muted flex items-center justify-between">
          <span>CCR UI v3.19.1</span>
          <span class="animate-pulse-subtle">●</span>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div
      id="main-content"
      class="flex-1 flex flex-col overflow-auto"
      tabindex="-1"
    >
      <RouterView v-slot="{ Component }">
        <Transition
          name="page"
          mode="out-in"
        >
          <component :is="Component" />
        </Transition>
      </RouterView>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
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
} from 'lucide-vue-next'
import LanguageSwitcher from '@/components/LanguageSwitcher.vue'
import { isTauriEnvironment, getSkipExitConfirm, setSkipExitConfirm } from '@/api/tauri'

// Tauri 环境检测和退出确认设置
const isTauri = ref(false)
const showExitConfirm = ref(true)

const toggleExitConfirm = async () => {
  showExitConfirm.value = !showExitConfirm.value
  if (isTauri.value) {
    await setSkipExitConfirm(!showExitConfirm.value)
  }
}

// Sidebar Resizing Logic
const sidebarWidth = ref(260)
const isResizing = ref(false)
const minWidth = 180
const maxWidth = 800

const startResize = () => {
  isResizing.value = true
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none' // Prevent text selection while dragging
  
  window.addEventListener('mousemove', handleResize)
  window.addEventListener('mouseup', stopResize)
}

const handleResize = (e: MouseEvent) => {
  if (!isResizing.value) return
  
  // Calculate new width
  let newWidth = e.clientX
  
  // Apply constraints
  if (newWidth < minWidth) newWidth = minWidth
  if (newWidth > maxWidth) newWidth = maxWidth
  
  sidebarWidth.value = newWidth
}

const stopResize = () => {
  isResizing.value = false
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  
  // Save preference
  localStorage.setItem('ccr-sidebar-width', sidebarWidth.value.toString())
  
  window.removeEventListener('mousemove', handleResize)
  window.removeEventListener('mouseup', stopResize)
}

onMounted(async () => {
  // Restore sidebar width preference
  const savedWidth = localStorage.getItem('ccr-sidebar-width')
  if (savedWidth) {
    const width = parseInt(savedWidth)
    if (!isNaN(width) && width >= minWidth && width <= maxWidth) {
      sidebarWidth.value = width
    }
  }

  // Check Tauri environment and load settings
  isTauri.value = isTauriEnvironment()
  if (isTauri.value) {
    try {
      const skipConfirm = await getSkipExitConfirm()
      showExitConfirm.value = !skipConfirm
    } catch (e) {
      console.error('Failed to load exit confirm setting:', e)
    }
  }
})

onUnmounted(() => {
  window.removeEventListener('mousemove', handleResize)
  window.removeEventListener('mouseup', stopResize)
})
</script>