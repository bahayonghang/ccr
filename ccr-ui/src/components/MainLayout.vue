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
      class="fixed inset-0 layout-layer-modal-backdrop bg-black/55 lg:hidden"
      :aria-label="closeNavigationLabel"
      @click="closeSidebar"
    />

    <div
      v-if="hasSidebar"
      id="primary-navigation-panel"
      class="sidebar-glass layout-sidebar flex flex-col transition-[width,transform,background-color,border-color,box-shadow] duration-300 ease-out will-change-[transform]"
      :class="[
        isResizing ? 'select-none is-resizing' : '',
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
          class="inline-flex h-11 w-11 flex-none items-center justify-center rounded-2xl border border-border-default/70 bg-bg-surface text-text-primary shadow-sm transition-colors hover:border-accent-primary/30 hover:bg-bg-elevated/90 lg:hidden"
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
            class="settings-dock group"
            :class="{ 'settings-dock--active': isSettingsRoute }"
            data-testid="settings-dock-link"
            :aria-current="isSettingsRoute ? 'page' : undefined"
            @click="navigate"
          >
            <span class="settings-dock__icon">
              <SIcon
                name="SlidersHorizontal"
                size="w-4 h-4"
              />
            </span>
            <span class="settings-dock__copy">
              <span class="settings-dock__title">{{ t('nav.settings') }}</span>
              <span class="settings-dock__meta">
                <span>{{ currentThemeLabel }}</span>
                <span
                  class="settings-dock__sep"
                  aria-hidden="true"
                >·</span>
                <span>{{ currentFlavorLabel }}</span>
                <span
                  class="settings-dock__sep"
                  aria-hidden="true"
                >·</span>
                <span>{{ currentLocaleLabel }}</span>
                <span
                  class="settings-dock__sep"
                  aria-hidden="true"
                >·</span>
                <span class="settings-dock__version">{{ appVersionLabel }}</span>
              </span>
            </span>
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
            class="inline-flex h-11 w-11 flex-none items-center justify-center rounded-2xl border border-border-default/70 bg-bg-surface text-text-primary shadow-sm transition-colors hover:border-accent-primary/30 hover:bg-bg-elevated/90 lg:hidden"
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
const { theme, effectiveTheme, flavor, locale } = storeToRefs(shellPreferencesStore)

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
// dock 摘要的 flavor 显示名映射（neutral | clay）。
const currentFlavorLabel = computed(() => t(`settings.appearance.flavor.${flavor.value}`))
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
  background: var(--color-bg-base);
}

/* 侧栏 / 顶栏：不透明 chrome，无内高光、无装饰玻璃 */
:root[data-theme="dark"] .sidebar-glass,
.sidebar-glass {
  background: var(--surface-shell-bg);
  backdrop-filter: var(--surface-shell-blur);
  border-right: 1px solid var(--surface-shell-border);
  box-shadow: var(--surface-shell-shadow);
}

/* 拖拽 resize 全程重模糊会拖累合成帧;拖拽期间临时降级为不透明 */
.sidebar-glass.is-resizing {
  backdrop-filter: none;
}

.topbar-glass {
  background: var(--surface-shell-bg);
  backdrop-filter: var(--surface-shell-blur);
  box-shadow: var(--surface-shell-shadow);
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
  background: var(--color-bg-base);
}

.content-scroll-area--theme-stage {
  background: transparent;
}

/* 导航激活态：tonal 底 + 主文本 + 细描边，无左侧竖条、无渐变 */
.nav-item {
  @apply relative flex items-center gap-3 px-3 py-2.5 text-sm font-medium text-text-secondary
         transition-interactive duration-200;

  border: 1px solid transparent;
  border-radius: var(--radius-lg);
  background: transparent;
}

.nav-item:focus-visible {
  @apply outline-none ring-2 ring-accent-primary/25 ring-offset-2 ring-offset-bg-base;
}

.nav-item:hover {
  @apply text-text-primary;

  background: var(--color-bg-overlay);
  border-color: var(--color-border-subtle);
}

.nav-item.router-link-active:not(.nav-item--root),
.nav-item.router-link-exact-active.nav-item--root {
  @apply text-text-primary;

  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
}

.settings-dock {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--surface-card-border);
  border-radius: var(--radius-lg);
  background: var(--surface-card-bg);
  box-shadow: var(--surface-card-shadow);
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.settings-dock:hover {
  border-color: var(--color-border-strong);
  background: var(--color-bg-surface);
}

.settings-dock--active {
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
}

.settings-dock__icon {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
}

.settings-dock--active .settings-dock__icon {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: var(--color-text-primary);
}

.settings-dock__copy {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 0.125rem;
}

.settings-dock__title {
  font-size: 0.875rem;
  font-weight: 600;
  line-height: 1.3;
  color: var(--color-text-primary);
}

.settings-dock__meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.15rem 0.2rem;
  font-size: 0.6875rem;
  font-weight: 500;
  line-height: 1.24;
  color: var(--color-text-secondary);
}

.settings-dock__sep {
  color: var(--color-text-muted);
}

.settings-dock__version {
  font-family: var(--font-mono);
}

</style>
