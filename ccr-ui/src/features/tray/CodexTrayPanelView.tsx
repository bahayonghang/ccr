import { useCallback, type MouseEvent } from 'react'
import { PageHeader, SIcon } from '@/ui'
import { useTrayTt } from './locale'
import { TrayAccountSwitchScreen } from './TrayAccountSwitchScreen'
import { TrayOverview } from './TrayOverview'
import { useCodexTrayPanel } from './useCodexTrayPanel'
import './styles/tray.css'

export function CodexTrayPanelView() {
  const tt = useTrayTt()
  const panel = useCodexTrayPanel()
  const { loadSnapshot, startPanelDrag, openMain, openUsage, openAuth, quit, switchAccount } = panel
  const snapshotStatusLabel = panel.loading
    ? tt('正在加载 Codex 托盘…', 'Loading Codex tray…')
    : tt('暂时还没有托盘快照。', 'No tray snapshot yet.')

  const handleRefresh = useCallback(() => {
    void loadSnapshot(true)
  }, [loadSnapshot])
  const handleDrag = useCallback((event: MouseEvent<HTMLDivElement>) => {
    if (event.button !== 0) return
    event.preventDefault()
    void startPanelDrag()
  }, [startPanelDrag])
  const handleOpenMain = useCallback(() => {
    void openMain()
  }, [openMain])
  const handleOpenUsage = useCallback(() => {
    void openUsage()
  }, [openUsage])
  const handleOpenAuth = useCallback(() => {
    void openAuth()
  }, [openAuth])
  const handleQuit = useCallback(() => {
    void quit()
  }, [quit])
  const handleSwitch = useCallback((name: string) => {
    void switchAccount(name)
  }, [switchAccount])

  return (
    <main className="codex-tray-panel">
      <section className="codex-tray-panel__shell">
        <header className="codex-tray-panel__header">
          <div
            className={`codex-tray-panel__drag-surface${panel.isDragging ? ' codex-tray-panel__drag-surface--dragging' : ''}`}
            title={tt('拖动窗口', 'Drag window')}
            onMouseDown={handleDrag}
          >
            <span className="codex-tray-panel__drag-grip" aria-hidden="true">
              <span />
              <span />
              <span />
            </span>
            <PageHeader className="codex-tray-panel__header-copy" title="Codex Tray" eyebrow="CCR Desktop" />
          </div>
          <button type="button" className="codex-tray-panel__icon-button" title="Refresh" disabled={panel.loading} onClick={handleRefresh}>
            <SIcon name="RefreshCw" size="w-4 h-4" className={panel.loading ? 'animate-spin' : ''} />
          </button>
        </header>

        {panel.error ? (
          <div className="codex-tray-panel__callout codex-tray-panel__callout--danger" aria-live="polite">
            <SIcon name="AlertTriangle" size="w-4 h-4" />
            <p>{panel.error}</p>
          </div>
        ) : null}

        {panel.snapshot && panel.screen === 'overview' ? (
          <TrayOverview
            snapshot={panel.snapshot}
            currentAccount={panel.currentAccount}
            canManageAccounts={panel.canManageAccounts}
            onOpenMain={handleOpenMain}
            onOpenSwitch={panel.goToSwitchScreen}
            onOpenUsage={handleOpenUsage}
            onOpenAuth={handleOpenAuth}
            onQuit={handleQuit}
          />
        ) : null}

        {panel.snapshot && panel.screen === 'switch' ? (
          <TrayAccountSwitchScreen
            snapshot={panel.snapshot}
            currentAccount={panel.currentAccount}
            accounts={panel.accounts}
            busyAccount={panel.busyAccount}
            canManageAccounts={panel.canManageAccounts}
            onBack={panel.goToOverview}
            onSwitch={handleSwitch}
            onOpenAuth={handleOpenAuth}
          />
        ) : null}

        {!panel.snapshot ? (
          <div className="codex-tray-panel__callout">
            <SIcon name="Clock3" size="w-4 h-4" />
            <p>{snapshotStatusLabel}</p>
          </div>
        ) : null}
      </section>
    </main>
  )
}
