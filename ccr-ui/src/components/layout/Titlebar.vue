<template>
  <div
    class="titlebar-shell fixed top-0 left-0 right-0 flex h-9 items-center justify-between border-b border-border-default/30 px-3 text-text-primary select-none"
  >
    <!-- Left: App Icon and Menu -->
    <div class="flex items-center space-x-1">
      <div
        data-tauri-drag-region
        class="titlebar-drag-region flex items-center"
      >
        <!-- App Icon -->
        <div
          class="titlebar-interactive w-5 h-5 rounded-md flex items-center justify-center mr-2 shadow-sm relative overflow-hidden group cursor-pointer"
          @click="showAboutDialog = true"
        >
          <img
            :src="appIconUrl"
            class="w-full h-full object-cover transition-transform group-hover:scale-110"
            :alt="appName"
          >
        </div>
      </div>

      <!-- Simple Menu Item as Example -->
      <div
        ref="menuRef"
        class="titlebar-interactive relative"
      >
        <button
          class="titlebar-menu-btn"
          :class="{ 'bg-bg-overlay/70 text-text-primary': isMenuOpen }"
          @click="toggleMenu"
        >
          {{ menuLabel }}
        </button>

        <!-- Dropdown -->
        <div
          v-if="isMenuOpen"
          class="titlebar-menu absolute top-full left-0 mt-1 w-48 overflow-hidden rounded-lg py-1"
        >
          <button
            class="w-full text-left px-3 py-1.5 text-xs text-text-secondary hover:text-text-primary hover:bg-bg-overlay/70 transition-colors flex items-center"
            @click="openAbout"
          >
            <i class="i-carbon-information mr-2" /> {{ t('common.about.menu', { name: appName }) }}
          </button>
          <div class="my-1 h-px bg-border-default/40" />
          <button
            class="w-full text-left px-3 py-1.5 text-xs text-danger hover:text-danger hover:bg-danger/10 transition-colors flex items-center"
            @click="closeWindow"
          >
            <i class="i-carbon-close mr-2" /> {{ quitLabel }}
          </button>
        </div>
      </div>
    </div>

    <!-- Center: Window Title -->
    <div
      data-tauri-drag-region
      class="titlebar-drag-region titlebar-title absolute left-1/2 -translate-x-1/2 flex items-center space-x-2 text-xs font-medium tracking-wider"
    >
      <span class="opacity-50">·</span>
      <span>{{ windowTitle }}</span>
      <span class="opacity-50">·</span>
    </div>

    <!-- Right: Window Controls -->
    <div class="titlebar-interactive flex items-center space-x-0.5">
      <button
        type="button"
        class="titlebar-control-btn group"
        title="最小化"
        @click="minimizeWindow"
      >
        <svg
          class="titlebar-control-icon w-3.5 h-3.5"
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
        type="button"
        class="titlebar-control-btn group"
        :title="isMaximized ? '还原' : '最大化'"
        @click="toggleMaximize"
      >
        <svg
          v-if="!isMaximized"
          class="titlebar-control-icon w-3.5 h-3.5"
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
          class="titlebar-control-icon w-3.5 h-3.5"
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
        type="button"
        class="titlebar-control-btn titlebar-control-btn--close group"
        title="关闭"
        @click="closeWindow"
      >
        <svg
          class="titlebar-control-icon w-3.5 h-3.5"
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
          class="titlebar-dialog-backdrop fixed inset-0 flex items-center justify-center p-4"
          @click.self="showAboutDialog = false"
        >
          <div class="surface-modal relative w-full max-w-sm overflow-hidden overflow-y-auto rounded-2xl border border-border-default/70">
            <div class="p-6 flex flex-col items-center">
              <div class="w-24 h-24 rounded-2xl mb-4 relative overflow-hidden border border-border-default/30">
                <img
                  :src="appLogoUrl"
                  :alt="`${appName} logo`"
                  class="w-full h-full object-cover"
                >
              </div>
              
              <h2 class="text-2xl font-bold text-text-primary tracking-tight mb-1">
                {{ appName }}
              </h2>
              <div class="flex items-center space-x-2 text-xs mb-4">
                <span class="rounded-full border border-accent-primary/20 bg-accent-primary/10 px-2 py-0.5 text-accent-primary">
                  {{ appTagline }}
                </span>
                <span class="text-text-muted">{{ appVersionText }}</span>
              </div>
              
              <p class="mb-6 text-center text-sm leading-relaxed text-text-secondary">
                {{ t('common.about.description') }}
              </p>
              
              <div class="w-full space-y-2 mb-6">
                <div class="surface-status flex items-center justify-between rounded-lg border border-border-default/60 p-2 text-xs">
                  <span class="text-text-muted">{{ t('common.about.owner') }}</span>
                  <span class="font-medium text-text-primary">{{ appOwner }}</span>
                </div>
                <div class="surface-status flex items-center justify-between rounded-lg border border-border-default/60 p-2 text-xs">
                  <span class="text-text-muted">{{ t('common.about.engine') }}</span>
                  <span class="font-medium text-text-primary">{{ appEngine }}</span>
                </div>
              </div>

              <button 
                class="w-full py-2 border border-border-default/60 rounded-xl text-sm font-medium text-text-primary transition-[color,background-color,transform] transform hover:scale-[1.01] active:scale-95 flex items-center justify-center focus:outline-none bg-bg-surface hover:bg-bg-elevated"
                @click="showAboutDialog = false"
              >
                {{ t('common.about.close') }}
              </button>
            </div>
            
            <button
              class="absolute top-3 right-3 rounded-full p-1.5 text-text-muted transition-colors hover:bg-bg-overlay/70 hover:text-text-primary"
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
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { APP_ENGINE, APP_NAME, APP_OWNER, APP_TAGLINE, APP_VERSION } from '@/config/appMeta'
import { logger } from '@/utils/logger'
import { getCurrentWindowSafe } from '@/utils/tauriWindow'

