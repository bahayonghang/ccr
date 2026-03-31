<template>
  <div
    class="layout-shell relative flex h-screen overflow-hidden font-sans text-text-primary selection:bg-accent-primary/30"
    :class="{ 'layout-shell--theme-stage': shouldUseThemeStage }"
  >
    <!-- Skip Link -->
    <a
      href="#main-content"
      class="skip-to-content layout-layer-toast"
    >
      {{ $t('common.skipToContent') || 'Skip to content' }}
    </a>

    <!-- Sidebar (Glassmorphism + Resize) -->
    <button
      v-if="showMobileBackdrop"
      type="button"
      class="fixed inset-0 layout-layer-modal-backdrop bg-slate-950/55 backdrop-blur-[2px] lg:hidden"
      :aria-label="closeNavigationLabel"
      @click="closeSidebar"
    />

    <div
      v-if="hasSidebar"
      id="primary-navigation-panel"
      class="sidebar-glass layout-sidebar flex flex-col transition-all duration-300 ease-out will-change-[width,transform]"
      :class="[
        isResizing ? 'select-none' : '',
        isMobileSidebar
          ? 'fixed inset-y-0 left-0 layout-layer-modal w-[min(86vw,320px)] max-w-[320px] border-r border-white/10 shadow-2xl shadow-slate-950/30'
          : 'relative layout-layer-dropdown flex-shrink-0',
        isMobileSidebar && !isSidebarOpen ? '-translate-x-full pointer-events-none' : 'translate-x-0',
        isMobileSidebar && isSidebarOpen ? 'pointer-events-auto' : ''
      ]"
      :style="sidebarShellStyle"
    >
      <!-- Resize Handle -->
      <button
        v-if="!isMobileSidebar"
        type="button"
        class="group absolute -right-2 top-0 layout-layer-popover h-full w-5 cursor-col-resize rounded-full focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30"
        :aria-label="t('common.resizeSidebar')"
        aria-controls="primary-navigation"
        :title="t('common.resizeSidebar')"
        @mousedown.prevent="startResize"
        @keydown="handleResizeKeydown"
      >
        <div class="absolute inset-y-0 right-1/2 w-[1px] bg-border-default/70 transition-colors delay-75 group-hover:bg-accent-primary/50" />
      </button>

      <!-- Logo Area -->
      <div class="flex h-[84px] items-center justify-between border-b border-border-default/45 px-4 pt-6 shrink-0">
        <div class="flex items-center gap-3">
          <div class="relative flex h-9 w-9 items-center justify-center rounded-xl border border-accent-primary/20 bg-gradient-to-br from-accent-primary/70 to-accent-secondary/70 shadow-md shadow-accent-primary/15">
            <SIcon
              name="Cat"
              size="w-5 h-5"
              class="text-white"
            />
          </div>
          <div class="min-w-0">
            <h1 class="truncate text-base font-semibold leading-none tracking-tight text-text-primary font-brand">
              {{ appNamePrefix }} <span class="text-accent-primary">{{ appNameSuffix }}</span>
            </h1>
            <p class="mt-1 text-[10px] font-semibold tracking-[0.18em] text-text-muted">
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
        class="flex-1 overflow-y-auto px-3 pb-4 pt-3 space-y-5 scrollbar-hide"
        aria-label="Primary navigation"
        @click="isMobileSidebar ? closeSidebar() : undefined"
      >
        <div
          v-for="section in navSections"
          :key="section.id"
        >
          <div
            v-if="section.titleKey"
            class="mb-2 flex items-center gap-2 px-3 text-[10px] font-semibold tracking-[0.16em] text-text-muted"
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
      <div class="border-t border-border-default/40 p-3 pb-5">
        <div class="user-card group relative overflow-hidden rounded-2xl transition-interactive duration-300">
          <div class="absolute inset-0 bg-gradient-to-br from-accent-primary/10 via-accent-secondary/8 to-transparent opacity-90" />
          <div class="absolute inset-0 user-card-accent-mesh" />

          <!-- Inner content -->
          <div class="relative flex flex-col gap-3 p-3.5 backdrop-blur-md">
            <div class="flex items-center justify-between">
              <!-- Session Status -->
              <p class="flex items-center gap-2 text-[11px] font-mono tracking-wide">
                <span class="text-text-muted">{{ t('common.shell.session') }}:</span>
                <span class="flex items-center gap-1.5 font-semibold text-accent-success">
                  <span class="relative flex h-2 w-2">
                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-accent-success opacity-65" />
                    <span class="relative inline-flex h-2 w-2 rounded-full bg-accent-success shadow-[0_0_6px_rgba(100,192,166,0.65)]" />
                  </span>
                  {{ t('common.shell.active') }}
                </span>
              </p>
              
              <!-- Theme Toggle -->
              <ThemeToggle class="relative" />
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
      :class="{ 'content-main--theme-stage': shouldUseThemeStage }"
    >
      <!-- Top Bar -->
      <div class="topbar-glass sticky top-0 layout-layer-sticky flex min-h-[78px] shrink-0 items-center justify-between border-b border-border-default/40 px-4 pt-5 sm:px-6 sm:pt-6">
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
        ref="contentScrollAreaRef"
        class="flex-1 overflow-y-auto scroll-smooth p-4 sm:p-6 content-scroll-area"
        :class="{ 'content-scroll-area--theme-stage': shouldUseThemeStage }"
        @scroll.passive="handleContentScroll"
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

      <ScrollToTopButton
        :visible="showScrollToTop"
        :button-label="t('common.backToTop')"
        :label="t('common.topShort')"
        @click="scrollMainContentToTop"
      />
    </main>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import ScrollToTopButton from '@/components/common/ScrollToTopButton.vue'
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { APP_NAME, APP_VERSION_LABEL } from '@/config/appMeta'
import {
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
const cachedViews = [
  'ConfigsView', 'CommandsView', 'CodexView', 'CodexAuthView', 'CodexProfilesView', 'CodexMcpView',
  'UnifiedSkillsView',
]
const MAIN_SCROLL_TOP_THRESHOLD = 480
const contentScrollAreaRef = ref<HTMLElement | null>(null)
const showScrollToTop = ref(false)
let scrollVisibilityFrame = 0

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

const shouldUseThemeStage = computed(() => Boolean(route.meta.hideGlobalBackground))

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

const syncScrollToTopVisibility = () => {
  showScrollToTop.value = (contentScrollAreaRef.value?.scrollTop ?? 0) > MAIN_SCROLL_TOP_THRESHOLD
}

const clearScrollVisibilityFrame = () => {
  if (!scrollVisibilityFrame) return

  window.cancelAnimationFrame(scrollVisibilityFrame)
  scrollVisibilityFrame = 0
}

const handleContentScroll = () => {
  if (scrollVisibilityFrame) return

  scrollVisibilityFrame = window.requestAnimationFrame(() => {
    scrollVisibilityFrame = 0
    syncScrollToTopVisibility()
  })
}

const prefersReducedMotion = (): boolean => {
  return typeof window.matchMedia === 'function'
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches
    : false
}

const scrollMainContentToTop = () => {
  const container = contentScrollAreaRef.value

  if (!container) return

  container.scrollTo({
    top: 0,
    behavior: prefersReducedMotion() ? 'auto' : 'smooth',
  })
}

watch(() => route.fullPath, async () => {
  await nextTick()
  syncScrollToTopVisibility()
})

onMounted(() => {
  syncScrollToTopVisibility()
})

onBeforeUnmount(() => {
  clearScrollVisibilityFrame()
})
</script>

<style scoped>
.layout-shell--theme-stage {
  background:
    radial-gradient(circle at top left, rgb(var(--color-accent-secondary-rgb) / 12%) 0%, transparent 24%),
    radial-gradient(circle at bottom left, rgb(var(--color-accent-primary-rgb) / 8%) 0%, transparent 22%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 100%), rgb(var(--color-bg-base-rgb) / 96%));
}

