<template>
  <div class="flex h-screen text-text-primary overflow-hidden font-sans selection:bg-accent-primary/30">
    <!-- Background Image Layer -->
    <BackgroundImage />

    <!-- Skip Link -->
    <a
      href="#main-content"
      class="skip-to-content z-50"
    >
      {{ $t('common.skipToContent') || 'Skip to content' }}
    </a>

    <!-- Sidebar (Glassmorphism + Resize) -->
    <div
      class="flex flex-col relative flex-shrink-0 z-40 transition-all duration-300 ease-out will-change-[width] sidebar-glass"
      :class="[isResizing ? 'select-none' : '']"
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
      <div class="h-16 flex items-center px-4 border-b border-pink-200/20 dark:border-pink-300/10">
        <div class="flex items-center gap-3">
          <div class="relative w-8 h-8 flex items-center justify-center rounded-lg bg-gradient-to-br from-pink-400 to-violet-400 shadow-lg shadow-pink-400/30">
            <Cat class="w-5 h-5 text-white" />
          </div>
          <div>
            <h1 class="text-lg font-bold font-display tracking-tight leading-none text-slate-800 dark:text-white">
              CCR <span class="text-accent-primary">UI</span>
            </h1>
            <p class="text-[10px] uppercase tracking-widest text-pink-400 dark:text-pink-300 font-bold mt-0.5">
              Neko Console
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

        <!-- Section: Skills Hub -->
        <div>
          <div class="px-3 mb-2 text-[10px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500 flex items-center gap-2">
            {{ $t('nav.skillsHub') }}
            <div class="h-px flex-1 bg-black/10 dark:bg-white/10" />
          </div>
          <div class="space-y-0.5">
            <RouterLink
              to="/skills"
              class="nav-item group"
            >
              <Package class="w-4 h-4 text-fuchsia-400 group-hover:text-fuchsia-300 transition-colors" />
              <span>{{ $t('nav.skills') }}</span>
            </RouterLink>
            <RouterLink
              to="/skills/add"
              class="nav-item group"
            >
              <PlusCircle class="w-4 h-4 text-fuchsia-400 group-hover:text-fuchsia-300 transition-colors" />
              <span>{{ $t('nav.addSkill') }}</span>
            </RouterLink>
          </div>
        </div>

        <!-- Section: Modules -->
        <div>
          <div class="px-3 mb-2 text-[10px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500 flex items-center gap-2">
            {{ $t('nav.mainModules') }}
            <div class="h-px flex-1 bg-black/10 dark:bg-white/10" />
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
          <div class="px-3 mb-2 text-[10px] font-bold uppercase tracking-wider text-slate-400 dark:text-slate-500 flex items-center gap-2">
            {{ $t('nav.toolsCenter') }}
            <div class="h-px flex-1 bg-black/10 dark:bg-white/10" />
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

      <!-- Footer: User Profile - Neko Kawaii Style -->
      <div class="p-3 border-t border-pink-200/10 dark:border-pink-300/10">
        <div class="user-card group relative rounded-2xl overflow-hidden transition-all duration-300">
          <!-- Animated gradient background -->
          <div class="absolute inset-0 bg-gradient-to-br from-pink-500/15 via-fuchsia-500/12 to-violet-500/15 opacity-80" />
          <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,rgba(244,114,182,0.12),transparent_50%)]" />

          <!-- Neko dot pattern -->
          <div class="absolute inset-0 opacity-10 bg-[radial-gradient(circle,rgba(255,255,255,0.15)_1px,transparent_1px)] bg-[size:10px_10px]" />

          <!-- Glow effect on hover -->
          <div class="absolute -inset-1 bg-gradient-to-r from-pink-500/0 via-fuchsia-400/20 to-violet-500/0 blur-xl opacity-0 group-hover:opacity-100 transition-opacity duration-500" />

          <!-- Inner content -->
          <div class="relative p-3.5 backdrop-blur-sm z-10">
            <!-- Top Row: Catgirl Avatar & NYA Badge & Toggle -->
            <div class="flex items-start justify-between mb-3">
              <!-- Catgirl Avatar with neko glow frame -->
              <div class="relative">
                <div class="w-14 h-14 rounded-xl overflow-hidden bg-gradient-to-br from-pink-400/20 via-fuchsia-400/20 to-violet-400/20 shadow-lg shadow-pink-500/40 ring-2 ring-pink-300/30 group-hover:ring-pink-400/50 group-hover:shadow-pink-500/60 transition-all duration-300">
                  <img
                    src="/catgirl_avatar.png"
                    alt="Catgirl"
                    class="w-full h-full object-cover object-top scale-[1.3] group-hover:scale-[1.4] transition-transform duration-500 drop-shadow-[0_0_8px_rgba(244,114,182,0.5)]"
                  >
                </div>
                <!-- Pulsing status ring -->
                <div class="absolute -bottom-0.5 -right-0.5">
                  <div class="w-4 h-4 rounded-full bg-slate-900 flex items-center justify-center ring-2 ring-slate-800">
                    <div class="w-2.5 h-2.5 rounded-full bg-emerald-400 shadow-[0_0_12px_rgba(52,211,153,1)] animate-pulse" />
                  </div>
                </div>
              </div>

              <!-- NYA Badge + Theme Toggle -->
              <div class="flex items-center gap-2">
                <span class="px-2 py-1 rounded-lg text-[10px] font-bold font-mono tracking-wider bg-gradient-to-br from-pink-400 via-fuchsia-400 to-violet-400 text-white shadow-md shadow-pink-500/30 drop-shadow-[0_0_6px_rgba(244,114,182,0.6)]">
                  NYA
                </span>
                <ThemeToggle class="relative z-20 p-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 hover:border-pink-400/30 transition-all duration-200" />
              </div>
            </div>

            <!-- User Info -->
            <div class="space-y-2">
              <div class="flex items-center gap-2.5">
                <h3 class="text-sm font-bold text-slate-800 dark:text-white tracking-wide drop-shadow-sm">
                  {{ $t('nav.user.role') }}
                </h3>
                <span class="px-2 py-0.5 rounded-md text-[10px] font-bold uppercase tracking-wider bg-gradient-to-r from-pink-500/30 to-fuchsia-500/30 text-pink-300 border border-pink-400/30 shadow-sm shadow-pink-500/20">
                  Pro
                </span>
              </div>

              <div class="flex items-center justify-between">
                <p class="text-[11px] font-mono uppercase tracking-wider flex items-center gap-2">
                  <span class="text-slate-400">Session:</span>
                  <span class="flex items-center gap-1.5 text-emerald-400 font-semibold">
                    <span class="relative flex h-2 w-2">
                      <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                      <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.8)]" />
                    </span>
                    Active
                  </span>
                </p>
                <span class="text-[10px] font-mono text-slate-500 bg-white/50 dark:bg-slate-800/50 px-2 py-0.5 rounded-md border border-slate-200 dark:border-slate-700/50">
                  CCR UI v4.0.8
                </span>
              </div>
            </div>

            <!-- Decorative corner accents -->
            <div class="absolute top-2 right-14 w-8 h-[1px] bg-gradient-to-r from-pink-400/50 to-transparent" />
            <div class="absolute bottom-2 left-2 w-6 h-[1px] bg-gradient-to-r from-transparent to-fuchsia-500/50" />
          </div>

          <!-- Bottom accent line -->
          <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-gradient-to-r from-transparent via-pink-400/50 to-transparent" />
        </div>
      </div>
    </div>

    <!-- Main Content Area -->
    <main
      id="main-content"
      class="flex-1 relative overflow-hidden flex flex-col"
    >
      <!-- Top Bar (Optional, if needed for breadcrumbs or global search) -->
      <div class="h-14 flex items-center px-6 border-b border-black/5 dark:border-white/10 bg-white/60 dark:bg-slate-900/40 backdrop-blur-xl z-30 sticky top-0 justify-between">
        <!-- Breadcrumbs -->
        <div class="flex items-center text-sm text-text-muted">
          <span class="opacity-50">{{ $t('nav.mainModules') }}</span>
          <span class="mx-2 opacity-30">/</span>
          <span class="text-text-primary font-medium">{{ currentPageTitle }}</span>
        </div>

        <div class="flex items-center gap-4">
          <LanguageSwitcher />
          <div
            v-if="isTauri"
            class="h-4 w-px bg-black/10 dark:bg-white/10 mx-2"
          />
          <!-- Exit Toggle -->
          <button
            v-if="isTauri"
            class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200 border"
            :class="[
              showExitConfirm 
                ? 'bg-accent-primary/10 border-accent-primary/30 text-accent-primary' 
                : 'bg-bg-surface border-border-default text-text-secondary hover:text-text-primary hover:border-accent-primary/30 hover:bg-bg-elevated'
            ]"
            :title="showExitConfirm ? $t('common.yes') : $t('common.no')"
            @click="toggleExitConfirm"
          >
            <div class="w-3 h-3 rounded-full border border-current flex items-center justify-center">
              <div
                class="w-1.5 h-1.5 rounded-full bg-current transition-transform duration-300"
                :class="showExitConfirm ? 'scale-100' : 'scale-0'"
              />
            </div>
            {{ $t('common.exitConfirm') }}
          </button>
        </div>
      </div>

      <!-- Scrollable Content with glass effect -->
      <div class="flex-1 overflow-y-auto scroll-smooth p-6 bg-white/30 dark:bg-slate-900/30 backdrop-blur-sm">
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
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  Home, Code2, Settings, Sparkles, Zap, Activity,
  Terminal, Cloud, Bot, ClipboardList, Cat, Package, PlusCircle
} from 'lucide-vue-next'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import BackendStatusBanner from '@/components/BackendStatusBanner.vue'
import LanguageSwitcher from '@/components/LanguageSwitcher.vue'
import ThemeToggle from '@/components/ThemeToggle.vue'
import BackgroundImage from '@/components/common/BackgroundImage.vue'
import { isTauriEnvironment, getSkipExitConfirm, setSkipExitConfirm } from '@/api/tauri'

