import { computed, onMounted, onUnmounted, ref, watch, type ComputedRef } from 'vue'
import { getSkipExitConfirm, isTauriEnvironment, setSkipExitConfirm } from '@/api/tauri'
import { logger } from '@/utils/logger'

interface UseMainLayoutShellOptions {
  hasSidebar: ComputedRef<boolean>
  routeFullPath: ComputedRef<string>
  t: (key: string) => string
}

export function useMainLayoutShell({ hasSidebar, routeFullPath, t }: UseMainLayoutShellOptions) {
  const sidebarWidth = ref(240)
  const isResizing = ref(false)
  const isMobileSidebar = ref(false)
  const isSidebarOpen = ref(false)
  const minWidth = 200
  const maxWidth = 480
  const isTauri = ref(false)
  const showExitConfirm = ref(true)
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

  const toggleExitConfirm = async () => {
    showExitConfirm.value = !showExitConfirm.value
    if (isTauri.value) {
      await setSkipExitConfirm(!showExitConfirm.value)
    }
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
    sidebarWidth.value = nextWidth
  }

  const stopResize = () => {
    isResizing.value = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    localStorage.setItem('ccr-sidebar-width', sidebarWidth.value.toString())
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
    if (savedWidth) {
      sidebarWidth.value = Number(savedWidth) || 240
    }

    mobileMediaQuery = window.matchMedia('(max-width: 1023px)')
    handleViewportChange(mobileMediaQuery.matches)
    mobileMediaQuery.addEventListener('change', handleMobileMediaChange)
    window.addEventListener('keydown', handleEscapeKey)

    isTauri.value = isTauriEnvironment()
    if (isTauri.value) {
      try {
        const skipConfirm = await getSkipExitConfirm()
        showExitConfirm.value = !skipConfirm
      } catch (error) {
        logger.error('Failed to load exit confirmation preference', error)
      }
    }
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
    showExitConfirm,
    showMobileBackdrop,
    sidebarShellStyle,
    sidebarToggleLabel,
    sidebarWidth,
    startResize,
    toggleExitConfirm,
    toggleSidebar,
  }
}
