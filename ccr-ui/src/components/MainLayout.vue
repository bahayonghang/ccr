<template>
  <div class="flex h-screen text-text-primary overflow-hidden font-sans selection:bg-accent-primary/30">
    <!-- Skip Link -->
    <a
      href="#main-content"
      class="skip-to-content z-50"
    >
      {{ $t('common.skipToContent') || 'Skip to content' }}
    </a>

    <!-- Sidebar (Glassmorphism + Resize) -->
    <div
      v-if="!route.meta.hideSidebar"
      class="flex flex-col relative flex-shrink-0 z-40 transition-all duration-300 ease-out will-change-[width] sidebar-glass"
      :class="[isResizing ? 'select-none' : '']"
      :style="{ width: sidebarWidth + 'px' }"
    >
      <!-- Resize Handle -->
      <button
        type="button"
        class="group absolute -right-2 top-0 z-50 h-full w-5 cursor-col-resize rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30"
        :aria-label="t('common.resizeSidebar')"
        aria-controls="primary-navigation"
        :title="t('common.resizeSidebar')"
        @mousedown.prevent="startResize"
        @keydown="handleResizeKeydown"
      >
        <div class="absolute inset-y-0 right-1/2 w-[1px] bg-border-default/70 transition-colors delay-75 group-hover:bg-accent-primary/50" />
      </button>

      <!-- Logo Area -->
      <div class="h-[100px] pt-9 flex items-center px-4 border-b border-pink-200/20 dark:border-pink-300/10 shrink-0">
        <div class="flex items-center gap-3">
          <div class="relative flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-pink-400 to-violet-400 shadow-lg shadow-pink-400/30">
            <SIcon
              name="Cat"
              size="w-5 h-5"
              class="text-white"
            />
          </div>
          <div>
            <h1 class="text-lg font-bold font-display tracking-tight leading-none text-text-primary">
              CCR <span class="text-accent-primary">UI</span>
            </h1>
            <p class="text-[10px] uppercase tracking-widest text-pink-400 dark:text-pink-300 font-bold mt-0.5">
              Neko Console
            </p>
          </div>
        </div>
      </div>

      <!-- Navigation -->
      <nav
        id="primary-navigation"
        class="flex-1 overflow-y-auto p-3 space-y-6 scrollbar-hide"
      >
        <!-- Section: Main -->
        <div class="space-y-1">
          <RouterLink
            to="/"
            class="nav-item"
          >
            <SIcon
              name="Home"
              size="w-4 h-4"
            />
            <span>{{ $t('nav.home') }}</span>
          </RouterLink>
        </div>

        <!-- Section: Skills Hub -->
        <div>
          <div class="px-3 mb-2 text-[10px] font-bold uppercase tracking-wider text-text-muted flex items-center gap-2">
            {{ $t('nav.skillsHub') }}
            <div class="h-px flex-1 bg-border-default/70" />
          </div>
          <div class="space-y-0.5">
            <RouterLink
              to="/skills"
              class="nav-item group"
            >
              <SIcon
                name="Package"
                size="w-4 h-4"
                class="text-fuchsia-400 group-hover:text-fuchsia-300 transition-colors"
              />
              <span>{{ $t('nav.skills') }}</span>
            </RouterLink>
            <RouterLink
              to="/skills/add"
              class="nav-item group"
            >
              <SIcon
                name="PlusCircle"
                size="w-4 h-4"
                class="text-fuchsia-400 group-hover:text-fuchsia-300 transition-colors"
              />
              <span>{{ $t('nav.addSkill') }}</span>
            </RouterLink>
          </div>
        </div>

        <!-- Section: Modules -->
        <div>
          <div class="px-3 mb-2 text-[10px] font-bold uppercase tracking-wider text-text-muted flex items-center gap-2">
            {{ $t('nav.mainModules') }}
            <div class="h-px flex-1 bg-border-default/70" />
          </div>
          <div class="space-y-0.5">
            <RouterLink
              to="/claude-code"
              class="nav-item group"
            >
              <SIcon
                name="Code2"
                size="w-4 h-4"
                class="text-platform-claude group-hover:text-platform-claude/80 transition-colors"
              />
              <span>{{ $t('nav.claudeCode') }}</span>
            </RouterLink>
            <RouterLink
              to="/codex"
              class="nav-item group"
            >
              <SIcon
                name="Settings"
                size="w-4 h-4"
                class="text-platform-codex group-hover:text-platform-codex/80 transition-colors"
              />
              <span>{{ $t('nav.codex') }}</span>
            </RouterLink>
            <RouterLink
              to="/gemini-cli"
              class="nav-item group"
            >
              <SIcon
                name="Sparkles"
                size="w-4 h-4"
                class="text-platform-gemini group-hover:text-platform-gemini/80 transition-colors"
              />
              <span>{{ $t('nav.gemini') }}</span>
            </RouterLink>
            <RouterLink
              to="/qwen"
              class="nav-item group"
            >
              <SIcon
                name="Zap"
                size="w-4 h-4"
                class="text-platform-qwen group-hover:text-platform-qwen/80 transition-colors"
              />
              <span>{{ $t('nav.qwen') }}</span>
            </RouterLink>
            <RouterLink
              to="/iflow"
              class="nav-item group"
            >
              <SIcon
                name="Activity"
                size="w-4 h-4"
                class="text-platform-iflow group-hover:text-platform-iflow/80 transition-colors"
              />
              <span>{{ $t('nav.iflow') }}</span>
            </RouterLink>
            <RouterLink
              to="/droid"
              class="nav-item group"
            >
              <SIcon
                name="Bot"
                size="w-4 h-4"
                class="text-accent-secondary group-hover:text-accent-secondary/80 transition-colors"
              />
              <span>{{ $t('nav.droid') }}</span>
            </RouterLink>
          </div>
        </div>

        <!-- Section: Tools -->
        <div>
          <div class="px-3 mb-2 text-[10px] font-bold uppercase tracking-wider text-text-muted flex items-center gap-2">
            {{ $t('nav.toolsCenter') }}
            <div class="h-px flex-1 bg-border-default/70" />
          </div>
          <div class="space-y-0.5">
            <RouterLink
              to="/ccr-control"
              class="nav-item"
            >
              <SIcon
                name="Terminal"
                size="w-4 h-4"
              />
              <span>{{ $t('nav.ccrControl') }}</span>
            </RouterLink>
            <RouterLink
              to="/commands"
              class="nav-item"
            >
              <SIcon
                name="Terminal"
                size="w-4 h-4"
              />
              <span>{{ $t('nav.commands') }}</span>
            </RouterLink>
            <RouterLink
              to="/checkin"
              class="nav-item"
            >
              <SIcon
                name="ClipboardList"
                size="w-4 h-4"
              />
              <span>{{ $t('nav.checkin') }}</span>
            </RouterLink>
            <RouterLink
              to="/sync"
              class="nav-item"
            >
              <SIcon
                name="Cloud"
                size="w-4 h-4"
              />
              <span>{{ $t('nav.sync') }}</span>
            </RouterLink>
            <RouterLink
              to="/usage"
              class="nav-item"
            >
              <SIcon
                name="Activity"
                size="w-4 h-4"
              />
              <span>{{ $t('nav.usage') }}</span>
            </RouterLink>
          </div>
        </div>
      </nav>

      <!-- Footer: User Profile - Neko Kawaii Style -->
      <div class="p-3 pb-6 border-t border-pink-200/10 dark:border-pink-300/10">
        <div class="user-card group relative overflow-hidden rounded-2xl transition-[color,background-color,box-shadow,border-color] duration-300">
          <!-- Animated gradient background -->
          <div class="absolute inset-0 bg-gradient-to-br from-pink-500/15 via-fuchsia-500/12 to-violet-500/15 opacity-80" />
          <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,rgba(244,114,182,0.12),transparent_50%)]" />

          <!-- Neko dot pattern -->
          <div class="absolute inset-0 opacity-10 bg-[radial-gradient(circle,rgba(255,255,255,0.15)_1px,transparent_1px)] bg-[size:10px_10px]" />

          <!-- Glow effect on hover -->
          <div class="absolute -inset-1 bg-gradient-to-r from-pink-500/0 via-fuchsia-400/20 to-violet-500/0 blur-xl opacity-0 group-hover:opacity-100 transition-opacity duration-500" />

          <!-- Inner content -->
          <div class="relative p-3.5 backdrop-blur-md z-10 flex flex-col gap-3">
            <div class="flex items-center justify-between">
              <!-- Session Status -->
              <p class="text-[11px] font-mono uppercase tracking-wider flex items-center gap-2">
                <span class="text-text-muted">Session:</span>
                <span class="flex items-center gap-1.5 text-emerald-400 font-semibold">
                  <span class="relative flex h-2 w-2">
                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                    <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.8)]" />
                  </span>
                  Active
                </span>
              </p>
              
              <!-- Theme Toggle -->
              <ThemeToggle class="relative z-20" />
            </div>

            <!-- Version -->
            <div class="flex items-center justify-between">
              <span class="rounded-md border border-border-default/60 bg-bg-elevated/80 px-2 py-0.5 text-[10px] font-mono text-text-muted">
                CCR UI v5.1.5
              </span>
            </div>
          </div>

          <!-- Bottom accent line -->
          <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-gradient-to-r from-transparent via-pink-400/50 to-transparent" />
        </div>
      </div>
    </div>

    <!-- Main Content Area -->
    <main
      id="main-content"
      class="flex-1 relative overflow-hidden flex flex-col content-main"
    >
      <!-- Top Bar -->
      <div class="sticky top-0 z-30 flex h-[92px] pt-9 items-center justify-between border-b border-border-default/40 topbar-glass px-6 shrink-0">
        <!-- Left: Breadcrumbs or Back + Title -->
        <div class="flex items-center text-sm text-text-secondary">
          <template v-if="route.meta.hideSidebar">
            <button
              class="flex items-center gap-1.5 px-2.5 py-1.5 -ml-2 rounded-lg text-text-secondary hover:text-text-primary hover:bg-bg-overlay/70 transition-colors duration-200"
              @click="router.back()"
            >
              <SIcon
                name="ArrowLeft"
                size="w-4 h-4"
              />
              <span class="text-xs font-medium">{{ t('common.back') }}</span>
            </button>
            <span class="mx-2 opacity-30">|</span>
            <span class="text-text-primary font-semibold">{{ currentPageTitle }}</span>
          </template>
          <template v-else>
            <span class="opacity-50">{{ $t('nav.mainModules') }}</span>
            <span class="mx-2 opacity-30">/</span>
            <span class="text-text-primary font-medium">{{ currentPageTitle }}</span>
          </template>
        </div>

        <div class="flex items-center gap-4">
          <!-- 环境切换器 (仅 Tauri 桌面模式) -->
          <EnvironmentSwitcher v-if="isTauri" />
          <LanguageSwitcher />
          <div
            v-if="isTauri"
            class="h-4 w-px bg-border-default/80 mx-2"
          />
          <!-- Exit Toggle -->
          <button
            v-if="isTauri"
            class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors duration-200 border whitespace-nowrap flex-shrink-0"
            :class="[
              showExitConfirm 
                ? 'bg-accent-primary/10 border-accent-primary/30 text-accent-primary' 
                : 'border-border-default/70 bg-bg-elevated/75 text-text-secondary hover:text-text-primary hover:border-accent-primary/30 hover:bg-bg-surface/90 shadow-sm'
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

      <!-- Scrollable Content -->
      <div
        class="flex-1 overflow-y-auto scroll-smooth p-6 content-scroll-area"
      >
        <BackendStatusBanner class="mb-6" />
        <RouterView v-slot="{ Component }">
          <transition
            :name="transitionName"
            mode="out-in"
          >
            <!-- 懒加载路由 Suspense 边界，防止异步组件加载时出现空白闪烁 -->
            <keep-alive
              :include="cachedViews"
              :max="10"
            >
              <Suspense>
                <component :is="Component" />
                <template #fallback>
                  <div class="flex items-center justify-center min-h-[200px]">
                    <div class="loading-spinner w-8 h-8 border-accent-primary/30 border-t-accent-primary" />
                  </div>
                </template>
              </Suspense>
            </keep-alive>
          </transition>
        </RouterView>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import BackendStatusBanner from '@/components/BackendStatusBanner.vue'
