import { useCallback, useState } from 'react'
import { clearWebdavConfig } from '@/api'
import type { SyncStatusView } from '@/types/syncSelection'
import { logger } from '@/utils/logger'
import { BaseModal, SIcon } from '@/ui'
import { useSyncT } from './locale'
import { SyncAccountDialog } from './SyncAccountDialog'

function connectionChip(syncStatus: SyncStatusView | null, t: ReturnType<typeof useSyncT>) {
  if (syncStatus?.remote_accessible === true) {
    return { className: 'bg-accent-success/12 text-accent-success', icon: 'CheckCircle', text: t('sync.webdav.connected') }
  }
  if (syncStatus?.remote_accessible === false) {
    return { className: 'bg-accent-danger/12 text-accent-danger', icon: 'AlertCircle', text: t('sync.webdav.unreachable') }
  }
  return { className: 'bg-bg-elevated text-text-muted', icon: 'Cloud', text: t('sync.webdav.untested') }
}

interface SyncInfoSidebarProps {
  syncStatus: SyncStatusView | null
  onStatusRefresh: () => void
  accountDialogOpen: boolean
  accountDialogMode: 'add' | 'edit'
  onAccountDialogOpenChange: (open: boolean) => void
  onOpenAccountDialog: (mode: 'add' | 'edit') => void
}

export function SyncInfoSidebar({
  syncStatus,
  onStatusRefresh,
  accountDialogOpen,
  accountDialogMode,
  onAccountDialogOpenChange,
  onOpenAccountDialog,
}: SyncInfoSidebarProps) {
  const t = useSyncT()
  const [confirmingDisconnect, setConfirmingDisconnect] = useState(false)
  const [disconnecting, setDisconnecting] = useState(false)
  const [testing, setTesting] = useState(false)
  const configured = Boolean(syncStatus?.configured)
  const chip = connectionChip(syncStatus, t)

  const openAdd = useCallback(() => {
    onOpenAccountDialog('add')
  }, [onOpenAccountDialog])
  const openEdit = useCallback(() => {
    onOpenAccountDialog('edit')
  }, [onOpenAccountDialog])
  const closeDisconnect = useCallback(() => {
    setConfirmingDisconnect(false)
  }, [])
  const askDisconnect = useCallback(() => {
    setConfirmingDisconnect(true)
  }, [])
  const onTestExisting = useCallback(() => {
    setTesting(true)
    onStatusRefresh()
    window.setTimeout(() => setTesting(false), 600)
  }, [onStatusRefresh])
  const onDisconnect = useCallback(async () => {
    setDisconnecting(true)
    try {
      await clearWebdavConfig()
      setConfirmingDisconnect(false)
      onStatusRefresh()
    } catch (err) {
      logger.error('clear_webdav_config failed:', err)
    } finally {
      setDisconnecting(false)
    }
  }, [onStatusRefresh])
  const handleDisconnect = useCallback(() => {
    void onDisconnect()
  }, [onDisconnect])

  return (
    <>
      <div className="rounded-2xl border border-border-default/25 bg-bg-elevated p-6">
        <div className="mb-5 flex items-center justify-between gap-3">
          <h2 className="text-xl font-bold text-text-primary">{t('sync.webdav.title')}</h2>
          {configured ? (
            <span className={`inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium ${chip.className}`}>
              <SIcon name={chip.icon} size="w-3.5 h-3.5" />
              {chip.text}
            </span>
          ) : null}
        </div>
        {configured ? (
          <div className="space-y-4">
            <div className="space-y-3">
              <Detail label={t('sync.webdav.server')} value={syncStatus?.webdav_url ?? ''} />
              <Detail label={t('sync.webdav.username')} value={syncStatus?.username ?? ''} />
              <Detail label={t('sync.webdav.remotePath')} value={syncStatus?.remote_path ?? ''} />
            </div>
            <div className="flex flex-wrap gap-2">
              <button type="button" className="rounded-lg bg-accent-primary px-3 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]" onClick={openEdit}>{t('sync.account.editBtn')}</button>
              <button type="button" className="rounded-lg border border-border-default px-3 py-2 text-sm" onClick={onTestExisting}>{testing ? t('sync.account.testing') : t('sync.account.testBtn')}</button>
              <button type="button" className="rounded-lg border border-border-default px-3 py-2 text-sm text-accent-danger" onClick={askDisconnect}>{t('sync.account.disconnectBtn')}</button>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            <p className="text-sm text-text-secondary">{t('sync.webdav.notConfigured')}</p>
            <button type="button" className="w-full rounded-lg bg-accent-primary px-4 py-3 text-sm font-semibold text-[color:var(--color-accent-primary-contrast)]" onClick={openAdd}>{t('sync.account.addCta')}</button>
          </div>
        )}
      </div>

      <SyncAccountDialog open={accountDialogOpen} mode={accountDialogMode} initial={syncStatus} onOpenChange={onAccountDialogOpenChange} onSaved={onStatusRefresh} />
      <BaseModal modelValue={confirmingDisconnect} title={t('sync.account.disconnectConfirmTitle')} size="sm" surface="solid" closeOnBackdrop={!disconnecting} closeOnEscape={!disconnecting} onUpdateModelValue={setConfirmingDisconnect} footer={
        <div className="flex w-full gap-2">
          <button type="button" className="flex-1 rounded-lg border border-border-default px-3 py-2" disabled={disconnecting} onClick={closeDisconnect}>{t('sync.account.cancelBtn')}</button>
          <button type="button" className="flex-1 rounded-lg bg-accent-danger px-3 py-2 text-[color:var(--color-danger-contrast)]" disabled={disconnecting} onClick={handleDisconnect}>{t('sync.account.disconnectConfirmBtn')}</button>
        </div>
      }>
        <p className="text-sm text-text-secondary">{t('sync.account.disconnectConfirmBody')}</p>
      </BaseModal>
    </>
  )
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-1">
      <div className="text-xs text-text-muted">{label}</div>
      <div className="break-all font-mono text-sm text-text-primary">{value}</div>
    </div>
  )
}
