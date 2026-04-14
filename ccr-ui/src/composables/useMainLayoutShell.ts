import { computed, onMounted, onUnmounted, ref, watch, type ComputedRef } from 'vue'
import { storeToRefs } from 'pinia'
import { isTauriEnvironment } from '@/api/runtime/environment'
import { useShellPreferencesStore } from '@/stores/shellPreferences'

interface UseMainLayoutShellOptions {
  hasSidebar: ComputedRef<boolean>
  routeFullPath: ComputedRef<string>
  t: (key: string) => string
}

export function useMainLayoutShell({ hasSidebar, routeFullPath, t }: UseMainLayoutShellOptions) {
  const shellPreferencesStore = useShellPreferencesStore()
  const { sidebarWidth } = storeToRefs(shellPreferencesStore)
  const isResizing = ref(false)
  const isMobileSidebar = ref(false)
  const isSidebarOpen = ref(false)
  const minWidth = 200
  const maxWidth = 480
  const isTauri = ref(false)
  const closeNavigationLabel = computed(() => t('common.closeNavigation'))
  const openNavigationLabel = computed(() => t('common.openNavigation'))
  const sidebarToggleLabel = computed(() => (
    isSidebarOpen.value ? closeNavigationLabel.value : openNavigationLabel.value
  ))
  const showMobileBackdrop = computed(() => (
    hasSidebar.value && isMobileSidebar.value && isSidebarOpen.value
  ))
  const sidebarShellStyle = computed(() => (
    isMobileSidebar.value
      ? undefined
      : { width: `${sidebarWidth.value}px` }
  ))

  let mobileMediaQuery: MediaQueryList | null = null

  const closeSidebar = () => {
    isSidebarOpen.value = false
  }

  const toggleSidebar = () => {
    isSidebarOpen.value = !isSidebarOpen.value
  }

  const handleViewportChange = (matches: boolean) => {
    isMobileSidebar.value = matches
    if (!matches) {
      isSidebarOpen.value = false
      isResizing.value = false
    }
  }

  const handleMobileMediaChange = (event: MediaQueryListEvent) => {
    handleViewportChange(event.matches)
  }

  const handleEscapeKey = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && isMobileSidebar.value && isSidebarOpen.value) {
      closeSidebar()
    }
  }

  const handleResize = (event: MouseEvent) => {
    if (!isResizing.value) return

    let nextWidth = event.clientX
    if (nextWidth < minWidth) nextWidth = minWidth
    if (nextWidth > maxWidth) nextWidth = maxWidth
    shellPreferencesStore.updateSidebarWidth(nextWidth, false)
  }

  const stopResize = () => {
    isResizing.value = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    shellPreferencesStore.commitSidebarWidth()
    window.removeEventListener('mousemove', handleResize)
    window.removeEventListener('mouseup', stopResize)
  }

  const startResize = () => {
    if (isMobileSidebar.value) return

    isResizing.value = true
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('mousemove', handleResize)
    window.addEventListener('mouseup', stopResize)
  }

  const handleResizeKeydown = (event: KeyboardEvent) => {
    if (isMobileSidebar.value) return

    const step = event.shiftKey ? 32 : 16
    if (event.key === 'ArrowLeft') {
      event.preventDefault()
      shellPreferencesStore.updateSidebarWidth(Math.max(minWidth, sidebarWidth.value - step))
    } else if (event.key === 'ArrowRight') {
      event.preventDefault()
      shellPreferencesStore.updateSidebarWidth(Math.min(maxWidth, sidebarWidth.value + step))
    } else if (event.key === 'Home') {
      event.preventDefault()
      shellPreferencesStore.updateSidebarWidth(minWidth)
    } else if (event.key === 'End') {
      event.preventDefault()
      shellPreferencesStore.updateSidebarWidth(maxWidth)
    } else {
      return
    }
  }

  onMounted(async () => {
    mobileMediaQuery = window.matchMedia('(max-width: 1023px)')
    handleViewportChange(mobileMediaQuery.matches)
    mobileMediaQuery.addEventListener('change', handleMobileMediaChange)
    window.addEventListener('keydown', handleEscapeKey)

    isTauri.value = isTauriEnvironment()
    await shellPreferencesStore.hydrateRuntimePreferences()
  })

  onUnmounted(() => {
    window.removeEventListener('mousemove', handleResize)
    window.removeEventListener('mouseup', stopResize)
    window.removeEventListener('keydown', handleEscapeKey)
    mobileMediaQuery?.removeEventListener('change', handleMobileMediaChange)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    document.body.style.overflow = ''
  })

  watch(routeFullPath, () => {
    closeSidebar()
  })

  watch(hasSidebar, (value) => {
    if (!value) {
      closeSidebar()
    }
  })

  watch([isMobileSidebar, isSidebarOpen], ([mobile, open]) => {
    document.body.style.overflow = mobile && open ? 'hidden' : ''
  })

  return {
    closeNavigationLabel,
    closeSidebar,
    handleResizeKeydown,
    isMobileSidebar,
    isResizing,
    isSidebarOpen,
    isTauri,
    openNavigationLabel,
    showMobileBackdrop,
    sidebarShellStyle,
    sidebarToggleLabel,
    sidebarWidth,
    startResize,
    toggleSidebar,
  }
}
