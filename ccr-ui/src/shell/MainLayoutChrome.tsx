import type { CSSProperties, ReactNode } from 'react'
import { NavLink } from 'react-router'
import { APP_NAME, APP_VERSION_LABEL } from '@/config/appMeta'
import { SIcon } from '@/ui/s-icon'
import type { ShellTranslate } from './i18n'
import { MainLayoutNav } from './MainLayoutNav'
import { EnvironmentSwitcher } from './EnvironmentSwitcher'

interface SidebarProps {
  t: ShellTranslate
  isMobile: boolean
  isOpen: boolean
  isResizing: boolean
  style?: CSSProperties
  onClose: () => void
  onStartResize: () => void
  onResizeKeyDown: (event: KeyboardEvent) => void
  isSettingsRoute: boolean
  themeLabel: string
  flavorLabel: string
  localeLabel: string
}

export function MainLayoutSidebar({
  t,
  isMobile,
  isOpen,
  isResizing,
  style,
  onClose,
  onStartResize,
  onResizeKeyDown,
  isSettingsRoute,
  themeLabel,
  flavorLabel,
  localeLabel,
}: SidebarProps) {
  return (
    <div
      id="primary-navigation-panel"
      className={`sidebar-glass layout-sidebar flex flex-col ${isResizing ? 'is-resizing select-none' : ''} ${
        isMobile
          ? 'layout-layer-modal fixed inset-y-0 left-0 w-[min(86vw,320px)] max-w-[320px] border-r border-border-default/20 shadow-2xl'
          : 'layout-layer-dropdown relative flex-shrink-0'
      } ${isMobile && !isOpen ? 'pointer-events-none -translate-x-full' : 'translate-x-0'}`}
      style={style}
    >
      {isMobile ? null : (
        <button
          type="button"
          className="layout-layer-popover group absolute -right-2 top-0 h-full w-5 cursor-col-resize rounded-full"
          aria-label={t('common.resizeSidebar')}
          onMouseDown={(event) => {
            event.preventDefault()
            onStartResize()
          }}
          onKeyDown={(event) => onResizeKeyDown(event.nativeEvent)}
        >
          <div className="absolute inset-y-0 right-1/2 w-px bg-border-default/70" />
        </button>
      )}
      <div className="flex h-[84px] shrink-0 items-center justify-between border-b border-border-default/45 px-4 pt-6">
        <div className="flex items-center gap-3">
          <img src="/icons/icon.svg" alt="CCR UI" className="h-10 w-10 rounded-[0.9rem] object-cover" />
          <div className="min-w-0">
            <h1 className="font-brand truncate text-[1.08rem] font-medium tracking-[-0.045em]">{APP_NAME}</h1>
            <p className="mt-1 text-[10px] font-semibold tracking-[0.18em] text-text-muted">
              {t('common.shell.tagline')}
            </p>
          </div>
        </div>
      </div>
      <MainLayoutNav t={t} onNavigate={isMobile ? onClose : undefined} />
      <div className="border-t border-border-default/40 p-3 pb-5">
        <NavLink
          to="/settings"
          data-testid="settings-dock-link"
          className={isSettingsRoute ? 'settings-dock settings-dock--active' : 'settings-dock'}
          aria-current={isSettingsRoute ? 'page' : undefined}
        >
          <span className="settings-dock__icon">
            <SIcon name="SlidersHorizontal" size="w-4 h-4" />
          </span>
          <span className="settings-dock__copy">
            <span className="settings-dock__title">{t('nav.settings')}</span>
            <span className="settings-dock__meta">
              <span>{themeLabel}</span>
              <span className="settings-dock__sep">·</span>
              <span>{flavorLabel}</span>
              <span className="settings-dock__sep">·</span>
              <span>{localeLabel}</span>
              <span className="settings-dock__sep">·</span>
              <span className="settings-dock__version">{APP_VERSION_LABEL}</span>
            </span>
          </span>
        </NavLink>
      </div>
    </div>
  )
}

interface TopbarProps {
  hasSidebar: boolean
  isMobile: boolean
  isSidebarOpen: boolean
  toggleLabel: string
  onToggle: () => void
  hideSidebar?: boolean
  sectionTitle: string
  pageTitle: string
  showEnvironment: boolean
}

export function MainLayoutTopbar({
  hasSidebar,
  isMobile,
  isSidebarOpen,
  toggleLabel,
  onToggle,
  hideSidebar,
  sectionTitle,
  pageTitle,
  showEnvironment,
}: TopbarProps) {
  return (
    <div className="topbar-glass layout-layer-sticky sticky top-0 flex min-h-[78px] shrink-0 items-center justify-between border-b border-border-default/40 px-4 pt-5 sm:px-6 sm:pt-6">
      <div className="flex min-w-0 items-center gap-3 text-sm text-text-secondary">
        {hasSidebar && isMobile ? (
          <button
            type="button"
            className="inline-flex h-11 w-11 items-center justify-center rounded-2xl border border-border-default/70 bg-bg-surface lg:hidden"
            aria-expanded={isSidebarOpen}
            aria-label={toggleLabel}
            onClick={onToggle}
          >
            <SIcon name={isSidebarOpen ? 'X' : 'Menu'} size="w-5 h-5" />
          </button>
        ) : null}
        <TitleTrail hideSidebar={hideSidebar} sectionTitle={sectionTitle} pageTitle={pageTitle} />
      </div>
      <div className="ml-4 flex items-center gap-2 sm:gap-4">
        {showEnvironment ? <EnvironmentSwitcher /> : null}
      </div>
    </div>
  )
}

function TitleTrail({
  hideSidebar,
  sectionTitle,
  pageTitle,
}: {
  hideSidebar?: boolean
  sectionTitle: string
  pageTitle: string
}) {
  if (hideSidebar) {
    return <span className="font-semibold text-text-primary">{pageTitle}</span>
  }
  return (
    <>
      <span className="truncate opacity-50">{sectionTitle}</span>
      {sectionTitle !== pageTitle ? (
        <>
          <span className="mx-2 opacity-30">/</span>
          <span className="truncate font-medium text-text-primary">{pageTitle}</span>
        </>
      ) : null}
    </>
  )
}

export function MainLayoutSkipLink({ children }: { children: ReactNode }) {
  return (
    <a href="#main-content" className="skip-to-content layout-layer-toast">
      {children}
    </a>
  )
}
