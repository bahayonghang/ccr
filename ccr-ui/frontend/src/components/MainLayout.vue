<template>
  <div class="flex h-screen bg-bg-base text-text-primary overflow-hidden font-sans selection:bg-accent-primary/30">
    <!-- Skip Link -->
    <a
      href="#main-content"
      class="skip-to-content z-50"
    >
      {{ $t('common.skipToContent') || 'Skip to content' }}
    </a>

    <!-- Sidebar (Glassmorphism + Resize) -->
    <div
      class="flex flex-col relative flex-shrink-0 z-40 transition-all duration-300 ease-out will-change-[width]"
      :class="[
        'bg-bg-elevated/80 backdrop-blur-xl border-r border-border-subtle shadow-xl',
        isResizing ? 'select-none' : ''
      ]"
      :style="{ width: sidebarWidth + 'px' }"
    >
      <!-- Resize Handle -->
      <div
        class="absolute -right-1 top-0 w-2 h-full cursor-col-resize z-50 group outline-none"
        @mousedown.prevent="startResize"
      >
        <div class="absolute inset-y-0 right-1/2 w-[1px] bg-border-subtle group-hover:bg-accent-primary/50 transition-colors delay-75" />
      </div>

      <!-- Logo Area -->
      <div class="h-16 flex items-center px-4 border-b border-white/5">
        <div class="flex items-center gap-3">
          <div class="relative w-8 h-8 flex items-center justify-center rounded-lg bg-gradient-to-br from-accent-primary to-accent-secondary shadow-glow-primary">
            <Zap class="w-5 h-5 text-white" />
          </div>
          <div>
            <h1 class="text-lg font-bold font-display tracking-tight leading-none bg-clip-text text-transparent bg-gradient-to-r from-text-primary to-text-secondary">
              CCR <span class="text-accent-primary">UI</span>
            </h1>
            <p class="text-[10px] uppercase tracking-widest text-text-muted font-bold mt-0.5 opacity-60">
              Terminal Node
            </p>
          </div>
        </div>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 overflow-y-auto p-3 space-y-6 scrollbar-hide">
        <!-- Section: Main -->
        <div class="space-y-1">
          <RouterLink
            to="/"
            class="nav-item"
          >
            <Home class="w-4 h-4" />
            <span>{{ $t('nav.home') }}</span>
          </RouterLink>
        </div>

        <!-- Section: Modules -->
        <div>
          <div class="px-3 mb-2 text-[10px] font-bold uppercase tracking-wider text-text-muted/60 flex items-center gap-2">
            {{ $t('nav.mainModules') }}
            <div class="h-px flex-1 bg-border-subtle" />
          </div>
          <div class="space-y-0.5">
            <RouterLink
              to="/claude-code"
              class="nav-item group"
            >
              <Code2 class="w-4 h-4 text-purple-400 group-hover:text-purple-300 transition-colors" />
              <span>{{ $t('nav.claudeCode') }}</span>
            </RouterLink>
            <RouterLink
              to="/codex"
              class="nav-item group"
            >
              <Settings class="w-4 h-4 text-emerald-400 group-hover:text-emerald-300 transition-colors" />
              <span>{{ $t('nav.codex') }}</span>
            </RouterLink>
            <RouterLink
              to="/gemini-cli"
              class="nav-item group"
            >
              <Sparkles class="w-4 h-4 text-sky-400 group-hover:text-sky-300 transition-colors" />
              <span>{{ $t('nav.gemini') }}</span>
            </RouterLink>
            <RouterLink
              to="/qwen"
              class="nav-item group"
            >
              <Zap class="w-4 h-4 text-cyan-400 group-hover:text-cyan-300 transition-colors" />
              <span>{{ $t('nav.qwen') }}</span>
            </RouterLink>
            <RouterLink
              to="/iflow"
              class="nav-item group"
            >
              <Activity class="w-4 h-4 text-amber-400 group-hover:text-amber-300 transition-colors" />
              <span>{{ $t('nav.iflow') }}</span>
            </RouterLink>
            <RouterLink
              to="/droid"
              class="nav-item group"
            >
              <Bot class="w-4 h-4 text-pink-400 group-hover:text-pink-300 transition-colors" />
              <span>{{ $t('nav.droid') }}</span>
            </RouterLink>
          </div>
        </div>

        <!-- Section: Tools -->
        <div>
          <div class="px-3 mb-2 text-[10px] font-bold uppercase tracking-wider text-text-muted/60 flex items-center gap-2">
            {{ $t('nav.toolsCenter') }}
            <div class="h-px flex-1 bg-border-subtle" />
          </div>
          <div class="space-y-0.5">
            <RouterLink
              to="/ccr-control"
              class="nav-item"
            >
              <Terminal class="w-4 h-4" />
              <span>{{ $t('nav.ccrControl') }}</span>
            </RouterLink>
            <RouterLink
              to="/commands"
              class="nav-item"
            >
              <Terminal class="w-4 h-4" />
              <span>{{ $t('nav.commands') }}</span>
            </RouterLink>
            <RouterLink
              to="/checkin"
              class="nav-item"
            >
              <ClipboardList class="w-4 h-4" />
              <span>{{ $t('nav.checkin') }}</span>
            </RouterLink>
            <RouterLink
              to="/sync"
              class="nav-item"
            >
              <Cloud class="w-4 h-4" />
              <span>{{ $t('nav.sync') }}</span>
            </RouterLink>
            <RouterLink
              to="/usage"
              class="nav-item"
            >
              <Activity class="w-4 h-4" />
              <span>{{ $t('nav.usage') }}</span>
            </RouterLink>
          </div>
        </div>
      </nav>

      <!-- Footer: User Profile -->
      <div class="p-4 border-t border-white/5 bg-black/20">
        <div class="relative group rounded-xl bg-white/5 border border-white/5 p-3 transition-all hover:bg-white/10 hover:border-white/10 hover:shadow-lg">
          <!-- Top Row: Avatar & Toggle -->
          <div class="flex items-start justify-between mb-3">
            <!-- Avatar (Hexagon Style for Neo-Terminal) -->
            <div class="relative">
              <div class="w-10 h-10 flex items-center justify-center rounded-lg bg-gradient-to-br from-indigo-500 to-violet-600 text-white font-bold font-mono text-xs shadow-glow-primary">
                ENG
              </div>
              <div class="absolute -bottom-1 -right-1 w-3 h-3 bg-bg-base rounded-full flex items-center justify-center">
                <div class="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]" />
              </div>
            </div>

            <!-- Theme Toggle (Fixed Position & Clickable) -->
            <ThemeToggle class="relative z-20 hover:text-accent-primary transition-colors" />
          </div>

          <!-- Bottom Row: User Info -->
          <div class="space-y-0.5">
            <h3 class="text-sm font-bold text-text-primary tracking-wide">
              ENGINEER
            </h3>
            <div class="flex items-center justify-between">
              <p class="text-[10px] text-text-muted font-mono uppercase tracking-wider">
                Session: <span class="text-emerald-500">Active</span>
              </p>
              <span class="text-[10px] font-mono text-text-muted/40">CCR UI v3.20.3</span>
            </div>
          </div>

          <!-- Decorative Lines -->
          <div class="absolute bottom-3 right-3 w-8 h-[2px] bg-white/10 rounded-full" />
        </div>
      </div>
    </div>

    <!-- Main Content Area -->
    <main
      id="main-content"
      class="flex-1 relative overflow-hidden flex flex-col"
    >
      <!-- Top Bar (Optional, if needed for breadcrumbs or global search) -->
      <div class="h-14 flex items-center px-6 border-b border-border-subtle bg-bg-base/50 backdrop-blur-sm z-30 sticky top-0 justify-between">
        <!-- Breadcrumbs Placeholder -->
        <div class="flex items-center text-sm text-text-muted">
          <span class="opacity-50">App</span>
          <span class="mx-2 opacity-30">/</span>
          <span class="text-text-primary font-medium">Dashboard</span>
        </div>
         
        <div class="flex items-center gap-4">
          <LanguageSwitcher />
          <div
            v-if="isTauri"
            class="h-4 w-px bg-border-subtle mx-2"
          />
          <!-- Exit Toggle -->
          <button
            v-if="isTauri"
            class="flex items-center gap-2 text-xs font-medium text-text-muted hover:text-text-primary transition-colors"
            :class="{ 'text-accent-primary': showExitConfirm }"
            @click="toggleExitConfirm"
          >
            <div class="w-3 h-3 rounded-full border border-current flex items-center justify-center">
              <div
                class="w-1.5 h-1.5 rounded-full bg-current transition-transform duration-300"
                :class="showExitConfirm ? 'scale-100' : 'scale-0'"
              />
            </div>
            Exit Confirm
          </button>
        </div>
      </div>

      <!-- Scrollable Content -->
      <div class="flex-1 overflow-y-auto scroll-smooth p-6">
        <BackendStatusBanner class="mb-6" />
        <RouterView v-slot="{ Component }">
          <transition 
            name="fade-slide" 
            mode="out-in"
            appear
          >
            <component :is="Component" />
          </transition>
        </RouterView>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { 
  Home, Code2, Settings, Sparkles, Zap, Activity, 
  Terminal, Cloud, Bot, ClipboardList 
} from 'lucide-vue-next'
import BackendStatusBanner from '@/components/BackendStatusBanner.vue'
import LanguageSwitcher from '@/components/LanguageSwitcher.vue'
import ThemeToggle from '@/components/ThemeToggle.vue'
import { isTauriEnvironment, getSkipExitConfirm, setSkipExitConfirm } from '@/api/tauri'

