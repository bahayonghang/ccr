import { useCallback, useEffect, useRef, useState } from 'react'
import { isTauriEnvironment } from '@/api/runtime/environment'
import { useShellPreferencesStore } from '@/shell/stores/shellPreferences'

// MainLayout 外壳瞬态（08-22-state-logic-port 批次 5；原 composable 语义等价迁移）。
// isSidebarOpen / isResizing / isMobileSidebar 为壳层瞬态 → useState；
// sidebarWidth 委派 shellPreferences（Zustand）。三个 watch 分别映射为 effect。

interface UseMainLayoutShellOptions {
  hasSidebar: boolean
  routeFullPath: string
  t: (key: string) => string
}

const MIN_WIDTH = 200
const MAX_WIDTH = 480

/** 键盘调宽的步进目标（原 handleResizeKeydown 的分支表）。 */
function keyboardWidthTarget(key: string, shiftKey: boolean, current: number): number | null {
  const step = shiftKey ? 32 : 16
  if (key === 'ArrowLeft') return Math.max(MIN_WIDTH, current - step)
  if (key === 'ArrowRight') return Math.min(MAX_WIDTH, current + step)
  if (key === 'Home') return MIN_WIDTH
  if (key === 'End') return MAX_WIDTH
  return null
}

export function useMainLayoutShell({ hasSidebar, routeFullPath, t }: UseMainLayoutShellOptions) {
  const sidebarWidth = useShellPreferencesStore((state) => state.sidebarWidth)
  const updateSidebarWidth = useShellPreferencesStore((state) => state.updateSidebarWidth)
  const commitSidebarWidth = useShellPreferencesStore((state) => state.commitSidebarWidth)
  const hydrateRuntimePreferences = useShellPreferencesStore((state) => state.hydrateRuntimePreferences)

  const [isResizing, setIsResizing] = useState(false)
  const [isMobileSidebar, setIsMobileSidebar] = useState(false)
  const [isSidebarOpen, setIsSidebarOpen] = useState(false)
  const [isTauri, setIsTauri] = useState(false)
  const isResizingRef = useRef(false)
  // 挂载期监听器读取最新瞬态用的镜像（监听只建一次，回调经 ref 惰性读值）。
  const mobileSidebarRef = useRef(false)
  const sidebarOpenRef = useRef(false)

  useEffect(() => {
    mobileSidebarRef.current = isMobileSidebar
  }, [isMobileSidebar])
  useEffect(() => {
    sidebarOpenRef.current = isSidebarOpen
  }, [isSidebarOpen])

  const closeSidebar = useCallback(() => {
    setIsSidebarOpen(false)
  }, [])

  const toggleSidebar = useCallback(() => {
    setIsSidebarOpen((open) => !open)
  }, [])

  const handleViewportChange = useCallback((matches: boolean) => {
    setIsMobileSidebar(matches)
    if (!matches) {
      setIsSidebarOpen(false)
      setIsResizing(false)
      isResizingRef.current = false
    }
  }, [])

  // —— 拖拽调宽：mousemove/up 监听随 start/stop 成对挂载解绑 ——
  const handleResize = useCallback(
    (event: MouseEvent) => {
      if (!isResizingRef.current) return
      updateSidebarWidth(
        Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, event.clientX)),
        false,
      )
    },
    [updateSidebarWidth],
  )

  const stopResize = useCallback(() => {
    isResizingRef.current = false
    setIsResizing(false)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    commitSidebarWidth()
    window.removeEventListener('mousemove', handleResize)
    window.removeEventListener('mouseup', stopResize)
  }, [commitSidebarWidth, handleResize])

  const startResize = useCallback(() => {
    if (isMobileSidebar) return
    isResizingRef.current = true
    setIsResizing(true)
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
    window.addEventListener('mousemove', handleResize)
    window.addEventListener('mouseup', stopResize)
  }, [handleResize, isMobileSidebar, stopResize])

  const handleResizeKeydown = useCallback(
    (event: KeyboardEvent) => {
      if (isMobileSidebar) return
      const target = keyboardWidthTarget(event.key, event.shiftKey, sidebarWidth)
      if (target !== null) {
        event.preventDefault()
        updateSidebarWidth(target)
      }
    },
    [isMobileSidebar, sidebarWidth, updateSidebarWidth],
  )

  // —— 挂载：移动断点 / Esc / runtime 偏好水合（原 onMounted + onUnmounted）。
  // 依赖均为稳定引用（useCallback/[] 或 Zustand 模块级 action），effect 只执行一次；
  // Esc 判定读 ref 镜像取最新瞬态。
  useEffect(() => {
    const mobileMediaQuery =
      typeof window.matchMedia === 'function' ? window.matchMedia('(max-width: 1023px)') : null
    if (mobileMediaQuery) handleViewportChange(mobileMediaQuery.matches)
    const handleMobileMediaChange = (event: MediaQueryListEvent) =>
      handleViewportChange(event.matches)
    const handleEscapeKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && mobileSidebarRef.current && sidebarOpenRef.current) {
        closeSidebar()
      }
    }
    mobileMediaQuery?.addEventListener('change', handleMobileMediaChange)
    window.addEventListener('keydown', handleEscapeKey)

    setIsTauri(isTauriEnvironment())
    void hydrateRuntimePreferences()

    return () => {
      window.removeEventListener('mousemove', handleResize)
      window.removeEventListener('mouseup', stopResize)
      window.removeEventListener('keydown', handleEscapeKey)
      mobileMediaQuery?.removeEventListener('change', handleMobileMediaChange)
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
      document.body.style.overflow = ''
    }
  }, [closeSidebar, handleResize, handleViewportChange, hydrateRuntimePreferences, stopResize])

  // 原 watch(routeFullPath)：路由切换收起移动侧栏。
  useEffect(() => {
    closeSidebar()
  }, [routeFullPath, closeSidebar])

  // 原 watch(hasSidebar)：无侧栏布局时收起。
  useEffect(() => {
    if (!hasSidebar) closeSidebar()
  }, [hasSidebar, closeSidebar])

  // 原 watch([isMobileSidebar, isSidebarOpen])：移动侧栏展开时锁 body 滚动。
  useEffect(() => {
    document.body.style.overflow = isMobileSidebar && isSidebarOpen ? 'hidden' : ''
  }, [isMobileSidebar, isSidebarOpen])

  const closeNavigationLabel = t('common.closeNavigation')
  const openNavigationLabel = t('common.openNavigation')
  const sidebarToggleLabel = isSidebarOpen ? closeNavigationLabel : openNavigationLabel
  const showMobileBackdrop = hasSidebar && isMobileSidebar && isSidebarOpen
  const sidebarShellStyle = isMobileSidebar ? undefined : { width: `${sidebarWidth}px` }

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
