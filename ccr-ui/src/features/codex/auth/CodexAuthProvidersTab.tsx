import { memo, useCallback } from 'react'
import type { UseFormReturn } from 'react-hook-form'
import type { CodexModelProviderRecord } from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { EmptyState, SIcon } from '@/ui'
import { defaultSurfaceT as t } from '@/features/platform'
import type { CodexTf } from '../useCodexLocale'
import { fieldInputClass, ghostBtnClass, panelCardClass, primaryBtnClass } from '../ui-classes'
import { CodexProviderTemplatePicker } from './CodexProviderTemplatePicker'
import type { CodexProviderForm } from './useCodexProviders'

interface CodexAuthProvidersTabProps {
  providerForm: CodexProviderForm
  providerFormApi: UseFormReturn<CodexProviderForm>
  providerError: string | null
  providerSaving: boolean
  providerLoading: boolean
  providers: CodexModelProviderRecord[]
  selectedProviderTemplate: string | null
  codexTemplateDraft: ProviderTemplateDraftContext
  formatProviderUpdatedAt: (value?: string | null, detailed?: boolean) => string
  tf: CodexTf
  onResetForm: () => void
  onApplyTemplate: (selection: ProviderTemplateSelection) => void
  onUseManualTemplate: () => void
  onSaveProvider: () => void
  onLoadProviders: () => void
  onUseInApiForm: (provider: CodexModelProviderRecord) => void
  onEditProvider: (provider: CodexModelProviderRecord) => void
  onDeleteProvider: (provider: CodexModelProviderRecord) => void
}

const ProviderCard = memo(function ProviderCard({
  provider,
  formatProviderUpdatedAt,
  tf,
  onUseInApiForm,
  onEditProvider,
  onDeleteProvider,
}: {
  provider: CodexModelProviderRecord
  formatProviderUpdatedAt: (value?: string | null, detailed?: boolean) => string
  tf: CodexTf
  onUseInApiForm: (provider: CodexModelProviderRecord) => void
  onEditProvider: (provider: CodexModelProviderRecord) => void
  onDeleteProvider: (provider: CodexModelProviderRecord) => void
}) {
  const handleUse = useCallback(() => onUseInApiForm(provider), [onUseInApiForm, provider])
  const handleEdit = useCallback(() => onEditProvider(provider), [onEditProvider, provider])
  const handleDelete = useCallback(() => onDeleteProvider(provider), [onDeleteProvider, provider])
  return (
    <article className="codex-auth-view__provider-card">
      <div className="codex-auth-view__provider-head">
        <div>
          <h4 className="codex-auth-view__provider-title">{provider.name}</h4>
          <p className="codex-auth-view__provider-url">{provider.base_url}</p>
        </div>
        <div className="codex-auth-view__provider-badges">
          <span className="codex-auth-view__provider-badge">
            {provider.api_keys.length} {tf('codex.auth.providers.badges.keys', 'keys')}
          </span>
          <span className="codex-auth-view__provider-badge codex-auth-view__provider-badge--muted">
            {formatProviderUpdatedAt(provider.updated_at)}
          </span>
        </div>
      </div>
      <div className="codex-auth-view__provider-footer">
        <span>{tf('codex.auth.providers.updatedAt', 'Updated')} {formatProviderUpdatedAt(provider.updated_at, true)}</span>
        <div className="codex-auth-view__provider-actions-inline">
          <button type="button" className={ghostBtnClass} onClick={handleUse}>
            {tf('codex.auth.providers.actions.useInApiForm', 'Use in API form')}
          </button>
          <button type="button" className={ghostBtnClass} onClick={handleEdit}>{tf('common.edit', 'Edit')}</button>
          <button type="button" className={ghostBtnClass} onClick={handleDelete}>{t('codex.actions.delete')}</button>
        </div>
      </div>
    </article>
  )
})