// Sidebar State
const sidebarWidth = ref(240)
const isResizing = ref(false)
const minWidth = 200
const maxWidth = 480

// Tauri State
const isTauri = ref(false)
const showExitConfirm = ref(true)

const toggleExitConfirm = async () => {
  showExitConfirm.value = !showExitConfirm.value
  if (isTauri.value) {
    await setSkipExitConfirm(!showExitConfirm.value)
  }
}

// Resizing Logic
const startResize = () => {
  isResizing.value = true
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  window.addEventListener('mousemove', handleResize)
  window.addEventListener('mouseup', stopResize)
}

const handleResize = (e: MouseEvent) => {
  if (!isResizing.value) return
  let newWidth = e.clientX
  if (newWidth < minWidth) newWidth = minWidth
  if (newWidth > maxWidth) newWidth = maxWidth
  sidebarWidth.value = newWidth
}

const stopResize = () => {
  isResizing.value = false
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  localStorage.setItem('ccr-sidebar-width', sidebarWidth.value.toString())
  window.removeEventListener('mousemove', handleResize)
  window.removeEventListener('mouseup', stopResize)
}

onMounted(async () => {
  const savedWidth = localStorage.getItem('ccr-sidebar-width')
  if (savedWidth) sidebarWidth.value = Number(savedWidth) || 240
  
  isTauri.value = isTauriEnvironment()
  if (isTauri.value) {
    try {
      const skipConfirm = await getSkipExitConfirm()
      showExitConfirm.value = !skipConfirm
    } catch (e) { console.error(e) }
  }
})

onUnmounted(() => {
  window.removeEventListener('mousemove', handleResize)
  window.removeEventListener('mouseup', stopResize)
})
</script>

<style scoped>
/* Nav Item Styles */
.nav-item {
  @apply flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-text-secondary 
         transition-all duration-200 relative overflow-hidden;
}

.nav-item:hover {
  @apply bg-bg-surface text-text-primary;
}

.nav-item.router-link-active {
  @apply bg-accent-primary/10 text-accent-primary shadow-sm;
}

/* Active indicator strip */
.nav-item.router-link-active::before {
  content: '';

  @apply absolute left-0 top-1/2 -translate-y-1/2 h-4 w-1 bg-accent-primary rounded-r-full;
}

/* Page Transition */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-5px);
}
</style>
