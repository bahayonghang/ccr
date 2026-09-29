import { Button } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

export function SettingsSessionNotice({ frozen, changed, reloadRequired, error, actionError, busy, onReturn, onReload, t }: {
  frozen: boolean
  changed: boolean
  reloadRequired: boolean
  error: unknown
  actionError: string | null
  busy: boolean
  onReturn: () => Promise<void>
  onReload: () => Promise<void>
  t: TranslateFunction
}) {
  if (!frozen && !changed && !reloadRequired && !error && !actionError) return null
  const message = frozen ? 'settingsSession.environmentChanged'
    : changed ? 'settingsSession.serverChanged' : reloadRequired ? 'settingsSession.reloadRequired' : null
  return (
    <div role="alert" className="mb-4 space-y-2 rounded-xl border border-border-default p-4 text-sm" data-testid="settings-session-notice">
      {message ? <p>{t(message)}</p> : null}
      {error ? <p>{error instanceof Error ? error.message : String(error)}</p> : null}
      {actionError ? <p>{actionError}</p> : null}
      <div className="flex gap-2">
        {frozen ? <Button disabled={busy} onClick={onReturn}>{t('settingsSession.returnEnvironment')}</Button> : null}
        <Button disabled={busy} onClick={onReload}>{t('settingsSession.discardReload')}</Button>
      </div>
    </div>
  )
}