/* Sidebar Glass Effect - Unified Transparent Mode */
:root[data-theme="dark"] .sidebar-glass,
.dark .sidebar-glass,
.sidebar-glass {
  background: var(--surface-shell-bg);
  backdrop-filter: var(--surface-shell-blur);
  border-right: 1px solid var(--surface-shell-border);
  box-shadow:
    var(--surface-shell-shadow),
    inset -1px 0 0 rgb(255 255 255 / 6%);
}

.topbar-glass {
  background: var(--surface-status-bg);
  backdrop-filter: var(--surface-status-blur);
  box-shadow:
    inset 0 -1px 0 rgb(var(--color-border-default-rgb) / 42%),
    var(--surface-status-shadow);
}

.layout-layer-dropdown {
  z-index: var(--layer-dropdown);
}

.layout-layer-sticky {
  z-index: var(--layer-sticky);
}

.layout-layer-modal-backdrop {
  z-index: var(--layer-modal-backdrop);
}

.layout-layer-modal {
  z-index: var(--layer-modal);
}

.layout-layer-popover {
  z-index: var(--layer-popover);
}

.layout-layer-toast {
  z-index: var(--layer-toast);
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

.content-main--theme-stage {
  background:
    radial-gradient(circle at top right, rgb(var(--color-accent-primary-rgb) / 10%) 0%, transparent 28%),
    radial-gradient(circle at top left, rgb(var(--color-accent-secondary-rgb) / 10%) 0%, transparent 24%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 98%), rgb(var(--color-bg-base-rgb) / 94%));
}

