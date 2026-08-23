import { SIcon } from '@/ui'
import type { CodexTf } from '../useCodexLocale'
import { primaryBtnClass } from '../ui-classes'

interface AddAccountLocalStepProps {
  tf: CodexTf
  localImportBusy: boolean
  canSubmit: boolean
  onImport: () => void
}

export function AddAccountLocalStep({ tf, localImportBusy, canSubmit, onImport }: AddAccountLocalStepProps) {
  return (
    <section className="rounded-2xl border border-border-default/15 bg-bg-surface p-5">
      <div className="codex-auth-view__title-inline">
        <SIcon name="FolderDown" size="w-5 h-5" className="codex-auth-view__section-icon" />
        <div>
          <h3 className="codex-auth-view__section-title">{tf('codex.auth.localImport.title', 'Import from local Codex runtime')}</h3>
          <p className="codex-auth-view__section-copy">
            {tf('codex.auth.localImport.hint', 'This reads the active local auth.json and turns it into a managed CCR account entry.')}
          </p>
        </div>
      </div>
      <p className="codex-auth-view__section-copy">
        {tf('codex.auth.localImport.note', 'Use this when Codex is already logged in on this machine and you want CCR to snapshot that session.')}
      </p>
      <button type="button" className={primaryBtnClass} disabled={localImportBusy || !canSubmit} onClick={onImport}>
        <SIcon name="FolderDown" size="w-4 h-4" />
        {tf('codex.auth.localImport.action', 'Import local runtime account')}
      </button>
    </section>
  )
}