import LanguageSwitcher from '@/components/LanguageSwitcher.vue'
import ThemeToggle from '@/components/ThemeToggle.vue'
import EnvironmentSwitcher from '@/components/EnvironmentSwitcher.vue'
import { isTauriEnvironment, getSkipExitConfirm, setSkipExitConfirm } from '@/api/tauri'
import { usePageTransition } from '@/composables/usePageTransition'
import { logger } from '@/utils/logger'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const { transitionName } = usePageTransition()

// keep-alive 缓存列表（仅缓存高频访问页面）
const cachedViews = [
  'HomeView', 'ConfigsView', 'CommandsView', 'ClaudeCodeView',
  'CodexView', 'CodexAuthView', 'CodexProfilesView', 'CodexMcpView',
  'GeminiCliView', 'QwenView', 'IflowView',
]

// 路由名 → i18n 键映射
const routeTitleMap: Record<string, string> = {
  home: 'nav.home',
  skills: 'nav.skills',
  'skills-add': 'nav.addSkill',
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
  'mcp-unified': 'MCP 统一管理',
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

const handleResizeKeydown = (event: KeyboardEvent) => {
  const step = event.shiftKey ? 32 : 16
  if (event.key === 'ArrowLeft') {
    event.preventDefault()
    sidebarWidth.value = Math.max(minWidth, sidebarWidth.value - step)
  } else if (event.key === 'ArrowRight') {
    event.preventDefault()
    sidebarWidth.value = Math.min(maxWidth, sidebarWidth.value + step)
  } else if (event.key === 'Home') {
    event.preventDefault()
    sidebarWidth.value = minWidth
  } else if (event.key === 'End') {
    event.preventDefault()
    sidebarWidth.value = maxWidth
  } else {
    return
  }

  localStorage.setItem('ccr-sidebar-width', sidebarWidth.value.toString())
}

onMounted(async () => {
  const savedWidth = localStorage.getItem('ccr-sidebar-width')
  if (savedWidth) sidebarWidth.value = Number(savedWidth) || 240
  
  isTauri.value = isTauriEnvironment()
  if (isTauri.value) {
    try {
      const skipConfirm = await getSkipExitConfirm()
      showExitConfirm.value = !skipConfirm
    } catch (e) {
      logger.error('Failed to load exit confirmation preference', e)
    }
  }
})

onUnmounted(() => {
  window.removeEventListener('mousemove', handleResize)
  window.removeEventListener('mouseup', stopResize)
})
</script>

<style scoped>
/* Sidebar Glass Effect - Unified Transparent Mode */
:root[data-theme="dark"] .sidebar-glass,
.dark .sidebar-glass,
.sidebar-glass {
  background: var(--liquid-glass-bg);
  backdrop-filter: var(--liquid-glass-blur);
  border-right: 1px solid var(--liquid-glass-border);
  box-shadow:
    var(--liquid-glass-shadow),
    inset -1px 0 0 rgb(255 255 255 / 8%);
}

.topbar-glass {
  background: var(--glass-bg-light);
  backdrop-filter: var(--glass-blur-md);
  box-shadow: inset 0 -1px 0 rgb(var(--color-border-default-rgb) / 45%);
}

[data-theme="light"] .sidebar-glass {
  background: rgb(255 240 245 / 80%);
}

[data-theme="light"] .topbar-glass {
  background: rgb(255 245 247 / 82%);
}

/* Content area compositing fix:
 * Force main + scroll container onto GPU compositing layers so that
 * children's backdrop-filter can blur through to the fixed AnimeBackground.
 * Using backface-visibility instead of transform to avoid creating a
 * containing block for fixed-positioned descendants. */
.content-main,
.content-scroll-area {
  backface-visibility: hidden;
}

/* Nav Item Styles */
.nav-item {
  @apply relative flex items-center gap-3 overflow-hidden rounded-xl px-3 py-2 text-sm font-medium text-text-secondary
         transition-[color,background-color,box-shadow] duration-200;
}

.nav-item:focus-visible {
  @apply outline-none ring-2 ring-accent-primary/25 ring-offset-2 ring-offset-bg-base;
}

.nav-item:hover {
  @apply bg-bg-overlay/75 text-text-primary shadow-sm;
}

.nav-item.router-link-active {
  @apply border border-accent-primary/15 bg-accent-primary/10 text-accent-primary shadow-sm;

  box-shadow:
    0 10px 24px rgb(var(--color-accent-primary-rgb) / 12%),
    0 0 20px rgb(var(--color-accent-primary-rgb) / 8%);
}

/* Active indicator strip */
.nav-item.router-link-active::before {
  content: '';

  @apply absolute left-0 top-1/2 -translate-y-1/2 h-4 w-1 bg-accent-primary rounded-r-full;

  box-shadow: 0 0 8px rgb(var(--color-accent-primary-rgb), 0.6);
}

/* User Card - Neko Kawaii Style */
.user-card {
  background: linear-gradient(135deg,
    rgb(120 40 90 / 25%) 0%,
    rgb(140 60 130 / 15%) 50%,
    rgb(100 50 150 / 20%) 100%
  );
  backdrop-filter: blur(16px) saturate(180%);
  border: 1px solid rgb(244 114 182 / 25%);
  box-shadow:
    0 4px 24px rgb(0 0 0 / 20%),
    inset 0 1px 0 rgb(255 255 255 / 10%);
}

.user-card:hover {
  border-color: rgb(244 114 182 / 45%);
  background: linear-gradient(135deg,
    rgb(120 40 90 / 35%) 0%,
    rgb(140 60 130 / 25%) 50%,
    rgb(100 50 150 / 30%) 100%
  );
  box-shadow:
    0 8px 32px rgb(244 114 182 / 15%),
    0 4px 20px rgb(0 0 0 / 25%),
    inset 0 1px 0 rgb(255 255 255 / 12%);
}

/* User Card - Light Mode Override */
[data-theme="light"] .user-card {
  background: linear-gradient(135deg,
    rgb(255 240 245 / 72%) 0%,
    rgb(255 228 237 / 80%) 50%,
    rgb(243 232 255 / 72%) 100%
  );
  backdrop-filter: blur(16px) saturate(180%);
  border: 1px solid rgb(244 114 182 / 35%);
  box-shadow:
    0 4px 15px rgb(244 114 182 / 10%),
    inset 0 1px 0 rgb(255 255 255 / 90%);
}

[data-theme="light"] .user-card:hover {
  background: linear-gradient(135deg,
    rgb(255 240 245 / 82%) 0%,
    rgb(255 228 237 / 88%) 50%,
    rgb(243 232 255 / 82%) 100%
  );
  box-shadow: 0 8px 25px rgb(244 114 182 / 12%);
}

</style>
