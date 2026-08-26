import { useCallback, useEffect, useMemo, useReducer } from 'react'
import { useForm } from 'react-hook-form'
import { createCheckinAccount, getOAuthAuthorizeUrl } from '@/api'
import type { BuiltinProvider } from '@/types/checkin'
import { copyText } from '@/utils/clipboard'
import { BaseModal, buttonClass } from '@/ui'
import { useCheckinT } from '../hooks/useCheckinT'
import {
  initialOAuthWizardState,
  oauthWizardReducer,
  type OAuthType,
} from '../lib/oauthWizardReducer'
import { extractApiUserFromCredentials, parseCookies } from '../lib/parseCredentials'
import { OAuthWizardBody } from './OAuthWizardBody'
import '../styles/oauth.css'

interface OAuthWizardModalProps {
  isOpen: boolean
  builtinProviders: BuiltinProvider[]
  onClose?: () => void
  onSuccess?: () => void
  onUpdateIsOpen?: (open: boolean) => void
}

interface OAuthFormValues {
  provider_id: string
  oauth_type: OAuthType
  credentials: string
  api_user: string
  account_name: string
}

export function OAuthWizardModal({
  isOpen,
  builtinProviders,
  onClose,
  onSuccess,
  onUpdateIsOpen,
}: OAuthWizardModalProps) {
  const t = useCheckinT()
  const [state, dispatch] = useReducer(oauthWizardReducer, undefined, initialOAuthWizardState)
  const form = useForm<OAuthFormValues>({
    defaultValues: {
      provider_id: '',
      oauth_type: 'linuxdo',
      credentials: '',
      api_user: '',
      account_name: '',
    },
  })
  const providerId = form.watch('provider_id')
  const oauthType = form.watch('oauth_type')
  const credentials = form.watch('credentials')
  const apiUser = form.watch('api_user')
  const accountName = form.watch('account_name')

  const oauthProviders = useMemo(
    () => builtinProviders.filter((provider) => provider.oauth_config != null),
    [builtinProviders],
  )
  const selectedProvider = builtinProviders.find((provider) => provider.id === providerId)
  const defaultAccountName = selectedProvider
    ? t('checkin.oauthWizard.defaultAccountName', { provider: selectedProvider.name })
    : ''

  useEffect(() => {
    if (!isOpen) return
    dispatch({ type: 'RESET' })
    form.reset()
  }, [form, isOpen])

  useEffect(() => {
    if (!state.copied) return
    const timer = window.setTimeout(() => dispatch({ type: 'CLEAR_COPIED' }), 2000)
    return () => window.clearTimeout(timer)
  }, [state.copied])

  const handleClose = useCallback(() => {
    onClose?.()
    onUpdateIsOpen?.(false)
  }, [onClose, onUpdateIsOpen])

  const selectProvider = useCallback(
    (id: string) => {
      const provider = builtinProviders.find((item) => item.id === id)
      const nextType: OAuthType = provider?.oauth_config?.linuxdo_client_id
        ? 'linuxdo'
        : 'github'
      form.setValue('provider_id', id)
      form.setValue('oauth_type', nextType)
      dispatch({ type: 'SELECT_PROVIDER', id, oauthType: nextType })
    },
    [builtinProviders, form],
  )

  const selectOAuthType = useCallback(
    (next: OAuthType) => {
      form.setValue('oauth_type', next)
      dispatch({ type: 'SELECT_OAUTH_TYPE', oauthType: next })
    },
    [form],
  )

  const goToStep1 = useCallback(async () => {
    dispatch({ type: 'FETCH_URL_START' })
    try {
      const response = await getOAuthAuthorizeUrl({
        provider_id: providerId,
        oauth_type: oauthType,
      })
      if (response.success && response.authorize_url) {
        dispatch({
          type: 'FETCH_URL_SUCCESS',
          url: response.authorize_url,
          guide: response.extraction_guide || [],
        })
        return
      }
      dispatch({
        type: 'FETCH_URL_ERROR',
        message: response.message || t('checkin.oauthWizard.errors.fetchAuthorizeUrlFailed'),
      })
    } catch (error: unknown) {
      dispatch({
        type: 'FETCH_URL_ERROR',
        message:
          error instanceof Error ? error.message : t('checkin.oauthWizard.errors.networkRequestFailed'),
      })
    }
  }, [oauthType, providerId, t])

  const copyUrl = useCallback(async () => {
    if (!(await copyText(state.authorizeUrl))) return
    dispatch({ type: 'COPIED' })
  }, [state.authorizeUrl])

  const goToCredentials = useCallback(() => {
    dispatch({ type: 'GOTO_CREDENTIALS' })
  }, [])

  const goToConfirm = useCallback(() => {
    dispatch({ type: 'CLEAR_PARSE_ERROR' })
    try {
      const parsed = parseCookies(credentials)
      if (Object.keys(parsed).length === 0) {
        dispatch({ type: 'PARSE_ERROR', message: t('checkin.oauthWizard.errors.emptyCookies') })
        return
      }
      const extracted = extractApiUserFromCredentials(credentials)
      if (extracted) form.setValue('api_user', extracted)
      dispatch({ type: 'GOTO_CONFIRM' })
    } catch (error: unknown) {
      const message =
        error instanceof Error && error.message === 'UNRECOGNIZED_CREDENTIALS'
          ? t('checkin.oauthWizard.errors.unrecognizedCredentialsFormat')
          : t('checkin.oauthWizard.errors.parseFailed')
      dispatch({ type: 'PARSE_ERROR', message })
    }
  }, [credentials, form, t])

  const createAccount = useCallback(async () => {
    dispatch({ type: 'CREATE_START' })
    try {
      if (!selectedProvider) throw new Error(t('checkin.oauthWizard.errors.providerRequired'))
      const cookies = parseCookies(credentials)
      await createCheckinAccount({
        provider_id: selectedProvider.id.replace('builtin-', ''),
        name: accountName || defaultAccountName,
        cookies_json: JSON.stringify(cookies),
        api_user: apiUser || '',
      })
      dispatch({ type: 'CREATE_SUCCESS' })
      onSuccess?.()
    } catch (error: unknown) {
      dispatch({
        type: 'CREATE_ERROR',
        message: error instanceof Error ? error.message : t('checkin.oauthWizard.errors.createFailed'),
      })
    }
  }, [accountName, apiUser, credentials, defaultAccountName, onSuccess, selectedProvider, t])

  const goBack = useCallback(() => dispatch({ type: 'BACK' }), [])
  const parsedCookieCount = useMemo(() => {
    try {
      return Object.keys(parseCookies(credentials)).length
    } catch {
      return 0
    }
  }, [credentials])

  return (
    <BaseModal
      modelValue={isOpen}
      title={t('checkin.actions.oauthLoginTitle')}
      size="lg"
      surface="solid"
      persistent={state.loading}
      onClose={handleClose}
      footer={
        <OAuthWizardFooter
          step={state.step}
          loading={state.loading}
          creating={state.creatingAccount}
          createSuccess={state.createSuccess}
          canStep0={Boolean(providerId && oauthType)}
          hasUrl={Boolean(state.authorizeUrl)}
          hasCredentials={Boolean(credentials.trim())}
          t={t}
          onBack={goBack}
          onClose={handleClose}
          onStep0={goToStep1}
          onStep1={goToCredentials}
          onStep2={goToConfirm}
          onCreate={createAccount}
        />
      }
    >
      <OAuthWizardBody
        state={state}
        form={form}
        oauthProviders={oauthProviders}
        selectedProvider={selectedProvider}
        defaultAccountName={defaultAccountName}
        parsedCookieCount={parsedCookieCount}
        t={t}
        onSelectProvider={selectProvider}
        onSelectOAuthType={selectOAuthType}
        onCopyUrl={copyUrl}
        onBackToSelection={goBack}
      />
    </BaseModal>
  )
}

