<template>
  <div class="relative flex h-screen overflow-hidden font-sans text-text-primary selection:bg-accent-primary/30">
    <!-- Skip Link -->
    <a
      href="#main-content"
      class="skip-to-content z-50"
    >
      {{ $t('common.skipToContent') || 'Skip to content' }}
    </a>

    <!-- Sidebar (Glassmorphism + Resize) -->
    <button
      v-if="showMobileBackdrop"
      type="button"
      class="fixed inset-0 z-40 bg-slate-950/55 backdrop-blur-[2px] lg:hidden"
      :aria-label="closeNavigationLabel"
      @click="closeSidebar"
    />

    <div
      v-if="hasSidebar"
      id="primary-navigation-panel"
      class="sidebar-glass flex flex-col transition-all duration-300 ease-out will-change-[width,transform]"
      :class="[
        isResizing ? 'select-none' : '',
        isMobileSidebar
          ? 'fixed inset-y-0 left-0 z-50 w-[min(86vw,320px)] max-w-[320px] border-r border-white/10 shadow-2xl shadow-slate-950/30'
          : 'relative z-40 flex-shrink-0',
        isMobileSidebar && !isSidebarOpen ? '-translate-x-full pointer-events-none' : 'translate-x-0',
        isMobileSidebar && isSidebarOpen ? 'pointer-events-auto' : ''
      ]"
      :style="sidebarShellStyle"
    >
      <!-- Resize Handle -->
      <button
        v-if="!isMobileSidebar"
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
      <div class="h-[100px] pt-9 flex items-center justify-between px-4 border-b border-pink-200/20 dark:border-pink-300/10 shrink-0">
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
              {{ appNamePrefix }} <span class="text-accent-primary">{{ appNameSuffix }}</span>
            </h1>
            <p class="text-[10px] uppercase tracking-widest text-pink-400 dark:text-pink-300 font-bold mt-0.5">
              {{ t('common.shell.tagline') }}
            </p>
          </div>
        </div>
        <button
          v-if="isMobileSidebar"
          type="button"
          class="inline-flex h-11 w-11 flex-none items-center justify-center rounded-2xl border border-border-default/70 bg-bg-surface/80 text-text-primary shadow-sm transition-colors hover:border-accent-primary/30 hover:bg-bg-elevated/90 lg:hidden"
          :aria-label="closeNavigationLabel"
          :title="closeNavigationLabel"
          @click="closeSidebar"
        >
          <SIcon
            name="X"
            size="w-4 h-4"
          />
        </button>
      </div>

      <!-- Navigation -->
      <nav
        id="primary-navigation"
        class="flex-1 overflow-y-auto p-3 space-y-6 scrollbar-hide"
        aria-label="Primary navigation"
        @click="isMobileSidebar ? closeSidebar() : undefined"
      >
        <div
          v-for="section in navSections"
          :key="section.id"
        >
          <div
            v-if="section.titleKey"
            class="px-3 mb-2 text-[10px] font-bold uppercase tracking-wider text-text-muted flex items-center gap-2"
          >
            {{ $t(section.titleKey) }}
            <div class="h-px flex-1 bg-border-default/70" />
          </div>
          <div :class="section.titleKey ? 'space-y-0.5' : 'space-y-1'">
            <RouterLink
              v-for="item in section.items"
              :key="item.to"
              :to="item.to"
              class="nav-item group"
            >
              <SIcon
                :name="item.icon"
                size="w-4 h-4"
                :class="item.iconClass"
              />
              <span>{{ $t(item.labelKey) }}</span>
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
                <span class="text-text-muted">{{ t('common.shell.session') }}:</span>
                <span class="flex items-center gap-1.5 text-emerald-400 font-semibold">
                  <span class="relative flex h-2 w-2">
                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                    <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-400 shadow-[0_0_6px_rgba(52,211,153,0.8)]" />
                  </span>
                  {{ t('common.shell.active') }}
                </span>
              </p>
              
              <!-- Theme Toggle -->
              <ThemeToggle class="relative z-20" />
            </div>

            <!-- Version -->
            <div class="flex items-center justify-between">
              <span class="rounded-md border border-border-default/60 bg-bg-elevated/80 px-2 py-0.5 text-[10px] font-mono text-text-muted">
                {{ appVersionLabel }}
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
      class="relative flex min-w-0 flex-1 flex-col overflow-hidden content-main"
    >
      <!-- Top Bar -->
      <div class="topbar-glass sticky top-0 z-30 flex min-h-[92px] shrink-0 items-center justify-between border-b border-border-default/40 px-4 pt-7 sm:px-6 sm:pt-9">
        <!-- Left: Breadcrumbs or Back + Title -->
        <div class="flex min-w-0 items-center gap-3 text-sm text-text-secondary">
          <button
            v-if="hasSidebar && isMobileSidebar"
            type="button"
            class="inline-flex h-11 w-11 flex-none items-center justify-center rounded-2xl border border-border-default/70 bg-bg-surface/80 text-text-primary shadow-sm transition-colors hover:border-accent-primary/30 hover:bg-bg-elevated/90 lg:hidden"
            :aria-expanded="isSidebarOpen"
            aria-controls="primary-navigation-panel"
            :aria-label="sidebarToggleLabel"
            :title="sidebarToggleLabel"
            @click="toggleSidebar"
          >
            <SIcon
              :name="isSidebarOpen ? 'X' : 'Menu'"
              size="w-5 h-5"
            />
          </button>
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
            <span class="truncate opacity-50">{{ currentSectionTitle }}</span>
            <template v-if="currentSectionTitle !== currentPageTitle">
              <span class="mx-2 opacity-30">/</span>
              <span class="truncate text-text-primary font-medium">{{ currentPageTitle }}</span>
            </template>
          </template>
        </div>

        <div class="ml-4 flex items-center gap-2 sm:gap-4">
          <!-- 环境切换器 (仅 Tauri 桌面模式) -->
          <EnvironmentSwitcher
            v-if="isTauri && !isMobileSidebar"
          />
          <LanguageSwitcher />
          <div
            v-if="isTauri && !isMobileSidebar"
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
        class="flex-1 overflow-y-auto scroll-smooth p-4 sm:p-6 content-scroll-area"
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
import { computed, defineAsyncComponent } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { APP_NAME, APP_VERSION_LABEL } from '@/config/appMeta'
import {
  mainLayoutCachedViews,
  mainLayoutGroupTitleMap,
  mainLayoutNavSections,
  mainLayoutRouteTitleMap,
} from '@/config/mainLayoutShell'
import { usePageTransition } from '@/composables/usePageTransition'
import { useMainLayoutShell } from '@/composables/useMainLayoutShell'

