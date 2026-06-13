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

    <!-- Sidebar (editorial surface + resize) -->
    <button
      v-if="showMobileBackdrop"
      type="button"
      class="fixed inset-0 layout-layer-modal-backdrop bg-black/55 backdrop-blur-[2px] lg:hidden"
      :aria-label="closeNavigationLabel"
      @click="closeSidebar"
    />

    <div
      v-if="hasSidebar"
      id="primary-navigation-panel"
      class="sidebar-glass layout-sidebar flex flex-col transition-[width,transform,background-color,border-color,box-shadow] duration-300 ease-out will-change-[transform]"
      :class="[
        isResizing ? 'select-none' : '',
        isMobileSidebar
          ? 'fixed inset-y-0 left-0 layout-layer-modal w-[min(86vw,320px)] max-w-[320px] border-r border-border-default/20 shadow-2xl shadow-black/15'
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
          <div class="relative flex h-10 w-10 items-center justify-center overflow-hidden rounded-[0.9rem] border border-border-default/50 bg-bg-elevated shadow-sm">
            <img
              :src="appIconUrl"
              alt="CCR UI"
              class="h-full w-full object-cover"
            >
          </div>
          <div class="min-w-0">
            <h1 class="truncate text-[1.08rem] font-medium leading-none tracking-[-0.045em] text-text-primary font-brand">
              {{ APP_NAME }}
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
              :class="{ 'nav-item--root': item.to === '/' }"
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

      <!-- Footer: Settings Dock -->
      <div class="border-t border-border-default/40 p-3 pb-5">
        <RouterLink
          v-slot="{ href, navigate }"
          to="/settings"
          custom
        >
          <a
            :href="href"
            class="settings-dock group relative block overflow-hidden rounded-[1.6rem] transition-interactive duration-300"
            :class="{ 'settings-dock--active': isSettingsRoute }"
            data-testid="settings-dock-link"
            :aria-current="isSettingsRoute ? 'page' : undefined"
            @click="navigate"
          >
            <div class="absolute inset-0 bg-gradient-to-br from-accent-primary/12 via-accent-secondary/10 to-transparent opacity-90" />

            <div class="relative flex flex-col gap-3 p-3.5">
              <div class="flex items-start justify-between gap-3">
                <div class="space-y-1.5">
                  <p class="flex items-center gap-2 text-[11px] font-mono tracking-wide">
                    <span class="text-text-muted">{{ t('common.shell.session') }}:</span>
                    <span class="flex items-center gap-1.5 font-semibold text-success">
                      <span class="relative flex h-2 w-2">
                        <span class="absolute inline-flex h-full w-full rounded-full bg-success/30" />
                        <span class="relative inline-flex h-2 w-2 rounded-full bg-success" />
                      </span>
                      {{ t('common.shell.active') }}
                    </span>
                  </p>
                  <div>
                    <p class="text-sm font-semibold tracking-[-0.02em] text-text-primary">
                      {{ t('nav.settings') }}
                    </p>
                    <p class="text-[11px] leading-5 text-text-secondary">
                      {{ t('settings.dock.summary') }}
                    </p>
                  </div>
                </div>

                <div class="flex h-10 w-10 flex-none items-center justify-center rounded-2xl border border-border-default/60 bg-bg-elevated/82 text-text-primary shadow-sm transition-colors duration-200 group-hover:border-accent-primary/30 group-hover:bg-bg-surface/90 group-hover:text-accent-primary">
                  <SIcon
                    name="SlidersHorizontal"
                    size="w-4 h-4"
                  />
                </div>
              </div>

              <div class="flex flex-wrap items-center gap-2">
                <span class="settings-dock-pill">
                  {{ currentThemeLabel }}
                </span>
                <span class="settings-dock-pill">
                  {{ currentLocaleLabel }}
                </span>
                <span class="settings-dock-pill font-mono">
                  {{ appVersionLabel }}
                </span>
              </div>
            </div>

            <div class="absolute bottom-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-accent-primary/60 to-transparent" />
          </a>
        </RouterLink>
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
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { APP_NAME, APP_VERSION_LABEL } from '@/config/appMeta'
import {
  mainLayoutGroupTitleMap,
  mainLayoutNavSections,
  mainLayoutRouteTitleMap,
} from '@/config/mainLayoutShell'
import { translateWithFallback } from '@/i18n/formatMessage'
import { usePageTransition } from '@/composables/usePageTransition'
import { useMainLayoutShell } from '@/composables/useMainLayoutShell'
import { useShellPreferencesStore } from '@/stores/shellPreferences'
import { collectCachedComponentNames } from '@/router'