const route = useRoute()
const { t } = useI18n()

// 路由名 → i18n 键映射
const routeTitleMap: Record<string, string> = {
  home: 'nav.home',
  skills: 'nav.skills',
  'add-skill': 'nav.addSkill',
  'claude-code': 'nav.claudeCode',
  codex: 'nav.codex',
  'gemini-cli': 'nav.gemini',
  qwen: 'nav.qwen',
  iflow: 'nav.iflow',
  droid: 'nav.droid',
  'ccr-control': 'nav.ccrControl',
  commands: 'nav.commands',
  checkin: 'nav.checkin',
  sync: 'nav.sync',
  usage: 'nav.usage',
}

const currentPageTitle = computed(() => {
  const name = route.name as string
  const key = routeTitleMap[name]
  return key ? t(key) : name || 'Home'
})

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
/* Sidebar Glass Effect - Dark Mode (Neko Purple Night) */
:root[data-theme="dark"] .sidebar-glass,
.dark .sidebar-glass {
  background: linear-gradient(
    180deg,
    rgb(26 10 32 / 88%) 0%,
    rgb(38 18 50 / 90%) 50%,
    rgb(26 10 32 / 95%) 100%
  );
  backdrop-filter: blur(20px) saturate(180%);
  border-right: 1px solid rgb(249 168 212 / 8%);
  box-shadow:
    4px 0 24px rgb(0 0 0 / 30%),
    inset -1px 0 0 rgb(249 168 212 / 5%);
}

