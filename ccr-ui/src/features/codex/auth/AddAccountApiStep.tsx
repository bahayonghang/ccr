import { memo, useCallback } from 'react'
import type { UseFormRegister } from 'react-hook-form'
import type { CodexModelProviderRecord } from '@/types'
import type { ProviderTemplateSelection } from '@/types/providerTemplates'
import { SIcon } from '@/ui'
import type { CodexTf } from '../useCodexLocale'
import { fieldInputClass, ghostBtnClass, primaryBtnClass } from '../ui-classes'
import type { AddAccountFormValues } from './addAccountForm'
import { CodexProviderTemplatePicker } from './CodexProviderTemplatePicker'

interface AddAccountApiStepProps {
  tf: CodexTf
  register: UseFormRegister<AddAccountFormValues>
  providers: CodexModelProviderRecord[]
  selectedTemplateId: string | null
  apiKeyBusy: boolean
  canSubmit: boolean
  onApplyTemplate: (selection: ProviderTemplateSelection) => void
  onManualTemplate: () => void
  onApplyProvider: (provider: CodexModelProviderRecord) => void
  onSave: () => void
}

const PresetRow = memo(function PresetRow({
  provider,
  onApply,
}: {
  provider: CodexModelProviderRecord
  onApply: (provider: CodexModelProviderRecord) => void
}) {
  const handleClick = useCallback(() => onApply(provider), [onApply, provider])
  return (
    <button type="button" className={ghostBtnClass} onClick={handleClick}>
      <span>{provider.name}</span>
      <span className="codex-auth-view__preset-meta">{provider.api_keys.length}</span>
    </button>
  )
})

export function AddAccountApiStep({
  tf,
  register,
  providers,
  selectedTemplateId,
  apiKeyBusy,
  canSubmit,
  onApplyTemplate,
  onManualTemplate,
  onApplyProvider,
  onSave,
}: AddAccountApiStepProps) {
  return (
    <div className="codex-auth-view__providers-grid codex-auth-view__providers-grid--modal">
      <section className="rounded-2xl border border-border-default/15 bg-bg-surface p-5">
        <div className="codex-auth-view__title-inline">
          <SIcon name="KeyRound" size="w-5 h-5" className="codex-auth-view__section-icon" />
          <div>
            <h3 className="codex-auth-view__section-title">{tf('codex.auth.api.title', 'Create API key account')}</h3>
            <p className="codex-auth-view__section-copy">
              {tf('codex.auth.api.hint', 'Store one API key as a named Codex account, optionally attaching it to a reusable saved provider.')}
            </p>
          </div>
        </div>
        <CodexProviderTemplatePicker
          selectedTemplateId={selectedTemplateId}
          label={tf('codex.auth.api.templateLabel', 'Provider template')}
          helper={tf('codex.auth.api.templateHelper', 'Templates fill non-secret metadata only.')}
          manualLabel={tf('codex.auth.providers.manualTemplate', 'Manual')}
          onSelect={onApplyTemplate}
          onManual={onManualTemplate}
        />
        <label className="codex-auth-view__input-group">
          <span className="codex-auth-view__input-label">{tf('codex.auth.api.fields.providerName', 'Provider name')}</span>
          <input className={fieldInputClass} {...register('providerName')} />
        </label>
        <label className="codex-auth-view__input-group">
          <span className="codex-auth-view__input-label">{tf('codex.auth.api.fields.baseUrl', 'Base URL')}</span>
          <input className={fieldInputClass} type="url" {...register('apiBaseUrl')} />
        </label>
        <label className="codex-auth-view__input-group">
          <span className="codex-auth-view__input-label">{tf('codex.auth.api.fields.apiKey', 'API key')}</span>
          <input className={fieldInputClass} type="password" autoComplete="off" {...register('apiKey')} />
        </label>
        <label className="codex-auth-view__checkbox-label">
          <input type="checkbox" {...register('saveProvider')} />
          <span>{tf('codex.auth.api.saveProvider', 'Also save/update saved provider')}</span>
        </label>
        <label className="codex-auth-view__checkbox-label">
          <input type="checkbox" {...register('switchAfterAdd')} />
          <span>{tf('codex.auth.api.switchAfter', 'Switch to the new API account immediately')}</span>
        </label>
        <button type="button" className={primaryBtnClass} disabled={apiKeyBusy || !canSubmit} onClick={onSave}>
          <SIcon name="KeyRound" size="w-4 h-4" />
          {tf('codex.auth.api.action', 'Save API account')}
        </button>
      </section>
      <section className="rounded-2xl border border-border-default/15 bg-bg-surface p-5">
        <h3 className="codex-auth-view__section-title">{tf('codex.auth.api.presetsTitle', 'Saved providers')}</h3>
        {providers.length === 0 ? (
          <p className="text-sm text-text-muted">{tf('codex.auth.api.noPresets', 'No saved providers yet')}</p>
        ) : (
          <div className="space-y-2">
            {providers.map((provider) => (
              <PresetRow key={provider.id} provider={provider} onApply={onApplyProvider} />
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