const BackendStatusBanner = defineAsyncComponent({
  loader: () => import('@/components/BackendStatusBanner.vue'),
  suspensible: false,
})

const LanguageSwitcher = defineAsyncComponent({
  loader: () => import('@/components/LanguageSwitcher.vue'),
  suspensible: false,
})

const ThemeToggle = defineAsyncComponent({
  loader: () => import('@/components/ThemeToggle.vue'),
  suspensible: false,
})

const EnvironmentSwitcher = defineAsyncComponent({
  loader: () => import('@/components/EnvironmentSwitcher.vue'),
  suspensible: false,
})

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const { transitionName } = usePageTransition()
const [appNamePrefix = APP_NAME, appNameSuffix = ''] = APP_NAME.split(' ')
const appVersionLabel = APP_VERSION_LABEL
const navSections = mainLayoutNavSections
const cachedViews = mainLayoutCachedViews

const currentPageTitle = computed(() => {
  const name = route.name as string
  const key = mainLayoutRouteTitleMap[name]
  return key ? t(key) : t('nav.home')
})

const currentSectionTitle = computed(() => {
  const group = route.meta.group as string | undefined
  if (!group) return t('nav.home')
  const key = mainLayoutGroupTitleMap[group]
  return key ? t(key) : t('nav.home')
})

const hasSidebar = computed(() => !route.meta.hideSidebar)
const {
  closeNavigationLabel,
  closeSidebar,
  handleResizeKeydown,
  isMobileSidebar,
  isResizing,
  isSidebarOpen,
  isTauri,
  showExitConfirm,
  showMobileBackdrop,
  sidebarShellStyle,
  sidebarToggleLabel,
  startResize,
  toggleExitConfirm,
  toggleSidebar,
} = useMainLayoutShell({
  hasSidebar,
  routeFullPath: computed(() => route.fullPath),
  t,
})
</script>

<style scoped>
/* Sidebar Glass Effect - Unified Transparent Mode */
:root[data-theme="dark"] .sidebar-glass,
.dark .sidebar-glass,
.sidebar-glass {
  background: var(--surface-shell-bg);
  backdrop-filter: var(--surface-shell-blur);
  border-right: 1px solid var(--surface-shell-border);
  box-shadow:
    var(--surface-shell-shadow),
    inset -1px 0 0 rgb(255 255 255 / 8%);
}

.topbar-glass {
  background: var(--surface-status-bg);
  backdrop-filter: var(--surface-status-blur);
  box-shadow:
    inset 0 -1px 0 rgb(var(--color-border-default-rgb) / 45%),
    var(--surface-status-shadow);
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