export function CodexAuthProvidersTab({
  providerForm,
  providerFormApi,
  providerError,
  providerSaving,
  providerLoading,
  providers,
  selectedProviderTemplate,
  formatProviderUpdatedAt,
  tf,
  onResetForm,
  onApplyTemplate,
  onUseManualTemplate,
  onSaveProvider,
  onLoadProviders,
  onUseInApiForm,
  onEditProvider,
  onDeleteProvider,
}: CodexAuthProvidersTabProps) {
  const dirty = Boolean(providerForm.id || providerForm.name || providerForm.baseUrl || providerForm.apiKey)
  return (
    <div className="codex-auth-view__providers-grid">
      <section className={panelCardClass}>
        <div className="codex-auth-view__section-header codex-auth-view__section-header--spread">
          <div className="codex-auth-view__title-inline">
            <SIcon name="Blocks" size="w-5 h-5" className="codex-auth-view__section-icon" />
            <div>
              <h3 className="codex-auth-view__section-title">{tf('codex.auth.providers.formTitle', 'Saved provider')}</h3>
              <p className="codex-auth-view__section-copy">
                {tf('codex.auth.providers.formHint', 'Save reusable base URLs and optional API keys. Provider templates only fill non-secret metadata.')}
              </p>
            </div>
          </div>
          {dirty ? (
            <button type="button" className={ghostBtnClass} onClick={onResetForm}>
              {tf('codex.auth.providers.resetForm', 'Reset form')}
            </button>
          ) : null}
        </div>
        <CodexProviderTemplatePicker
          selectedTemplateId={selectedProviderTemplate}
          label="Provider template"
          helper="Search non-secret templates by name, host, tag, or model. API keys stay in the saved provider form."
          manualLabel={tf('codex.auth.providers.manualTemplate', 'Manual')}
          onSelect={onApplyTemplate}
          onManual={onUseManualTemplate}
        />
        <div className="codex-auth-view__provider-form">
          <label className="codex-auth-view__input-group">
            <span className="codex-auth-view__input-label">{tf('codex.auth.providers.fields.name', 'Provider name')}</span>
            <input className={fieldInputClass} {...providerFormApi.register('name')} />
          </label>
          <label className="codex-auth-view__input-group">
            <span className="codex-auth-view__input-label">{tf('codex.auth.providers.fields.baseUrl', 'Base URL')}</span>
            <input className={fieldInputClass} type="url" {...providerFormApi.register('baseUrl')} />
          </label>
          <label className="codex-auth-view__input-group">
            <span className="codex-auth-view__input-label">{tf('codex.auth.providers.fields.websiteUrl', 'Website URL')}</span>
            <input className={fieldInputClass} type="url" {...providerFormApi.register('websiteUrl')} />
          </label>
          <label className="codex-auth-view__input-group">
            <span className="codex-auth-view__input-label">{tf('codex.auth.providers.fields.apiKeyUrl', 'API key docs URL')}</span>
            <input className={fieldInputClass} type="url" {...providerFormApi.register('apiKeyUrl')} />
          </label>
          <label className="codex-auth-view__input-group">
            <span className="codex-auth-view__input-label">{tf('codex.auth.providers.fields.apiKeyName', 'Stored key label')}</span>
            <input className={fieldInputClass} {...providerFormApi.register('apiKeyName')} />
          </label>
          <label className="codex-auth-view__input-group">
            <span className="codex-auth-view__input-label">{tf('codex.auth.providers.fields.apiKey', 'Stored API key')}</span>
            <input className={fieldInputClass} type="password" autoComplete="off" {...providerFormApi.register('apiKey')} />
          </label>
        </div>
        {providerError ? <div className="codex-auth-view__inline-error">{providerError}</div> : null}
        <div className="codex-auth-view__provider-actions">
          <button type="button" className={primaryBtnClass} disabled={providerSaving || !providerForm.name.trim() || !providerForm.baseUrl.trim()} onClick={onSaveProvider}>
            <SIcon name={providerForm.id ? 'Save' : 'Plus'} size="w-4 h-4" />
            {providerForm.id ? tf('codex.auth.providers.actions.update', 'Update provider') : tf('codex.auth.providers.actions.create', 'Save provider')}
          </button>
        </div>
      </section>

      <section className={panelCardClass}>
        <div className="codex-auth-view__section-header codex-auth-view__section-header--spread">
          <div className="codex-auth-view__title-inline">
            <SIcon name="Globe" size="w-5 h-5" className="codex-auth-view__section-icon" />
            <div>
              <h3 className="codex-auth-view__section-title">{tf('codex.auth.providers.listTitle', 'Saved providers')}</h3>
              <p className="codex-auth-view__section-copy">
                {tf('codex.auth.providers.listHint', 'Saved providers can include API keys and can be injected directly into the API key account flow.')}
              </p>
            </div>
          </div>
          <button type="button" className={ghostBtnClass} disabled={providerLoading} onClick={onLoadProviders}>
            <SIcon name="RefreshCw" size="w-4 h-4" className={providerLoading ? 'animate-spin' : undefined} />
            {t('codex.auth.refresh')}
          </button>
        </div>
        {providerLoading ? (
          <div className="space-y-3">
            <div className="h-24 animate-pulse rounded-2xl bg-bg-elevated" />
            <div className="h-24 animate-pulse rounded-2xl bg-bg-elevated" />
          </div>
        ) : providers.length === 0 ? (
          <EmptyState
            icon="Blocks"
            title={tf('codex.auth.providers.emptyState', 'No saved providers yet')}
            description={tf('codex.auth.providers.emptyHint', 'Create a saved provider if you often switch between OpenAI-compatible gateways.')}
          />
        ) : (
          <div className="codex-auth-view__provider-list">
            {providers.map((provider) => (
              <ProviderCard
                key={provider.id}
                provider={provider}
                formatProviderUpdatedAt={formatProviderUpdatedAt}
                tf={tf}
                onUseInApiForm={onUseInApiForm}
                onEditProvider={onEditProvider}
                onDeleteProvider={onDeleteProvider}
              />
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
