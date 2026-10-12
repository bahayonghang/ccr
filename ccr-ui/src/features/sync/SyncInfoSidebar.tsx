import { useCallback, useState } from 'react'
import { clearWebdavConfig } from '@/api'
import type { SyncStatusView } from '@/types/syncSelection'
import { logger } from '@/utils/logger'
import { BaseModal, buttonClass } from '@/ui'
import { useSyncT } from './locale'
import { SyncAccountDialog } from './SyncAccountDialog'

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
      <div className="sync-account-panel">
        <h2 className="sync-account-panel__title">{t('sync.webdav.title')}</h2>
        {configured ? (
          <div className="space-y-4">
            <div className="space-y-3">
              <Detail label={t('sync.webdav.server')} value={syncStatus?.webdav_url ?? ''} />
              <Detail label={t('sync.webdav.username')} value={syncStatus?.username ?? ''} />
              <Detail label={t('sync.webdav.remotePath')} value={syncStatus?.remote_path ?? ''} />
            </div>
            <div className="flex flex-wrap gap-2">
              <button type="button" className={buttonClass({ variant: 'secondary', size: 'sm' })} onClick={openEdit}>{t('sync.account.editBtn')}</button>
              <button type="button" className={buttonClass({ variant: 'secondary', size: 'sm' })} onClick={onTestExisting}>{testing ? t('sync.account.testing') : t('sync.account.testBtn')}</button>
              <button type="button" className={buttonClass({ variant: 'secondary', size: 'sm' })} onClick={askDisconnect}>{t('sync.account.disconnectBtn')}</button>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            <p className="text-sm text-text-secondary">{t('sync.webdav.notConfigured')}</p>
            <button type="button" className={buttonClass({ variant: 'secondary', size: 'sm' })} onClick={openAdd}>{t('sync.account.addCta')}</button>
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
