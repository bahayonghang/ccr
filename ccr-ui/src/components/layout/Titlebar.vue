<template>
  <div
    data-tauri-drag-region
    class="fixed top-0 left-0 right-0 h-9 z-50 flex items-center justify-between px-3 select-none text-white"
    :class="[
      isFocused ? 'bg-black/30' : 'bg-black/10',
      'backdrop-blur-md border-b border-white/10 transition-colors duration-200'
    ]"
  >
    <!-- Left: App Icon and Menu -->
    <div
      class="flex items-center space-x-1"
      data-tauri-drag-region
    >
      <!-- App Icon -->
      <div 
        class="w-5 h-5 rounded-md flex items-center justify-center mr-2 shadow-sm relative overflow-hidden group cursor-pointer"
        @click="showAboutDialog = true"
      >
        <img
          src="/icons/icon.png"
          class="w-full h-full object-cover transition-transform group-hover:scale-110"
          alt="CCR Desktop"
        >
      </div>

      <!-- Simple Menu Item as Example -->
      <div
        ref="menuRef"
        class="relative"
      >
        <button 
          class="titlebar-menu-btn"
          :class="{ 'bg-white/10 text-white': isMenuOpen }"
          @click="toggleMenu"
        >
          文件
        </button>
        
        <!-- Dropdown -->
        <div
          v-if="isMenuOpen"
          class="absolute top-full left-0 mt-1 w-48 bg-white/95 dark:bg-slate-900/95 backdrop-blur-xl border border-black/10 dark:border-white/10 rounded-lg shadow-2xl py-1 z-[100] overflow-hidden transform origin-top-left transition-[opacity,transform]"
        >
          <button
            class="w-full text-left px-3 py-1.5 text-xs text-slate-700 dark:text-white/80 hover:text-slate-900 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/10 transition-colors flex items-center"
            @click="openAbout"
          >
            <i class="i-carbon-information mr-2" /> 关于 CCR Neko
          </button>
          <div class="h-px bg-black/10 dark:bg-white/10 my-1" />
          <button
            class="w-full text-left px-3 py-1.5 text-xs text-red-500 dark:text-red-400 hover:text-red-600 dark:hover:text-red-300 hover:bg-red-50 dark:hover:bg-red-500/10 transition-colors flex items-center"
            @click="closeWindow"
          >
            <i class="i-carbon-close mr-2" /> 离开系统
          </button>
        </div>
      </div>
    </div>

    <!-- Center: Window Title -->
    <div
      data-tauri-drag-region
      class="absolute left-1/2 -translate-x-1/2 text-xs font-medium text-secondary-400 tracking-wider flex items-center space-x-2"
    >
      <span class="opacity-50">✦</span>
      <span>CCR DESKTOP</span>
      <span class="opacity-50">✦</span>
    </div>

    <!-- Right: Window Controls -->
    <div class="flex items-center space-x-0.5">
      <button 
        class="titlebar-control-btn group hover:bg-white/10" 
        title="最小化"
        @click="minimizeWindow"
      >
        <svg
          class="w-3.5 h-3.5 transition-colors text-secondary-400 group-hover:text-white"
          fill="currentColor"
          viewBox="0 0 16 16"
        >
          <rect
            x="3"
            y="8"
            width="10"
            height="1"
            rx="0.5"
          />
        </svg>
      </button>

      <button 
        class="titlebar-control-btn group hover:bg-white/10" 
        :title="isMaximized ? '还原' : '最大化'"
        @click="toggleMaximize"
      >
        <svg
          v-if="!isMaximized"
          class="w-3.5 h-3.5 transition-colors text-secondary-400 group-hover:text-white"
          fill="none"
          viewBox="0 0 16 16"
        >
          <rect
            x="3.5"
            y="3.5"
            width="9"
            height="9"
            rx="1"
            stroke="currentColor"
            stroke-width="1.2"
          />
        </svg>
        <svg
          v-else
          class="w-3.5 h-3.5 transition-colors text-secondary-400 group-hover:text-white"
          fill="none"
          viewBox="0 0 16 16"
        >
          <path
            d="M5.5 5.5v-2a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1h-2"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linecap="round"
          />
          <rect
            x="2.5"
            y="5.5"
            width="8"
            height="8"
            rx="1"
            stroke="currentColor"
            stroke-width="1.2"
          />
        </svg>
      </button>

      <button 
        class="titlebar-control-btn group hover:bg-red-500 hover:text-white control-close" 
        title="关闭"
        @click="closeWindow"
      >
        <svg
          class="w-3.5 h-3.5 transition-colors text-secondary-400 control-close-icon"
          fill="currentColor"
          viewBox="0 0 16 16"
        >
          <path d="M4.146 4.146a.5.5 0 0 0 0 .708L7.293 8l-3.147 3.146a.5.5 0 0 0 .708.708L8 8.707l3.146 3.147a.5.5 0 0 0 .708-.708L8.707 8l3.147-3.146a.5.5 0 0 0-.708-.708L8 7.293 4.854 4.146a.5.5 0 0 0-.708 0z" />
        </svg>
      </button>
    </div>

    <!-- About Dialog -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="opacity-0 scale-95"
        enter-to-class="opacity-100 scale-100"
        leave-active-class="transition duration-200 ease-in"
        leave-from-class="opacity-100 scale-100"
        leave-to-class="opacity-0 scale-95"
      >
        <div
          v-if="showAboutDialog"
          class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-slate-900/20 dark:bg-black/60 backdrop-blur-md"
          @click.self="showAboutDialog = false"
        >
          <div class="relative bg-white/95 dark:bg-surface-800/90 border border-slate-200 dark:border-white/10 rounded-2xl shadow-2xl w-full max-w-sm overflow-hidden overflow-y-auto">
            <!-- Glitch / Cyber effect top bar -->
            <div class="h-1 w-full bg-gradient-to-r from-primary-500 via-secondary-500 to-cyan-500" />
            
            <div class="p-6 flex flex-col items-center">
              <div class="w-24 h-24 rounded-2xl mb-4 relative overflow-hidden shadow-lg border border-slate-100 dark:border-white/5 ring-4 ring-slate-50 dark:ring-white/5">
                <img
                  src="/icons/icon.png"
                  alt="CCR Desktop Logo"
                  class="w-full h-full object-cover"
                >
                <div class="absolute inset-0 bg-gradient-to-tr from-primary-500/20 to-transparent mix-blend-overlay" />
              </div>
              
              <h2 class="text-2xl font-bold text-slate-800 dark:text-white tracking-tight mb-1">
                CCR Desktop
              </h2>
              <div class="flex items-center space-x-2 text-xs mb-4">
                <span class="px-2 py-0.5 rounded-full bg-primary-100 dark:bg-primary-500/20 text-primary-600 dark:text-primary-300 border border-primary-200 dark:border-primary-500/30">Neko Console</span>
                <span class="text-slate-500 dark:text-secondary-400">v4.1.4</span>
              </div>
              
              <p class="text-slate-600 dark:text-secondary-300 text-sm text-center mb-6 leading-relaxed">
                A sleek, powerful configuration manager for Claude Code, Codex, and Gemini. Designed with cyber-glass aesthetics and cat girl energy. (*/ω＼*)
              </p>
              
              <div class="w-full space-y-2 mb-6">
                <div class="flex justify-between items-center text-xs p-2 rounded-lg bg-slate-50 dark:glass-surface border border-slate-100 dark:border-white/5">
                  <span class="text-slate-500 dark:text-secondary-400">Owner</span>
                  <span class="text-slate-800 dark:text-white font-medium">李永航</span>
                </div>
                <div class="flex justify-between items-center text-xs p-2 rounded-lg bg-slate-50 dark:glass-surface border border-slate-100 dark:border-white/5">
                  <span class="text-slate-500 dark:text-secondary-400">Engine</span>
                  <span class="text-slate-800 dark:text-white font-medium">Tauri 2.0 & Vue 3</span>
                </div>
              </div>

              <button 
                class="w-full py-2 bg-slate-100 hover:bg-slate-200 dark:bg-white/10 dark:hover:bg-white/20 border border-slate-200 dark:border-white/10 rounded-xl text-sm font-medium text-slate-800 dark:text-white transition-[color,background-color,transform] transform hover:scale-[1.02] active:scale-95 flex items-center justify-center focus:outline-none"
                @click="showAboutDialog = false"
              >
                我知道了喵～
              </button>
            </div>
            
            <button
              class="absolute top-3 right-3 p-1.5 rounded-full text-slate-400 hover:text-slate-800 hover:bg-slate-100 dark:text-secondary-400 dark:hover:text-white dark:hover:bg-white/10 transition-colors"
              @click="showAboutDialog = false"
            >
              <svg
                class="w-4 h-4"
                fill="currentColor"
                viewBox="0 0 16 16"
              >
                <path d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z" />
              </svg>
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { logger } from '@/utils/logger'
import { getCurrentWindowSafe } from '@/utils/tauriWindow'

