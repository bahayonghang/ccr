import { useCallback, useEffect, useMemo, useState } from 'react'
import { FormProvider, useForm } from 'react-hook-form'
import {
  codexAddAuthWithApiKey,
  codexImportAuthFromLocal,
  codexImportAuthPayload,
  codexOAuthLoginCancel,
} from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { BaseModal, SIcon } from '@/ui'
import type { CodexAuthMutationResponse, CodexModelProviderRecord } from '@/types'
import type { ProviderTemplateSelection } from '@/types/providerTemplates'
import { extractErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import { mapTemplateToCodexApiAccountPatch } from '@/utils/providerTemplates'
import {
  canCustomizeAccountName,
  detectImportPayloadNamingState,
  getAccountNameValidationMessage,
  normalizeAccountNameInput,
} from '../codexAuthAccounts'
import { preferredNameErrorText } from './naming'
import { ghostBtnClass } from '../ui-classes'
import { useCodexLocale } from '../useCodexLocale'
import { AddAccountApiStep } from './AddAccountApiStep'
import { AddAccountLocalStep } from './AddAccountLocalStep'
import { AddAccountOauthStep } from './AddAccountOauthStep'
import { AddAccountTokenStep } from './AddAccountTokenStep'
import { ADD_ACCOUNT_DEFAULTS, type AddAccountFormValues, type AddMethod } from './addAccountForm'
import { useCodexOAuthFlow } from './useCodexOAuthFlow'

interface AddCodexAccountModalProps {
  modelValue: boolean
  providers: CodexModelProviderRecord[]
  canManageAuthAccounts: boolean
  initialMethod?: AddMethod
  presetProvider?: CodexModelProviderRecord | null
  refreshOnMutation?: () => Promise<void> | void
  onUpdateModelValue: (value: boolean) => void
}

export function AddCodexAccountModal({
  modelValue,
  providers,
  canManageAuthAccounts,
  initialMethod = 'oauth',
  presetProvider = null,
  refreshOnMutation,
  onUpdateModelValue,
}: AddCodexAccountModalProps) {
  const { t, tf } = useCodexLocale()
  const form = useForm<AddAccountFormValues>({ defaultValues: ADD_ACCOUNT_DEFAULTS })
  const values = form.watch()
  const [activeAddMethod, setActiveAddMethod] = useState<AddMethod>('oauth')
  const [addAccountError, setAddAccountError] = useState<string | null>(null)
  const [addAccountNotice, setAddAccountNotice] = useState<string | null>(null)
  const [importBusy, setImportBusy] = useState(false)
  const [apiKeyBusy, setApiKeyBusy] = useState(false)
  const [localImportBusy, setLocalImportBusy] = useState(false)
  const [selectedApiTemplate, setSelectedApiTemplate] = useState<string | null>(null)

  const namingState = detectImportPayloadNamingState(values.importContent)
  const canCustomize = canCustomizeAccountName(activeAddMethod, namingState)
  const normalizedName = normalizeAccountNameInput(values.preferredAccountName)
  const preferredAccountNameError = canCustomize
    ? preferredNameErrorText(getAccountNameValidationMessage(normalizedName), tf)
    : null
  const effectivePreferredAccountName = canCustomize && !preferredAccountNameError ? normalizedName : null

  const ensurePreferredAccountNameIsValid = useCallback(() => {
    if (preferredAccountNameError) {
      setAddAccountError(preferredAccountNameError)
      return false
    }
    return true
  }, [preferredAccountNameError])

  const applyMutationSuccess = useCallback(
    async (result: CodexAuthMutationResponse, successMessage: string) => {
      await refreshOnMutation?.()
      surfaceNotify.success(successMessage)
      setAddAccountNotice(result.account_name ? tf('codex.auth.feedback.savedAs', 'Saved as {name}.', { name: result.account_name }) : successMessage)
    },
    [refreshOnMutation, tf],
  )

  const oauth = useCodexOAuthFlow({
    t,
    effectivePreferredAccountName,
    ensurePreferredAccountNameIsValid,
    applyMutationSuccess,
    setAddAccountError,
    setAddAccountNotice,
    setShowAddAccountModal: onUpdateModelValue,
  })

  const closeModal = useCallback(async () => {
    onUpdateModelValue(false)
    setAddAccountError(null)
    setAddAccountNotice(null)
    oauth.setOauthTimeoutMessage(null)
    if (oauth.oauthPending && oauth.oauthLoginId) {
      try {
        await codexOAuthLoginCancel(oauth.oauthLoginId)
      } catch (error) {
        logger.warn('Failed to cancel oauth flow while closing modal:', error)
      }
    }
    oauth.resetOauthState()
    form.reset(ADD_ACCOUNT_DEFAULTS)
  }, [form, onUpdateModelValue, oauth])

  const handleClose = useCallback(
    (open: boolean) => {
      if (!open) void closeModal()
    },
    [closeModal],
  )
  const clearApiTemplate = useCallback(() => {
    setSelectedApiTemplate(null)
  }, [])
  const renderHeader = useCallback(
    ({ titleId }: { titleId: string }) => (
      <div className="flex items-center justify-between border-b border-border-default/10 px-6 py-4">
        <div>
          <h2 id={titleId} className="text-xl font-bold text-text-primary">{tf('codex.auth.actions.addAccount', 'Add account')}</h2>
          <p className="mt-1 text-sm text-text-muted">{tf('codex.auth.addAccountDescription', 'Store one or more Codex credentials and switch them from CCR.')}</p>
        </div>
        <button type="button" className={ghostBtnClass} onClick={closeModal}>
          <SIcon name="X" size="w-5 h-5" />
        </button>
      </div>
    ),
    [closeModal, tf],
  )

  const switchAddMethod = useCallback(
    async (method: AddMethod) => {
      setActiveAddMethod(method)
      setAddAccountError(null)
      setAddAccountNotice(null)
      if (method === 'oauth') await oauth.refreshOauthPortStatus()
    },
    [oauth],
  )

  const applyProviderToApiForm = useCallback(
    (provider: CodexModelProviderRecord) => {
      form.setValue('providerName', provider.name)
      form.setValue('apiBaseUrl', provider.base_url)
      form.setValue('apiKey', provider.api_keys[0]?.api_key || form.getValues('apiKey'))
      form.setValue('saveProvider', false)
      setSelectedApiTemplate(null)
      setActiveAddMethod('api')
      setAddAccountNotice(tf('codex.auth.api.presetApplied', 'Loaded saved provider "{name}" into the API key form.', { name: provider.name }))
    },
    [form, tf],
  )

  const applyApiTemplate = useCallback(
    (selection: ProviderTemplateSelection) => {
      const patch = mapTemplateToCodexApiAccountPatch(selection.template, selection.endpoint)
      setSelectedApiTemplate(selection.template.id)
      form.setValue('providerName', patch.providerName || selection.template.name)
      form.setValue('apiBaseUrl', patch.apiBaseUrl || '')
      setAddAccountError(null)
    },
    [form],
  )

  const handleImportPayload = useCallback(async () => {
    setAddAccountError(null)
    if (!values.importContent.trim()) {
      setAddAccountError(tf('codex.auth.import.validation.contentRequired', 'Paste a JSON payload before importing it.'))
      return
    }
    if (!ensurePreferredAccountNameIsValid()) return
    try {
      setImportBusy(true)
      const result = await codexImportAuthPayload({
        content: values.importContent,
        switchAfterImport: values.importSwitchAfter && canManageAuthAccounts,
        preferredAccountName: namingState === 'single' ? effectivePreferredAccountName ?? undefined : undefined,
      })
      await applyMutationSuccess(result, tf('codex.auth.import.success', 'Imported account payload successfully.'))
      form.setValue('importContent', '')
      onUpdateModelValue(false)
    } catch (error) {
      setAddAccountError(extractErrorMessage(error) || tf('codex.auth.import.failed', 'Failed to import the JSON payload.'))
    } finally {
      setImportBusy(false)
    }
  }, [applyMutationSuccess, canManageAuthAccounts, effectivePreferredAccountName, ensurePreferredAccountNameIsValid, form, namingState, onUpdateModelValue, tf, values.importContent, values.importSwitchAfter])

  const handleImportFromLocal = useCallback(async () => {
    if (!ensurePreferredAccountNameIsValid()) return
    try {
      setLocalImportBusy(true)
      const result = await codexImportAuthFromLocal(effectivePreferredAccountName)
      await applyMutationSuccess(result, tf('codex.auth.localImport.success', 'Imported the local runtime account successfully.'))
      onUpdateModelValue(false)
    } catch (error) {
      setAddAccountError(extractErrorMessage(error) || tf('codex.auth.localImport.failed', 'Failed to import the local runtime account.'))
    } finally {
      setLocalImportBusy(false)
    }
  }, [applyMutationSuccess, effectivePreferredAccountName, ensurePreferredAccountNameIsValid, onUpdateModelValue, tf])

  const handleAddApiKeyAccount = useCallback(async () => {
    if (!values.apiKey.trim()) {
      setAddAccountError(tf('codex.auth.api.validation.apiKeyRequired', 'Enter an API key before saving the account.'))
      return
    }
    if (!ensurePreferredAccountNameIsValid()) return
    try {
      setApiKeyBusy(true)
      const result = await codexAddAuthWithApiKey({
        apiKey: values.apiKey.trim(),
        apiBaseUrl: values.apiBaseUrl.trim() || undefined,
        providerName: values.providerName.trim() || undefined,
        saveProvider: values.saveProvider,
        switchAfterAdd: values.switchAfterAdd && canManageAuthAccounts,
        preferredAccountName: effectivePreferredAccountName ?? undefined,
      })
      await applyMutationSuccess(result, tf('codex.auth.api.success', 'API key account added successfully.'))
      form.setValue('apiKey', '')
      onUpdateModelValue(false)
    } catch (error) {
      setAddAccountError(extractErrorMessage(error) || tf('codex.auth.api.failed', 'Failed to save the API key account.'))
    } finally {
      setApiKeyBusy(false)
    }
  }, [applyMutationSuccess, canManageAuthAccounts, effectivePreferredAccountName, ensurePreferredAccountNameIsValid, form, onUpdateModelValue, tf, values])

  const installOauthListeners = oauth.installOauthListeners
  const cleanupOauthListeners = oauth.cleanupOauthListeners
  const refreshOauthPortStatus = oauth.refreshOauthPortStatus
  useEffect(() => {
    void installOauthListeners()
    return () => {
      void cleanupOauthListeners()
    }
  }, [cleanupOauthListeners, installOauthListeners])

  useEffect(() => {
    if (!modelValue) return
    setActiveAddMethod(initialMethod)
    setAddAccountError(null)
    setAddAccountNotice(null)
    form.reset(ADD_ACCOUNT_DEFAULTS)
    if (presetProvider) applyProviderToApiForm(presetProvider)
    void refreshOauthPortStatus()
  }, [applyProviderToApiForm, form, initialMethod, modelValue, presetProvider, refreshOauthPortStatus])

  const tabs = useMemo(
    () => [
      { value: 'oauth' as const, label: tf('codex.auth.methods.oauth', 'OAuth'), icon: 'Globe' },
      { value: 'token' as const, label: tf('codex.auth.methods.token', 'Token / JSON'), icon: 'FileJson' },
      { value: 'api' as const, label: tf('codex.auth.methods.api', 'API Key'), icon: 'KeyRound' },
      { value: 'local' as const, label: tf('codex.auth.methods.local', 'Local import'), icon: 'FolderDown' },
    ],
    [tf],
  )

  return (
    <BaseModal
      modelValue={modelValue}
      size="full"
      surface="glass"
      contentClass="w-full max-w-[min(70rem,calc(100vw-2rem))] max-h-[92vh] overflow-y-auto"
      onUpdateModelValue={handleClose}
      header={renderHeader}
    >
      <FormProvider {...form}>
        <div className="codex-auth-view__composer-shell">
          <aside className="codex-auth-view__composer-sidebar">
            <div className="codex-auth-view__composer-card">
              <p className="codex-auth-view__composer-eyebrow">{tf('codex.auth.naming.eyebrow', 'Account blueprint')}</p>
              <h3 className="codex-auth-view__composer-title">{tf('codex.auth.naming.title', 'Decide how this account should be saved')}</h3>
              <label className="codex-auth-view__input-group">
                <span className="codex-auth-view__input-label">{tf('codex.auth.naming.fieldLabel', 'Custom saved name')}</span>
                <input data-testid="codex-add-account-name-input" className="input" disabled={!canCustomize} {...form.register('preferredAccountName')} />
              </label>
              <p data-testid="codex-add-account-name-helper" className={preferredAccountNameError ? 'codex-auth-view__composer-helper--error' : 'codex-auth-view__composer-helper'}>
                {preferredAccountNameError || tf('codex.auth.naming.helper.auto', 'Leave this empty to let CCR derive the account name from email, provider, or runtime metadata.')}
              </p>
            </div>
          </aside>
          <div className="codex-auth-view__composer-main">
            <div className="codex-auth-view__segment-row codex-auth-view__segment-row--modal">
              {tabs.map((tab) => (
                <MethodTab key={tab.value} tab={tab} active={activeAddMethod === tab.value} onSelect={switchAddMethod} />
              ))}
            </div>
            {addAccountNotice ? <div className="codex-auth-view__inline-note">{addAccountNotice}</div> : null}
            {addAccountError ? <div className="codex-auth-view__inline-error">{addAccountError}</div> : null}
            {activeAddMethod === 'oauth' ? (
              <AddAccountOauthStep
                tf={tf}
                oauthPortBusy={oauth.oauthPortBusy}
                oauthPending={oauth.oauthPending}
                oauthBusy={oauth.oauthBusy}
                oauthTimeoutMessage={oauth.oauthTimeoutMessage}
                oauthAuthUrl={oauth.oauthAuthUrl}
                oauthCallbackUrl={oauth.oauthCallbackUrl}
                nameError={preferredAccountNameError}
                onReleasePort={oauth.handleReleaseOauthPort}
                onStart={oauth.handleStartOauth}
                onFinalize={oauth.handleFinalizeOauth}
                onCancel={oauth.cancelOauthFlow}
                onCallbackChange={oauth.setOauthCallbackUrl}
                onSubmitCallback={oauth.handleSubmitOauthCallback}
              />
            ) : null}
            {activeAddMethod === 'token' ? (
              <AddAccountTokenStep
                tf={tf}
                register={form.register}
                canManageAuthAccounts={canManageAuthAccounts}
                importBusy={importBusy}
                canSubmit={Boolean(values.importContent.trim()) && !preferredAccountNameError}
                onImport={handleImportPayload}
              />
            ) : null}
            {activeAddMethod === 'api' ? (
              <AddAccountApiStep
                tf={tf}
                register={form.register}
                providers={providers}
                selectedTemplateId={selectedApiTemplate}
                apiKeyBusy={apiKeyBusy}
                canSubmit={Boolean(values.apiKey.trim()) && !preferredAccountNameError}
                onApplyTemplate={applyApiTemplate}
                onManualTemplate={clearApiTemplate}
                onApplyProvider={applyProviderToApiForm}
                onSave={handleAddApiKeyAccount}
              />
            ) : null}
            {activeAddMethod === 'local' ? (
              <AddAccountLocalStep tf={tf} localImportBusy={localImportBusy} canSubmit={!preferredAccountNameError} onImport={handleImportFromLocal} />
            ) : null}
          </div>
        </div>
      </FormProvider>
    </BaseModal>
  )
}

function MethodTab({
  tab,
  active,
  onSelect,
}: {
  tab: { value: AddMethod; label: string; icon: string }
  active: boolean
  onSelect: (method: AddMethod) => void
}) {
  const handleClick = useCallback(() => {
    void onSelect(tab.value)
  }, [onSelect, tab.value])
  return (
    <button
      type="button"
      className={active ? 'codex-auth-view__segment codex-auth-view__segment--modal codex-auth-view__segment--active' : 'codex-auth-view__segment codex-auth-view__segment--modal'}
      onClick={handleClick}
    >
      <SIcon name={tab.icon} size="w-4 h-4" />
      <span>{tab.label}</span>
    </button>
  )
}