const { t, locale } = useI18n()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)

const isMaximized = ref(false)
const isFocused = ref(true)
const isMenuOpen = ref(false)
const showAboutDialog = ref(false)
const menuRef = ref<HTMLElement | null>(null)
const appName = APP_NAME
const appTagline = APP_TAGLINE
const appOwner = APP_OWNER
const appEngine = APP_ENGINE
const appIconUrl = '/icons/icon.svg'
const appLogoUrl = '/icons/logo.svg'
const windowTitle = computed(() => appName.toUpperCase())
const appVersionText = computed(() => `v${APP_VERSION}`)
const menuLabel = computed(() => tt('文件', 'File'))
const quitLabel = computed(() => tt('离开系统', 'Quit'))

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
.titlebar-drag-region {
  app-region: drag;
}

.titlebar-interactive,
.titlebar-interactive *,
.titlebar-menu-btn,
.titlebar-control-btn,
.titlebar-menu,
.titlebar-menu * {
  app-region: no-drag;
}

.titlebar-shell {
  z-index: var(--layer-sticky);
  background: var(--surface-shell-bg);
  backdrop-filter: var(--surface-shell-blur);
  box-shadow: var(--surface-shell-shadow);
  transition: background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.titlebar-menu {
  z-index: var(--layer-dropdown);
  background: var(--surface-modal-bg);
  backdrop-filter: var(--surface-modal-blur);
  border: 1px solid var(--surface-modal-border);
  box-shadow: var(--surface-modal-shadow);
}

.titlebar-dialog-backdrop {
  z-index: var(--layer-modal);
  background: var(--surface-modal-backdrop);
  backdrop-filter: var(--surface-modal-blur);
}

.titlebar-control-btn {
  width: 46px;
  height: 2.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.75rem;
  color: var(--color-text-secondary);
  transition:
    color var(--motion-feedback-duration) var(--motion-standard-ease),
    background-color var(--motion-feedback-duration) var(--motion-standard-ease),
    box-shadow var(--motion-feedback-duration) var(--motion-standard-ease),
    transform var(--motion-feedback-duration) var(--motion-standard-ease);
  cursor: default;
  outline: none;

  &:hover {
    color: var(--color-text-primary);
    background: rgb(var(--color-bg-overlay-rgb) / 55%);
    box-shadow: inset 0 0 0 1px var(--color-border-subtle);
  }

  &:active {
    transform: translateY(0.5px);
  }

  &:focus-visible {
    outline: 2px solid rgb(var(--color-accent-primary-rgb) / 50%);
    outline-offset: 2px;
  }
}

.titlebar-control-btn--close:hover {
  color: var(--color-danger-contrast);
  background: rgb(var(--color-danger-rgb) / 92%);
  box-shadow: none;
}

.titlebar-control-btn--close:focus-visible {
  outline-color: rgb(var(--color-danger-rgb) / 42%);
}

.titlebar-control-icon {
  display: block;
  transition: color var(--motion-feedback-duration) var(--motion-standard-ease);
}

.titlebar-title {
  color: var(--color-text-muted);
}

.titlebar-menu-btn {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
  color: var(--color-text-muted);
  border-radius: 0.375rem;
  transition: color 150ms, background-color 150ms;
  outline: none;

  &:focus-visible {
    outline: 2px solid rgb(var(--color-accent-primary-rgb) / 50%);
    outline-offset: 2px;
  }
}

.titlebar-menu-btn:hover {
  color: var(--color-text-primary);
  background-color: rgb(var(--color-bg-overlay-rgb) / 72%);
}
</style>
