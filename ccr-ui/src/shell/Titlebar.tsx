import { lazy, Suspense, useCallback, useEffect, useRef, useState } from 'react'
import { APP_ENGINE, APP_NAME, APP_OWNER, APP_TAGLINE, APP_VERSION } from '@/config/appMeta'
import { ErrorBoundary } from '@/shell/ErrorBoundary'
import { useShellT } from '@/shell/i18n'
import { useShellPreferencesStore } from '@/shell/stores/shellPreferences'
import { logger } from '@/utils/logger'
import { getCurrentWindowSafe } from '@/utils/tauriWindow'

const LazyBaseModal = lazy(() =>
  import('@/ui/base-modal').then((mod) => ({ default: mod.BaseModal })),
)

interface AboutDialogFailureFallbackProps {
  onFailure: () => void
}

function AboutDialogFailureFallback({ onFailure }: AboutDialogFailureFallbackProps) {
  useEffect(() => {
    onFailure()
  }, [onFailure])

  return null
}

export function Titlebar() {
  const t = useShellT()
  const locale = useShellPreferencesStore((state) => state.locale)
  const isZh = locale.startsWith('zh')
  const [isMaximized, setIsMaximized] = useState(false)
  const [isMenuOpen, setIsMenuOpen] = useState(false)
  const [showAbout, setShowAbout] = useState(false)
  const [hasRequestedAbout, setHasRequestedAbout] = useState(false)
  const [aboutLoadFailed, setAboutLoadFailed] = useState(false)
  const menuRef = useRef<HTMLDivElement | null>(null)

  const openAbout = () => {
    if (aboutLoadFailed) return
    setHasRequestedAbout(true)
    setShowAbout(true)
  }

  const containAboutLoadFailure = useCallback(() => {
    setShowAbout(false)
    setHasRequestedAbout(false)
    setAboutLoadFailed(true)
  }, [])

  useEffect(() => {
    const onClick = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsMenuOpen(false)
      }
    }
    document.addEventListener('click', onClick)
    let stopResize: (() => void) | undefined
    let stopFocus: (() => void) | undefined
    void getCurrentWindowSafe().then(async (win) => {
      if (!win) return
      setIsMaximized(await win.isMaximized())
      stopResize = await win.onResized(async () => {
        setIsMaximized(await win.isMaximized())
      })
      stopFocus = await win.onFocusChanged(() => undefined)
    }).catch((error) => {
      logger.debug('[Titlebar] skip tauri listeners in browser runtime', error)
    })
    return () => {
      document.removeEventListener('click', onClick)
      stopResize?.()
      stopFocus?.()
    }
  }, [])

  const runWindow = async (action: 'minimize' | 'toggle' | 'close') => {
    const win = await getCurrentWindowSafe()
    if (!win) {
      if (action === 'toggle') setIsMaximized((value) => !value)
      return
    }
    if (action === 'minimize') {
      await win.minimize()
      return
    }
    if (action === 'close') {
      await win.close()
      return
    }
    if (await win.isMaximized()) {
      await win.unmaximize()
      setIsMaximized(false)
      return
    }
    await win.maximize()
    setIsMaximized(true)
  }

  return (
    <div className="titlebar-shell fixed top-0 left-0 right-0 flex h-9 items-center justify-between border-b border-border-default/30 px-3 text-text-primary select-none">
      <div className="flex items-center space-x-1">
        <div data-tauri-drag-region className="titlebar-drag-region flex items-center">
          <button
            type="button"
            className="titlebar-interactive mr-2 flex h-5 w-5 items-center justify-center overflow-hidden rounded-md shadow-sm"
            onClick={openAbout}
            disabled={aboutLoadFailed}
          >
            <img src="/icons/icon.svg" className="h-full w-full object-cover" alt={APP_NAME} />
          </button>
        </div>
        <div ref={menuRef} className="titlebar-interactive relative">
          <button
            type="button"
            className="titlebar-menu-btn"
            onClick={() => setIsMenuOpen((open) => !open)}
          >
            {isZh ? '文件' : 'File'}
          </button>
          {isMenuOpen ? (
            <div className="titlebar-menu absolute top-full left-0 mt-1 w-48 overflow-hidden rounded-lg py-1">
              <button
                type="button"
                className="flex w-full items-center px-3 py-1.5 text-left text-xs text-text-secondary hover:bg-bg-overlay/70 hover:text-text-primary"
                disabled={aboutLoadFailed}
                onClick={() => {
                  setIsMenuOpen(false)
                  openAbout()
                }}
              >
                {t('common.about.menu', { name: APP_NAME })}
              </button>
              <div className="my-1 h-px bg-border-default/40" />
              <button
                type="button"
                className="flex w-full items-center px-3 py-1.5 text-left text-xs text-danger hover:bg-danger/10"
                onClick={() => void runWindow('close')}
              >
                {isZh ? '离开系统' : 'Quit'}
              </button>
            </div>
          ) : null}
        </div>
      </div>
      <div
        data-tauri-drag-region
        className="titlebar-drag-region titlebar-title absolute left-1/2 flex -translate-x-1/2 items-center space-x-2 text-xs font-medium tracking-wider"
      >
        <span className="opacity-50">·</span>
        <span>{APP_NAME.toUpperCase()}</span>
        <span className="opacity-50">·</span>
      </div>
      <div className="titlebar-interactive flex items-center space-x-0.5">
        <button type="button" className="titlebar-control-btn" title="最小化" onClick={() => void runWindow('minimize')}>
          <svg className="titlebar-control-icon h-3.5 w-3.5" fill="currentColor" viewBox="0 0 16 16">
            <rect x="3" y="8" width="10" height="1" rx="0.5" />
          </svg>
        </button>
        <button
          type="button"
          className="titlebar-control-btn"
          title={isMaximized ? '还原' : '最大化'}
          onClick={() => void runWindow('toggle')}
        >
          <svg className="titlebar-control-icon h-3.5 w-3.5" fill="none" viewBox="0 0 16 16">
            {isMaximized ? (
              <path
                d="M5.5 5.5v-2a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1h-2"
                stroke="currentColor"
                strokeWidth="1.2"
              />
            ) : (
              <rect x="3.5" y="3.5" width="9" height="9" rx="1" stroke="currentColor" strokeWidth="1.2" />
            )}
          </svg>
        </button>
        <button
          type="button"
          className="titlebar-control-btn titlebar-control-btn--close"
          title="关闭"
          onClick={() => void runWindow('close')}
        >
          <svg className="titlebar-control-icon h-3.5 w-3.5" fill="currentColor" viewBox="0 0 16 16">
            <path d="M4.146 4.146a.5.5 0 0 0 0 .708L7.293 8l-3.147 3.146a.5.5 0 0 0 .708.708L8 8.707l3.146 3.147a.5.5 0 0 0 .708-.708L8.707 8l3.147-3.146a.5.5 0 0 0-.708-.708L8 7.293 4.854 4.146a.5.5 0 0 0-.708 0z" />
          </svg>
        </button>
      </div>
      {hasRequestedAbout ? (
        <ErrorBoundary
          fallback={<AboutDialogFailureFallback onFailure={containAboutLoadFailure} />}
        >
          <Suspense fallback={null}>
            <LazyBaseModal
              modelValue={showAbout}
              title={t('common.about.title', { name: APP_NAME })}
              size="sm"
              onUpdateModelValue={setShowAbout}
            >
              <div className="flex flex-col items-center p-2">
                <img src="/icons/logo.svg" alt={`${APP_NAME} logo`} className="mb-4 h-24 w-24 rounded-2xl object-cover" />
                <h2 className="mb-1 text-2xl font-bold tracking-tight">{APP_NAME}</h2>
                <div className="mb-4 flex items-center space-x-2 text-xs">
                  <span className="rounded-full border border-accent-primary/20 bg-accent-primary/10 px-2 py-0.5 text-accent-primary">
                    {APP_TAGLINE}
                  </span>
                  <span className="text-text-muted">v{APP_VERSION}</span>
                </div>
                <p className="mb-6 text-center text-sm text-text-secondary">{t('common.about.description')}</p>
                <div className="mb-4 w-full space-y-2">
                  <div className="flex items-center justify-between rounded-lg border border-border-default/60 p-2 text-xs">
                    <span className="text-text-muted">{t('common.about.owner')}</span>
                    <span className="font-medium">{APP_OWNER}</span>
                  </div>
                  <div className="flex items-center justify-between rounded-lg border border-border-default/60 p-2 text-xs">
                    <span className="text-text-muted">{t('common.about.engine')}</span>
                    <span className="font-medium">{APP_ENGINE}</span>
                  </div>
                </div>
                <button
                  type="button"
                  className="w-full rounded-xl border border-border-default/60 bg-bg-surface py-2 text-sm font-medium"
                  onClick={() => setShowAbout(false)}
                >
                  {t('common.about.close')}
                </button>
              </div>
            </LazyBaseModal>
          </Suspense>
        </ErrorBoundary>
      ) : null}
    </div>
  )
}