function OAuthWizardFooter({
  step,
  loading,
  creating,
  createSuccess,
  canStep0,
  hasUrl,
  hasCredentials,
  t,
  onBack,
  onClose,
  onStep0,
  onStep1,
  onStep2,
  onCreate,
}: {
  step: number
  loading: boolean
  creating: boolean
  createSuccess: boolean
  canStep0: boolean
  hasUrl: boolean
  hasCredentials: boolean
  t: (key: string) => string
  onBack: () => void
  onClose: () => void
  onStep0: () => void
  onStep1: () => void
  onStep2: () => void
  onCreate: () => void
}) {
  return (
    <div className="oauth-wizard__footer">
      {step > 0 && !createSuccess ? (
        <button
          type="button"
          className="oauth-wizard__button oauth-wizard__button--ghost"
          disabled={loading || creating}
          onClick={onBack}
        >
          {t('common.previous')}
        </button>
      ) : (
        <div />
      )}
      <div className="oauth-wizard__footer-actions">
        <button type="button" className="oauth-wizard__button oauth-wizard__button--secondary" onClick={onClose}>
          {createSuccess ? t('common.close') : t('common.cancel')}
        </button>
        {step === 0 ? (
          <button
            type="button"
            disabled={!canStep0}
            className={buttonClass({ variant: 'primary', className: 'oauth-wizard__button oauth-wizard__button--primary' })}
            onClick={onStep0}
          >
            {t('checkin.actions.oauthLogin')}
          </button>
        ) : null}
        {step === 1 && hasUrl ? (
          <button type="button" className={buttonClass({ variant: 'primary', className: 'oauth-wizard__button oauth-wizard__button--primary' })} onClick={onStep1}>
            {t('common.next')}
          </button>
        ) : null}
        {step === 2 ? (
          <button
            type="button"
            disabled={!hasCredentials}
            className={buttonClass({ variant: 'primary', className: 'oauth-wizard__button oauth-wizard__button--primary' })}
            onClick={onStep2}
          >
            {t('common.next')}
          </button>
        ) : null}
        {step === 3 && !createSuccess ? (
          <button
            type="button"
            disabled={creating}
            className="oauth-wizard__button oauth-wizard__button--success"
            onClick={onCreate}
          >
            {t('common.confirm')}
          </button>
        ) : null}
      </div>
    </div>
  )
}
