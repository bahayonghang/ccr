import { useCallback } from 'react'
import type { UseFormReturn } from 'react-hook-form'
import type { BuiltinProvider } from '@/types/checkin'
import type { TranslateFunction } from '@/utils/tf'
import { SIcon } from '@/ui'
import type { OAuthType, OAuthWizardState } from '../lib/oauthWizardReducer'

interface OAuthFormValues {
  provider_id: string
  oauth_type: OAuthType
  credentials: string
  api_user: string
  account_name: string
}

interface OAuthWizardBodyProps {
  state: OAuthWizardState
  form: UseFormReturn<OAuthFormValues>
  oauthProviders: BuiltinProvider[]
  selectedProvider?: BuiltinProvider
  defaultAccountName: string
  parsedCookieCount: number
  t: TranslateFunction
  onSelectProvider: (id: string) => void
  onSelectOAuthType: (type: OAuthType) => void
  onCopyUrl: () => void
  onBackToSelection: () => void
}

const STEP_KEYS = [
  'checkin.oauthWizard.steps.selectMethod',
  'checkin.oauthWizard.steps.getLink',
  'checkin.oauthWizard.steps.pasteCredentials',
  'checkin.oauthWizard.steps.confirmCreate',
] as const

export function OAuthWizardBody({
  state,
  form,
  oauthProviders,
  selectedProvider,
  defaultAccountName,
  parsedCookieCount,
  t,
  onSelectProvider,
  onSelectOAuthType,
  onCopyUrl,
  onBackToSelection,
}: OAuthWizardBodyProps) {
  const { register } = form

  return (
    <>
      <div className="oauth-wizard__steps">
        {STEP_KEYS.map((key, idx) => (
          <OAuthStepMark
            key={key}
            index={idx}
            current={state.step}
            label={t(key)}
            showDivider={idx < STEP_KEYS.length - 1}
          />
        ))}
      </div>
      {state.step === 0 ? (
        <OAuthStepZero
          providerId={form.watch('provider_id')}
          oauthType={form.watch('oauth_type')}
          oauthProviders={oauthProviders}
          selectedProvider={selectedProvider}
          t={t}
          onSelectProvider={onSelectProvider}
          onSelectOAuthType={onSelectOAuthType}
        />
      ) : null}

      {state.step === 1 ? (
        <OAuthLinkStep
          loading={state.loading}
          error={state.oauthError}
          authorizeUrl={state.authorizeUrl}
          guide={state.extractionGuide}
          copied={state.copied}
          t={t}
          onCopyUrl={onCopyUrl}
          onBackToSelection={onBackToSelection}
        />
      ) : null}

      {state.step === 2 ? (
        <div className="oauth-wizard__section">
          <div>
            <label className="oauth-wizard__label">{t('checkin.oauthWizard.credentialsLabel')}</label>
            <textarea
              rows={6}
              placeholder={t('checkin.oauthWizard.credentialsPlaceholder')}
              className="oauth-wizard__input oauth-wizard__input--mono"
              {...register('credentials')}
            />
          </div>
          <div>
            <label className="oauth-wizard__label">{t('checkin.oauthWizard.apiUserLabel')}</label>
            <input
              className="oauth-wizard__input"
              placeholder={t('checkin.oauthWizard.apiUserPlaceholder')}
              {...register('api_user')}
            />
          </div>
          <div>
            <label className="oauth-wizard__label">{t('checkin.oauthWizard.accountNameLabel')}</label>
            <input
              className="oauth-wizard__input"
              placeholder={defaultAccountName}
              {...register('account_name')}
            />
          </div>
          {state.parseError ? (
            <div className="oauth-wizard__panel oauth-wizard__panel--error">
              <p className="oauth-wizard__error-text">{state.parseError}</p>
            </div>
          ) : null}
        </div>
      ) : null}

      {state.step === 3 ? (
        <OAuthConfirmStep
          creating={state.creatingAccount}
          success={state.createSuccess}
          error={state.createError}
          providerName={selectedProvider?.name}
          accountName={form.watch('account_name') || defaultAccountName}
          cookieCount={parsedCookieCount}
          apiUser={form.watch('api_user')}
          t={t}
        />
      ) : null}
    </>
  )
}

function OAuthStepZero({
  providerId,
  oauthType,
  oauthProviders,
  selectedProvider,
  t,
  onSelectProvider,
  onSelectOAuthType,
}: {
  providerId: string
  oauthType: OAuthType
  oauthProviders: BuiltinProvider[]
  selectedProvider?: BuiltinProvider
  t: TranslateFunction
  onSelectProvider: (id: string) => void
  onSelectOAuthType: (type: OAuthType) => void
}) {
  const handleProviderChange = useCallback(
    (event: { target: { value: string } }) => {
      onSelectProvider(event.target.value)
    },
    [onSelectProvider],
  )
  const onLinuxDo = useCallback(() => {
    onSelectOAuthType('linuxdo')
  }, [onSelectOAuthType])
  const onGithub = useCallback(() => {
    onSelectOAuthType('github')
  }, [onSelectOAuthType])
  const linuxClass =
    oauthType === 'linuxdo' ? 'oauth-wizard__choice oauth-wizard__choice--active' : 'oauth-wizard__choice oauth-wizard__choice--inactive'
  const githubClass =
    oauthType === 'github' ? 'oauth-wizard__choice oauth-wizard__choice--active' : 'oauth-wizard__choice oauth-wizard__choice--inactive'
  return (
    <div className="oauth-wizard__section">
      <div>
        <label className="oauth-wizard__label">{t('checkin.oauthWizard.providerLabel')}</label>
        <select className="oauth-wizard__input" value={providerId} onChange={handleProviderChange}>
          <option value="" disabled>
            {t('common.selectOption')}
          </option>
          {oauthProviders.map((provider) => (
            <option key={provider.id} value={provider.id}>
              {provider.icon} {provider.name} ({provider.domain})
            </option>
          ))}
        </select>
      </div>
      {selectedProvider ? (
        <div>
          <label className="oauth-wizard__label">{t('checkin.oauthWizard.loginMethodLabel')}</label>
          <div className="oauth-wizard__choice-grid">
            {selectedProvider.oauth_config?.linuxdo_client_id ? (
              <button type="button" className={linuxClass} onClick={onLinuxDo}>
                <SIcon name="Globe" size="w-5 h-5" />
                {t('checkin.oauthWizard.oauthTypes.linuxdo')}
              </button>
            ) : null}
            {selectedProvider.oauth_config?.github_client_id ? (
              <button type="button" className={githubClass} onClick={onGithub}>
                <SIcon name="Github" size="w-5 h-5" />
                {t('checkin.oauthWizard.oauthTypes.github')}
              </button>
            ) : null}
          </div>
        </div>
      ) : null}
    </div>
  )
}

