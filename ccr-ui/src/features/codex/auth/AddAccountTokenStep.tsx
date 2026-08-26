import type { UseFormRegister } from 'react-hook-form'
import { SIcon, buttonClass } from '@/ui'
import type { CodexTf } from '../useCodexLocale'
import type { AddAccountFormValues } from './addAccountForm'

interface AddAccountTokenStepProps {
  tf: CodexTf
  register: UseFormRegister<AddAccountFormValues>
  canManageAuthAccounts: boolean
  importBusy: boolean
  canSubmit: boolean
  onImport: () => void
}

export function AddAccountTokenStep({
  tf,
  register,
  canManageAuthAccounts,
  importBusy,
  canSubmit,
  onImport,
}: AddAccountTokenStepProps) {
  return (
    <section className="rounded-2xl border border-border-default/15 bg-bg-surface p-5">
      <div className="codex-auth-view__title-inline">
        <SIcon name="FileJson" size="w-5 h-5" className="codex-auth-view__section-icon" />
        <div>
          <h3 className="codex-auth-view__section-title">{tf('codex.auth.import.title', 'Import token / auth JSON')}</h3>
          <p className="codex-auth-view__section-copy">
            {tf('codex.auth.import.hint', 'Paste a single auth.json payload or a Cockpit Tools-style export bundle. CCR will normalize and save each account entry.')}
          </p>
        </div>
      </div>
      <label className="codex-auth-view__input-group codex-auth-view__input-group--full">
        <span className="codex-auth-view__input-label">{tf('codex.auth.import.payload', 'JSON payload')}</span>
        <textarea
          rows={14}
          className="codex-auth-view__textarea codex-auth-view__textarea--mono"
          placeholder={tf('codex.auth.import.placeholder', 'Paste auth.json, export JSON, or a serialized Codex account payload here...')}
          {...register('importContent')}
        />
      </label>
      <div className="codex-auth-view__checkbox-row">
        <label className="codex-auth-view__checkbox-label">
          <input type="checkbox" disabled={!canManageAuthAccounts} {...register('importSwitchAfter')} />
          <span>{tf('codex.auth.import.switchAfter', 'Switch to the first imported account immediately')}</span>
        </label>
      </div>
      <div className="codex-auth-view__provider-actions">
        <button type="button" className={buttonClass({ variant: 'primary' })} disabled={importBusy || !canSubmit} onClick={onImport}>
          <SIcon name="Download" size="w-4 h-4" />
          {tf('codex.auth.import.action', 'Import payload')}
        </button>
      </div>
    </section>
  )
}