.content-scroll-area--theme-stage {
  background:
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 10%), transparent 14rem);
}

/* Nav Item Styles */
.nav-item {
  @apply relative flex items-center gap-3 overflow-hidden rounded-xl px-3 py-2.5 text-sm font-medium text-text-secondary
         transition-interactive duration-200;

  border: 1px solid transparent;
}

.nav-item:focus-visible {
  @apply outline-none ring-2 ring-accent-primary/25 ring-offset-2 ring-offset-bg-base;
}

.nav-item:hover {
  @apply text-text-primary shadow-sm;

  background-color: rgb(var(--color-bg-overlay-rgb) / 65%);
  border-color: rgb(var(--color-border-default-rgb) / 60%);
}

.nav-item.router-link-active {
  @apply text-text-primary shadow-sm;

  box-shadow:
    0 16px 32px rgb(var(--color-accent-primary-rgb) / 10%),
    inset 0 1px 0 rgb(255 255 255 / 8%);
  background: linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 12%), rgb(var(--color-accent-secondary-rgb) / 8%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
}

/* Active indicator strip */
.nav-item.router-link-active::before {
  content: '';

  @apply absolute left-0 top-1/2 -translate-y-1/2 h-5 w-1 rounded-r-full;

  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 95%), rgb(var(--color-accent-secondary-rgb) / 92%));
  box-shadow: 0 0 8px rgb(var(--color-accent-primary-rgb) / 45%);
}

/* User Card - Neko Kawaii Style */
.user-card {
  background: linear-gradient(135deg,
    rgb(var(--color-bg-elevated-rgb) / 90%) 0%,
    rgb(var(--color-bg-surface-rgb) / 86%) 100%
  );
  backdrop-filter: blur(18px) saturate(140%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 55%);
  box-shadow:
    0 14px 32px rgb(4 6 13 / 24%),
    inset 0 1px 0 rgb(255 255 255 / 8%);
}

.user-card-accent-mesh {
  background:
    radial-gradient(ellipse at top right, rgb(var(--color-accent-primary-rgb) / 14%), transparent 54%),
    radial-gradient(ellipse at bottom left, rgb(var(--color-accent-secondary-rgb) / 10%), transparent 48%);
}

.user-card:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
  box-shadow:
    0 18px 38px rgb(var(--color-accent-primary-rgb) / 10%),
    0 6px 16px rgb(4 6 13 / 24%),
    inset 0 1px 0 rgb(255 255 255 / 10%);
}

/* User Card - Light Mode Override */
[data-theme="light"] .user-card {
  background: linear-gradient(135deg,
    rgb(var(--color-bg-elevated-rgb) / 94%) 0%,
    rgb(var(--color-bg-surface-rgb) / 92%) 100%
  );
  backdrop-filter: blur(18px) saturate(145%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 65%);
  box-shadow:
    0 16px 34px rgb(48 32 71 / 10%),
    inset 0 1px 0 rgb(255 255 255 / 75%);
}

[data-theme="light"] .user-card:hover {
  box-shadow: 0 18px 38px rgb(48 32 71 / 12%);
}

</style>