function OAuthStepMark({
  index,
  current,
  label,
  showDivider,
}: {
  index: number
  current: number
  label: string
  showDivider: boolean
}) {
  const tone =
    current > index ? 'complete' : current === index ? 'current' : 'inactive'
  return (
    <>
      <div className={`oauth-wizard__step oauth-wizard__step--${tone}`}>
        <div className={`oauth-wizard__step-circle oauth-wizard__step-circle--${tone}`}>
          {current > index ? <SIcon name="CheckCircle" /> : <span>{index + 1}</span>}
        </div>
        <span className="oauth-wizard__step-label">{label}</span>
      </div>
      {showDivider ? <SIcon name="ChevronRight" size="w-4 h-4" /> : null}
    </>
  )
}

function OAuthLinkStep({
  loading,
  error,
  authorizeUrl,
  guide,
  copied,
  t,
  onCopyUrl,
  onBackToSelection,
}: {
  loading: boolean
  error: string
  authorizeUrl: string
  guide: string[]
  copied: boolean
  t: TranslateFunction
  onCopyUrl: () => void
  onBackToSelection: () => void
}) {
  if (loading) {
    return (
      <div className="oauth-wizard__state">
        <SIcon name="Loader2" size="w-8 h-8" className="animate-spin" />
        <p>{t('checkin.oauthWizard.loadingAuthorizeUrl')}</p>
      </div>
    )
  }
  if (error) {
    return (
      <div className="oauth-wizard__panel oauth-wizard__panel--error">
        <p className="oauth-wizard__error-text">{error}</p>
        <button type="button" className="oauth-wizard__button oauth-wizard__button--ghost" onClick={onBackToSelection}>
          {t('checkin.oauthWizard.backToSelection')}
        </button>
      </div>
    )
  }
  if (!authorizeUrl) return null
  return (
    <div className="oauth-wizard__section">
      <div className="oauth-wizard__panel oauth-wizard__panel--info">
        <p>{t('checkin.oauthWizard.openInBrowserHint')}</p>
        <div className="oauth-wizard__url-row">
          <input value={authorizeUrl} readOnly className="oauth-wizard__url-input" />
          <button type="button" className="oauth-wizard__button oauth-wizard__button--primary" onClick={onCopyUrl}>
            {copied ? t('checkin.oauthWizard.copied') : t('checkin.oauthWizard.copy')}
          </button>
        </div>
      </div>
      <div className="oauth-wizard__panel">
        <p>{t('checkin.oauthWizard.guideTitle')}</p>
        <ol>
          {guide.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ol>
      </div>
    </div>
  )
}

function OAuthConfirmStep({
  creating,
  success,
  error,
  providerName,
  accountName,
  cookieCount,
  apiUser,
  t,
}: {
  creating: boolean
  success: boolean
  error: string
  providerName?: string
  accountName: string
  cookieCount: number
  apiUser: string
  t: TranslateFunction
}) {
  if (creating) {
    return (
      <div className="oauth-wizard__state">
        <SIcon name="Loader2" size="w-8 h-8" className="animate-spin" />
        <p>{t('checkin.oauthWizard.creatingAccount')}</p>
      </div>
    )
  }
  if (success) {
    return (
      <div className="oauth-wizard__state">
        <SIcon name="CheckCircle" size="w-12 h-12" />
        <p>{t('checkin.oauthWizard.createSuccess')}</p>
        <p>
          {providerName} - {accountName}
        </p>
      </div>
    )
  }
  return (
    <div className="oauth-wizard__section">
      <div className="oauth-wizard__panel">
        <div className="oauth-wizard__summary-row">
          <span>{t('checkin.providers.provider')}</span>
          <span>{providerName}</span>
        </div>
        <div className="oauth-wizard__summary-row">
          <span>{t('checkin.oauthWizard.summary.accountName')}</span>
          <span>{accountName}</span>
        </div>
        <div className="oauth-wizard__summary-row">
          <span>{t('checkin.oauthWizard.summary.cookieCount')}</span>
          <span>{t('checkin.oauthWizard.summary.cookieCountValue', { count: cookieCount })}</span>
        </div>
        <div className="oauth-wizard__summary-row">
          <span>{t('checkin.oauthWizard.summary.apiUser')}</span>
          <span>{apiUser || t('checkin.oauthWizard.unsetValue')}</span>
        </div>
      </div>
      {error ? (
        <div className="oauth-wizard__panel oauth-wizard__panel--error">
          <p className="oauth-wizard__error-text">{error}</p>
        </div>
      ) : null}
    </div>
  )
}