const BackendStatusBanner = defineAsyncComponent({
  loader: () => import('@/components/BackendStatusBanner.vue'),
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
const appVersionLabel = APP_VERSION_LABEL
const appIconUrl = '/icons/icon.svg'
const navSections = mainLayoutNavSections
const cachedViews = collectCachedComponentNames()
const MAIN_SCROLL_TOP_THRESHOLD = 480
const contentScrollAreaRef = ref<HTMLElement | null>(null)
const showScrollToTop = ref(false)
let scrollVisibilityFrame = 0
const shellPreferencesStore = useShellPreferencesStore()
const { theme, effectiveTheme, locale } = storeToRefs(shellPreferencesStore)

const currentPageTitle = computed(() => {
  const name = route.name as string
  const key = mainLayoutRouteTitleMap[name]
  return key ? t(key) : t('nav.dashboard')
})

const currentSectionTitle = computed(() => {
  const group = route.meta.group as string | undefined
  if (!group) return t('nav.dashboard')
  const key = mainLayoutGroupTitleMap[group]
  return key ? t(key) : t('nav.dashboard')
})

const shouldUseThemeStage = computed(() => Boolean(route.meta.hideGlobalBackground))
const isSettingsRoute = computed(() => route.name === 'settings')
const currentLocaleLabel = computed(() => (
  locale.value === 'en-US' ? t('language.english') : t('language.chinese')
))
const currentThemeLabel = computed(() => {
  if (theme.value === 'system') {
    const resolvedLabel = t(`theme.${effectiveTheme.value}`)
    return translateWithFallback(
      t,
      'settings.appearance.systemSummary',
      `${t('theme.system')} · {resolved}`,
      { resolved: resolvedLabel },
    )
  }

  return t(`theme.${theme.value}`)
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
  showMobileBackdrop,
  sidebarShellStyle,
  sidebarToggleLabel,
  startResize,
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
    radial-gradient(circle at top left, rgb(var(--color-accent-primary-rgb) / 6%) 0%, transparent 24%),
    radial-gradient(circle at bottom right, rgb(var(--color-premium-blue-rgb) / 28%) 0%, transparent 30%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 100%), rgb(var(--color-bg-base-rgb) / 96%));
}

/* Sidebar Glass Effect - Unified Transparent Mode */
:root[data-theme="dark"] .sidebar-glass,
.sidebar-glass {
  background: var(--surface-shell-bg);
  backdrop-filter: var(--surface-shell-blur);
  border-right: 1px solid var(--surface-shell-border);
  box-shadow:
    var(--surface-shell-shadow),
    inset -1px 0 0 rgb(var(--color-border-default-rgb) / 10%),
    var(--glass-inner-glow);
}

.topbar-glass {
  background: var(--surface-status-bg);
  backdrop-filter: var(--surface-status-blur);
  box-shadow:
    inset 0 -1px 0 rgb(var(--color-border-default-rgb) / 18%),
    var(--glass-inner-glow),
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
 * children's backdrop-filter can blur through to the fixed StageBackground.
 * Using backface-visibility instead of transform to avoid creating a
 * containing block for fixed-positioned descendants. */
.content-main,
.content-scroll-area {
  backface-visibility: hidden;
}

.content-main--theme-stage {
  background:
    radial-gradient(circle at top right, rgb(var(--color-accent-primary-rgb) / 8%) 0%, transparent 24%),
    radial-gradient(circle at top left, rgb(var(--color-premium-blue-rgb) / 22%) 0%, transparent 24%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 98%), rgb(var(--color-bg-base-rgb) / 94%));
}

.content-scroll-area--theme-stage {
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 16%), transparent 14rem);
}

/* Nav Item Styles */
.nav-item {
  @apply relative flex items-center gap-3 overflow-hidden px-3 py-2.5 text-sm font-medium text-text-secondary
         transition-interactive duration-200;

  border: 1px solid rgb(var(--color-border-default-rgb) / 0%);
  border-radius: var(--radius-lg);
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 0%), rgb(var(--color-bg-surface-rgb) / 0%));
}

.nav-item:focus-visible {
  @apply outline-none ring-2 ring-accent-primary/25 ring-offset-2 ring-offset-bg-base;
}

.nav-item:hover {
  @apply text-text-primary;

  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 92%), rgb(var(--color-bg-surface-rgb) / 82%));
  border-color: rgb(var(--color-border-default-rgb) / 54%);
  box-shadow: var(--shadow-md);
}

/* 激活态：去发光，改用边框 + 内描边强调（左侧 accent 标记保留） */
.nav-item.router-link-active:not(.nav-item--root),
.nav-item.router-link-exact-active.nav-item--root {
  @apply text-text-primary;

  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 94%), rgb(var(--color-bg-surface-rgb) / 86%));
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  box-shadow: inset 0 0 0 1px rgb(var(--color-accent-primary-rgb) / 10%);
}

/* Active indicator strip */
.nav-item.router-link-active:not(.nav-item--root)::before,
.nav-item.router-link-exact-active.nav-item--root::before {
  content: '';

  @apply absolute left-0 top-1/2 -translate-y-1/2 h-4 w-1 rounded-r-full;

  background: rgb(var(--color-accent-primary-rgb) / 100%);
}

/* 设置坞：扁平不透明表面 + 语义边框，去 backdrop-filter / 去发光 / 去 accent mesh */
.settings-dock {
  background: rgb(var(--color-bg-elevated-rgb) / 94%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  box-shadow: var(--surface-card-shadow);
}

.settings-dock:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  box-shadow: var(--shadow-md);
}

.settings-dock--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
  box-shadow: inset 0 0 0 1px rgb(var(--color-accent-primary-rgb) / 12%);
}

.settings-dock-pill {
  @apply inline-flex items-center border px-2.5 py-1 text-[10px] font-semibold tracking-[0.04em];

  border-radius: var(--radius-sm);
  border-color: rgb(var(--color-border-default-rgb) / 56%);
  background: rgb(var(--color-bg-elevated-rgb) / 84%);
  color: var(--color-text-secondary);
}

[data-theme="light"] .settings-dock {
  background: rgb(var(--color-bg-elevated-rgb) / 96%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  box-shadow: var(--surface-card-shadow);
}

[data-theme="light"] .settings-dock:hover {
  box-shadow: var(--shadow-md);
}

</style>