const isMaximized = ref(false)
const isFocused = ref(true)
const isMenuOpen = ref(false)
const showAboutDialog = ref(false)
const menuRef = ref<HTMLElement | null>(null)

// Actions
const minimizeWindow = async () => {
  const win = await getCurrentWindowSafe()
  if (!win) return
  await win.minimize()
}

const toggleMaximize = async () => {
  try {
    const win = await getCurrentWindowSafe()
    if (!win) {
      isMaximized.value = !isMaximized.value
      return
    }
    if (await win.isMaximized()) {
      await win.unmaximize()
      isMaximized.value = false
    } else {
      await win.maximize()
      isMaximized.value = true
    }
  } catch (e) {
    // Basic web fallback toggle for local dev
    isMaximized.value = !isMaximized.value
  }
}

const closeWindow = async () => {
  const win = await getCurrentWindowSafe()
  if (!win) return
  await win.close()
}

const toggleMenu = () => {
  isMenuOpen.value = !isMenuOpen.value
}

const openAbout = () => {
  isMenuOpen.value = false
  showAboutDialog.value = true
}

// Global click to close menu
const handleClickOutside = (e: MouseEvent) => {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    isMenuOpen.value = false
  }
}

// Listen to window state
let unlistenMaximized: (() => void) | undefined
let unlistenFocused: (() => void) | undefined

onMounted(async () => {
  document.addEventListener('click', handleClickOutside)
  
  try {
    const win = await getCurrentWindowSafe()
    if (!win) {
      return
    }
    isMaximized.value = await win.isMaximized()
    
    unlistenMaximized = await win.onResized(async () => {
      isMaximized.value = await win.isMaximized()
    })
    
    unlistenFocused = await win.onFocusChanged(({ payload: focused }) => {
      isFocused.value = focused
    })
  } catch (e) {
    logger.debug('[Titlebar] skip tauri listeners in browser runtime', e)
  }
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  if (unlistenMaximized) unlistenMaximized()
  if (unlistenFocused) unlistenFocused()
})
</script>

<style scoped>
.titlebar-control-btn {
  width: 46px;
  height: 2.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 150ms, background-color 150ms;
  cursor: default;
  outline: none;
}

.titlebar-menu-btn {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
  color: var(--color-secondary-400, #9ca3af);
  border-radius: 0.375rem;
  transition: color 150ms, background-color 150ms;
  outline: none;
}

.titlebar-menu-btn:hover {
  color: white;
  background-color: rgb(255 255 255 / 10%);
}

/* Ensure close button icon turns white on hover correctly */
.control-close:hover .control-close-icon {
  color: white;
}
</style>