/* Sidebar Glass Effect - Light Mode (Neko Pink Glass) */
.sidebar-glass {
  background: linear-gradient(
    180deg,
    rgb(255 240 245 / 40%) 0%,
    rgb(255 245 247 / 50%) 50%,
    rgb(255 255 255 / 55%) 100%
  );
  backdrop-filter: blur(12px) saturate(150%);
  border-right: 1px solid rgb(244 114 182 / 10%);
  box-shadow:
    4px 0 16px rgb(244 114 182 / 5%),
    inset -1px 0 0 rgb(255 255 255 / 50%);
}

/* Nav Item Styles - Light Mode (Neko Pink) */
.nav-item {
  @apply flex items-center gap-3 px-3 py-2 rounded-xl text-sm font-medium text-slate-600
         transition-all duration-200 relative overflow-hidden;
}

.nav-item:hover {
  @apply bg-pink-50 text-pink-900;
}

.nav-item.router-link-active {
  @apply bg-accent-primary/10 text-accent-primary shadow-sm;
}

/* Nav Item Styles - Dark Mode (Neko Purple Night) */
:root[data-theme="dark"] .nav-item,
.dark .nav-item {
  @apply text-pink-200/80;
}

:root[data-theme="dark"] .nav-item:hover,
.dark .nav-item:hover {
  @apply bg-pink-300/10 text-pink-100;
}

:root[data-theme="dark"] .nav-item.router-link-active,
.dark .nav-item.router-link-active {
  @apply bg-accent-primary/20 text-accent-primary;

  box-shadow: 0 0 20px rgb(var(--color-accent-primary-rgb) / 15%);
}

/* Active indicator strip */
.nav-item.router-link-active::before {
  content: '';

  @apply absolute left-0 top-1/2 -translate-y-1/2 h-4 w-1 bg-accent-primary rounded-r-full;
}

:root[data-theme="dark"] .nav-item.router-link-active::before,
.dark .nav-item.router-link-active::before {
  box-shadow: 0 0 8px rgb(var(--color-accent-primary-rgb) / 60%);
}

/* User Card - Neko Kawaii Style */
.user-card {
  background: linear-gradient(135deg,
    rgb(120 40 90 / 85%) 0%,
    rgb(140 60 130 / 75%) 50%,
    rgb(100 50 150 / 70%) 100%
  );
  border: 1px solid rgb(244 114 182 / 25%);
  box-shadow:
    0 4px 20px rgb(0 0 0 / 25%),
    inset 0 1px 0 rgb(255 255 255 / 8%);
}

.user-card:hover {
  border-color: rgb(244 114 182 / 45%);
  box-shadow:
    0 8px 32px rgb(244 114 182 / 15%),
    0 4px 20px rgb(0 0 0 / 25%),
    inset 0 1px 0 rgb(255 255 255 / 12%);
}

/* User Card - Light Mode Override */
[data-theme="light"] .user-card {
  background: linear-gradient(135deg,
    rgb(255 240 245 / 50%) 0%,
    rgb(255 228 237 / 60%) 50%,
    rgb(243 232 255 / 50%) 100%
  );
  border: 1px solid rgb(244 114 182 / 20%);
  box-shadow:
    0 4px 15px rgb(244 114 182 / 8%),
    inset 0 1px 0 rgb(255 255 255 / 80%);
}

[data-theme="light"] .user-card:hover {
  background: linear-gradient(135deg,
    rgb(255 240 245 / 70%) 0%,
    rgb(255 228 237 / 80%) 50%,
    rgb(243 232 255 / 70%) 100%
  );
  box-shadow: 0 8px 25px rgb(244 114 182 / 12%);
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
