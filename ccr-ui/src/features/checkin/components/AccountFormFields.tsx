import type { FormEventHandler } from 'react'
import type { UseFormReturn } from 'react-hook-form'
import type { CheckinProvider } from '@/types/checkin'
import type { TranslateFunction } from '@/utils/tf'

export interface AccountFormValues {
  provider_id: string
  name: string
  session: string
  api_user: string
  enabled: boolean
  fuli_cookies: string
  b4u_cdk_cookies: string
  x666_access_token: string
}

interface AccountFormFieldsProps {
  form: UseFormReturn<AccountFormValues>
  providers: CheckinProvider[]
  editing: boolean
  cdkType?: string
  requiresWaf: boolean
  wafProviderName?: string
  isZh: boolean
  t: TranslateFunction
  onSubmit: FormEventHandler<HTMLFormElement>
}

export function AccountFormFields({
  form,
  providers,
  editing,
  cdkType,
  requiresWaf,
  wafProviderName,
  isZh,
  t,
  onSubmit,
}: AccountFormFieldsProps) {
  const { register } = form
  const sessionLabel = isZh ? 'Session / Cookies' : 'Session / Cookies'
  const apiUserLabel = isZh ? 'API 用户' : 'API User'

  return (
    <form id="checkin-account-form" className="checkin-accounts-tab__form" onSubmit={onSubmit}>
      <section className="checkin-accounts-tab__form-section checkin-accounts-tab__form-section--identity">
        <div className="checkin-accounts-tab__form-grid">
          <div className="checkin-accounts-tab__field">
            <label className="checkin-accounts-tab__label">
              <span className="text-accent-danger">*</span> {t('checkin.accounts.fields.provider')}
            </label>
            <select
              className="checkin-accounts-tab__control"
              required
              disabled={editing}
              {...register('provider_id')}
            >
              <option value="">{t('checkin.accounts.fields.selectProvider')}</option>
              {providers.map((provider) => (
                <option key={provider.id} value={provider.id}>
                  {provider.name}
                </option>
              ))}
            </select>
          </div>
          <div className="checkin-accounts-tab__field">
            <label className="checkin-accounts-tab__label">
              <span className="text-accent-danger">*</span> {t('checkin.accounts.fields.accountName')}
            </label>
            <input
              type="text"
              required
              className="checkin-accounts-tab__control"
              placeholder={t('checkin.accounts.fields.accountNamePlaceholder')}
              {...register('name')}
            />
          </div>
        </div>
      </section>

      <section className="checkin-accounts-tab__form-section checkin-accounts-tab__form-section--credentials">
        <div className="checkin-accounts-tab__field checkin-accounts-tab__field--credential">
          <label className="checkin-accounts-tab__label">
            {!editing ? <span className="text-accent-danger">*</span> : null} {sessionLabel}
            {editing ? (
              <span className="font-normal text-text-muted">{t('checkin.accounts.fields.leaveBlank')}</span>
            ) : null}
          </label>
          <textarea
            rows={7}
            required={!editing}
            className="checkin-accounts-tab__control checkin-accounts-tab__control--textarea checkin-accounts-tab__control--mono checkin-accounts-tab__control--credential"
            placeholder={t('checkin.accounts.fields.sessionPlaceholder')}
            {...register('session')}
          />
          <p className="checkin-accounts-tab__hint checkin-accounts-tab__hint--with-icon">
            {t('checkin.accounts.fields.sessionHint')}
          </p>
        </div>
        <div className="checkin-accounts-tab__field">
          <label className="checkin-accounts-tab__label">
            <span className="text-accent-danger">*</span> {apiUserLabel}
          </label>
          <input
            type="text"
            required
            className="checkin-accounts-tab__control checkin-accounts-tab__control--mono"
            placeholder="12345"
            {...register('api_user')}
          />
          <p className="checkin-accounts-tab__hint">
            {t('checkin.accounts.fields.apiUserHintPrefix')}
            <code>user.id</code>
            {t('checkin.accounts.fields.apiUserHintMiddle')}
            <code>new-api-user</code>
            {t('checkin.accounts.fields.apiUserHintSuffix')}
          </p>
        </div>
      </section>

      {requiresWaf ? (
        <div className="checkin-accounts-tab__notice checkin-accounts-tab__notice--warning">
          <p className="checkin-accounts-tab__notice-title checkin-accounts-tab__notice-title--warning">
            {t('checkin.accounts.waf.title', { provider: wafProviderName ?? '' })}
          </p>
          <ol className="checkin-accounts-tab__notice-list">
            <li>{t('checkin.accounts.waf.stepSave')}</li>
            <li>{t('checkin.accounts.waf.stepProviders', { provider: wafProviderName ?? '' })}</li>
            <li>{t('checkin.accounts.waf.stepProxy')}</li>
          </ol>
        </div>
      ) : null}

      {cdkType === 'runawaytime' ? (
        <div className="checkin-accounts-tab__field">
          <label className="checkin-accounts-tab__label checkin-accounts-tab__label--amber">
            fuli.hxi.me Cookies
          </label>
          <textarea
            rows={3}
            className="checkin-accounts-tab__control checkin-accounts-tab__control--amber checkin-accounts-tab__control--textarea checkin-accounts-tab__control--mono"
            {...register('fuli_cookies')}
          />
        </div>
      ) : null}
      {cdkType === 'b4u' ? (
        <div className="checkin-accounts-tab__field">
          <label className="checkin-accounts-tab__label checkin-accounts-tab__label--amber">
            tw.b4u.qzz.io Cookies
          </label>
          <textarea
            rows={3}
            className="checkin-accounts-tab__control checkin-accounts-tab__control--amber checkin-accounts-tab__control--textarea checkin-accounts-tab__control--mono"
            {...register('b4u_cdk_cookies')}
          />
        </div>
      ) : null}
      {cdkType === 'x666' ? (
        <div className="checkin-accounts-tab__field">
          <label className="checkin-accounts-tab__label checkin-accounts-tab__label--amber">
            {isZh ? '访问令牌 (JWT)' : 'Access Token (JWT)'}
          </label>
          <input
            type="text"
            className="checkin-accounts-tab__control checkin-accounts-tab__control--amber checkin-accounts-tab__control--mono"
            {...register('x666_access_token')}
          />
        </div>
      ) : null}

      <div className="checkin-accounts-tab__toggle">
        <input
          id="account-enabled"
          type="checkbox"
          className="checkin-accounts-tab__checkbox"
          {...register('enabled')}
        />
        <label htmlFor="account-enabled" className="checkin-accounts-tab__checkbox-label">
          {t('checkin.accounts.fields.enabled')}
        </label>
      </div>
    </form>
  )
}
