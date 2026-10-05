import { useEffect, useRef, useState } from 'react'
import { MotionConfig } from 'motion/react'
import { Outlet, useLocation, useNavigation } from 'react-router'
import {
  mainLayoutGroupTitleMap,
  mainLayoutRouteTitleMap,
} from '@/config/mainLayoutShell'
import { translateWithFallback } from '@/i18n/formatMessage'
import { useMainLayoutShell } from '@/shell/hooks/useMainLayoutShell'
import { useShellT } from '@/shell/i18n'
import { restoreInnerScroll, saveInnerScroll } from '@/shell/innerScroll'
import { useRouteHandle } from '@/shell/routeHandle'
import { useShellPreferencesStore } from '@/shell/stores/shellPreferences'
import { initPerfTelemetry, recordRouteTiming } from '@/utils/perfTelemetry'
import { readPrefersReducedMotion } from '@/utils/reducedMotion'
import { ScrollToTopButton } from '@/ui/scroll-to-top-button'
import { BackendStatusBanner } from './BackendStatusBanner'
import { ErrorBoundary } from './ErrorBoundary'
import { MainLayoutSidebar, MainLayoutSkipLink, MainLayoutTopbar } from './MainLayoutChrome'

const MAIN_SCROLL_TOP_THRESHOLD = 480
let navStartMs: number | null = null
let navFrom = ''

const titleOf = (pathname: string): string => {
  if (pathname === '/') return 'dashboard'
  return pathname.replace(/^\//, '').split('/')[0] ?? 'dashboard'
}

export function MainLayout() {
  const t = useShellT()
  const location = useLocation()
  const navigation = useNavigation()
  const handle = useRouteHandle()
  const theme = useShellPreferencesStore((state) => state.theme)
  const effectiveTheme = useShellPreferencesStore((state) => state.effectiveTheme)
  const flavor = useShellPreferencesStore((state) => state.flavor)
  const locale = useShellPreferencesStore((state) => state.locale)
  const hasSidebar = !handle.hideSidebar
  const shell = useMainLayoutShell({
    hasSidebar,
    routeFullPath: `${location.pathname}${location.search}`,
    t,
  })
  const scrollRef = useRef<HTMLDivElement | null>(null)
  const [showScrollToTop, setShowScrollToTop] = useState(false)
  const prevPathRef = useRef(location.pathname)

  useEffect(() => {
    const previous = prevPathRef.current
    if (previous === location.pathname) return
    saveInnerScroll(previous, scrollRef.current?.scrollTop ?? 0)
    restoreInnerScroll({
      pathname: location.pathname,
      cache: handle.cache === true,
      element: scrollRef.current,
    })
    prevPathRef.current = location.pathname
  }, [location.pathname, handle.cache])

  useEffect(() => {
    if (!initPerfTelemetry()) return
    if (navigation.state === 'loading') {
      navStartMs = performance.now()
      navFrom = `${location.pathname}${location.search}`
      return
    }
    if (navStartMs == null) return
    recordRouteTiming(navFrom, `${location.pathname}${location.search}`, performance.now() - navStartMs)
    navStartMs = null
  }, [location.pathname, location.search, navigation.state])

  const routeId = titleOf(location.pathname)
  const currentPageTitle = t(mainLayoutRouteTitleMap[routeId] ?? 'nav.dashboard')
  const groupKey = handle.group ? mainLayoutGroupTitleMap[handle.group] : undefined
  const currentSectionTitle = t(groupKey || 'nav.dashboard')
  const currentLocaleLabel = locale === 'en-US' ? t('language.english') : t('language.chinese')
  const currentFlavorLabel = t(`settings.appearance.flavor.${flavor}`)
  const resolvedThemeLabel = t(`theme.${effectiveTheme}`)
  const currentThemeLabel =
    theme === 'system'
      ? translateWithFallback(t, 'settings.appearance.systemSummary', `${t('theme.system')} · {resolved}`, {
          resolved: resolvedThemeLabel,
        })
      : t(`theme.${theme}`)
  const shouldUseThemeStage = Boolean(handle.hideGlobalBackground)

  return (
    <MotionConfig reducedMotion="user">
      <div
        className={`layout-shell relative flex h-screen overflow-hidden font-sans text-text-primary ${
          shouldUseThemeStage ? 'layout-shell--theme-stage' : ''
        }`}
      >
        <MainLayoutSkipLink>{t('common.skipToContent') || 'Skip to content'}</MainLayoutSkipLink>
        {shell.showMobileBackdrop ? (
          <button
            type="button"
            className="layout-layer-modal-backdrop fixed inset-0 bg-black/55 lg:hidden"
            aria-label={shell.closeNavigationLabel}
            onClick={shell.closeSidebar}
          />
        ) : null}
        {hasSidebar ? (
          <MainLayoutSidebar
            t={t}
            isMobile={shell.isMobileSidebar}
            isOpen={shell.isSidebarOpen}
            isResizing={shell.isResizing}
            style={shell.sidebarShellStyle}
            onClose={shell.closeSidebar}
            onStartResize={shell.startResize}
            onResizeKeyDown={shell.handleResizeKeydown}
            isSettingsRoute={location.pathname === '/settings'}
            themeLabel={currentThemeLabel}
            flavorLabel={currentFlavorLabel}
            localeLabel={currentLocaleLabel}
          />
        ) : null}
        <main
          id="main-content"
          className={`content-main relative flex min-w-0 flex-1 flex-col overflow-hidden ${
            shouldUseThemeStage ? 'content-main--theme-stage' : ''
          }`}
        >
          <MainLayoutTopbar
            hasSidebar={hasSidebar}
            isMobile={shell.isMobileSidebar}
            isSidebarOpen={shell.isSidebarOpen}
            toggleLabel={shell.sidebarToggleLabel}
            onToggle={shell.toggleSidebar}
            hideSidebar={handle.hideSidebar}
            sectionTitle={currentSectionTitle}
            pageTitle={currentPageTitle}
            showEnvironment={shell.isTauri && !shell.isMobileSidebar}
          />
          <div
            ref={scrollRef}
            className={`content-scroll-area flex-1 overflow-y-auto scroll-smooth p-4 sm:p-6 ${
              shouldUseThemeStage ? 'content-scroll-area--theme-stage' : ''
            }`}
            onScroll={() => setShowScrollToTop((scrollRef.current?.scrollTop ?? 0) > MAIN_SCROLL_TOP_THRESHOLD)}
          >
            <BackendStatusBanner />
            <ErrorBoundary>
              {/* 全页 AnimatePresence 会在路由往返时保留已卸载树的 JS 快照。进出场只做 CSS enter。 */}
              <div key={location.pathname} className="route-page">
                {navigation.state === 'loading' ? (
                  <div className="flex min-h-[12.5rem] items-center justify-center">
                    <div className="loading-spinner h-8 w-8 border-accent-primary/30 border-t-accent-primary" />
                  </div>
                ) : (
                  <Outlet />
                )}
              </div>
            </ErrorBoundary>
          </div>
          <ScrollToTopButton
            visible={showScrollToTop}
            buttonLabel={t('common.backToTop')}
            label={t('common.topShort')}
            onClick={() => {
              scrollRef.current?.scrollTo({
                top: 0,
                behavior: readPrefersReducedMotion() ? 'auto' : 'smooth',
              })
            }}
          />
        </main>
      </div>
    </MotionConfig>
  )
}
