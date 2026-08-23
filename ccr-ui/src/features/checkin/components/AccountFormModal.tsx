import { forwardRef, useCallback, useImperativeHandle, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import {
  createCheckinAccount,
  updateCheckinAccount,
  getCheckinAccountCookies,
} from '@/api'
import { getErrorMessage } from '@/types/api'
import type { AccountInfo, BuiltinProvider, CheckinProvider } from '@/types/checkin'
import { logger } from '@/utils/logger'
import { BaseModal, SIcon } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'
import { sessionToCookiesJson } from '../lib/accountCookies'
import {
  emptyAccountForm,
  extraFromForm,
  parseCdkExtra,
  valuesFromAccount,
  valuesFromCookies,
} from '../lib/accountFormValues'
import { resolveBuiltinProvider } from '../lib/builtinProviderLookup'
import { checkinNotify } from '../lib/checkinNotify'
import { useCheckinLocale, useCheckinT } from '../hooks/useCheckinT'
import { AccountFormFields, type AccountFormValues } from './AccountFormFields'
import '../styles/form.css'

export interface AccountFormModalHandle {
  open: (account?: AccountInfo, options?: { focusSession?: boolean }) => Promise<void>
}

interface AccountFormModalProps {
  providers: CheckinProvider[]
  builtinProviders: BuiltinProvider[]
  onRefresh?: () => void
}

interface CookiesResponse {
  cookies_json: string
  api_user?: string | null
}

function AccountFormHeader({
  titleId,
  editing,
  providerLabel,
  requiresWaf,
  t,
}: {
  titleId: string
  editing: boolean
  providerLabel: string
  requiresWaf: boolean
  t: TranslateFunction
}) {
  return (
    <div className="checkin-accounts-tab__modal-header">
      <div className="checkin-accounts-tab__modal-header-copy">
        <p className="checkin-accounts-tab__modal-eyebrow">
          {editing ? t('checkin.accounts.modal.editEyebrow') : t('checkin.accounts.modal.createEyebrow')}
        </p>
        <h3 id={titleId} className="checkin-accounts-tab__modal-title">
          <SIcon name="Users" size="w-5 h-5" />
          {editing ? t('checkin.accounts.editAccount') : t('checkin.accounts.addAccount')}
        </h3>
        <p className="checkin-accounts-tab__modal-subtitle">
          {editing ? t('checkin.accounts.modal.editSubtitle') : t('checkin.accounts.modal.createSubtitle')}
        </p>
      </div>
      <div className="checkin-accounts-tab__modal-badge-row">
        <span className="checkin-accounts-tab__modal-badge checkin-badge-pill">{providerLabel}</span>
        {requiresWaf ? (
          <span className="checkin-accounts-tab__modal-badge checkin-badge-pill checkin-accounts-tab__modal-badge--warning">
            {t('checkin.accounts.modal.requiresWaf')}
          </span>
        ) : null}
      </div>
    </div>
  )
}

async function persistAccount(
  values: AccountFormValues,
  editingAccount: AccountInfo | null,
  t: TranslateFunction,
): Promise<boolean> {
  const cookiesJson = sessionToCookiesJson(values.session)
  const apiUser = values.api_user.trim()
  const extraConfig = extraFromForm(values)
  if (extraConfig === 'invalid-fuli') {
    checkinNotify.error(t('checkin.accounts.errors.invalidFuliCookies'))
    return false
  }
  if (extraConfig === 'invalid-b4u') {
    checkinNotify.error(t('checkin.accounts.errors.invalidB4uCookies'))
    return false
  }
  const extraConfigJson = Object.keys(extraConfig).length > 0 ? JSON.stringify(extraConfig) : '{}'
  if (!apiUser) {
    checkinNotify.error(t('checkin.accounts.errors.apiUserRequired'))
    return false
  }
  if (editingAccount) {
    const updateData: Record<string, unknown> = {
      name: values.name,
      api_user: apiUser,
      enabled: values.enabled,
      extra_config: extraConfigJson,
    }
    if (cookiesJson) updateData.cookies_json = cookiesJson
    await updateCheckinAccount(editingAccount.id, updateData)
    return true
  }
  if (!cookiesJson) {
    checkinNotify.error(t('checkin.accounts.errors.sessionRequired'))
    return false
  }
  await createCheckinAccount({
    provider_id: values.provider_id,
    name: values.name,
    cookies_json: cookiesJson,
    api_user: apiUser,
    extra_config: extraConfigJson,
  })
  return true
}

export const AccountFormModal = forwardRef<AccountFormModalHandle, AccountFormModalProps>(
  function AccountFormModal({ providers, builtinProviders, onRefresh }, ref) {
    const t = useCheckinT()
    const locale = useCheckinLocale()
    const isZh = locale.startsWith('zh')
    const [open, setOpen] = useState(false)
    const [editingAccount, setEditingAccount] = useState<AccountInfo | null>(null)
    const form = useForm<AccountFormValues>({
      defaultValues: emptyAccountForm(providers[0]?.id || ''),
    })
    const providerId = form.watch('provider_id')
    const selectedBuiltin = useMemo(() => {
      if (!providerId) return null
      const provider = providers.find((item) => item.id === providerId)
      if (!provider) return null
      return resolveBuiltinProvider(builtinProviders, provider) || null
    }, [builtinProviders, providerId, providers])
    const modalProviderLabel = providerId
      ? selectedBuiltin?.name || providers.find((item) => item.id === providerId)?.name || providerId
      : t('checkin.accounts.modal.providerPending')
    const close = useCallback(() => setOpen(false), [])

    const openEditor = useCallback(
      async (account?: AccountInfo, options?: { focusSession?: boolean }) => {
        setEditingAccount(account || null)
        if (!account) {
          form.reset(emptyAccountForm(providers[0]?.id || ''))
          setOpen(true)
          return
        }
        const existingExtra = parseCdkExtra(account.extra_config)
        try {
          const cookiesData = await getCheckinAccountCookies<CookiesResponse>(account.id)
          form.reset(
            valuesFromCookies({
              account,
              extra: existingExtra,
              cookiesJson: cookiesData.cookies_json,
              apiUser: cookiesData.api_user,
            }),
          )
        } catch (error: unknown) {
          logger.error('Failed to get cookies for check-in account', {
            account: { id: account.id, provider_id: account.provider_id, name: account.name },
            err: error,
          })
          form.reset(
            valuesFromAccount({
              account,
              extra: existingExtra,
              session: '',
              apiUser: account.api_user || '',
            }),
          )
        }
        setOpen(true)
        if (options?.focusSession) requestAnimationFrame(() => form.setFocus('session'))
      },
      [form, providers],
    )

    useImperativeHandle(ref, () => ({ open: openEditor }), [openEditor])

    const saveAccount = form.handleSubmit(async (values) => {
      try {
        const ok = await persistAccount(values, editingAccount, t)
        if (!ok) return
        setOpen(false)
        onRefresh?.()
      } catch (error: unknown) {
        checkinNotify.error(
          t('checkin.accounts.errors.saveFailed', {
            error: getErrorMessage(error, t('checkin.errors.unknown')),
          }),
        )
      }
    })

    const renderHeader = useCallback(
      ({ titleId }: { titleId: string }) => (
        <AccountFormHeader
          titleId={titleId}
          editing={Boolean(editingAccount)}
          providerLabel={modalProviderLabel}
          requiresWaf={Boolean(selectedBuiltin?.requires_waf_bypass)}
          t={t}
        />
      ),
      [editingAccount, modalProviderLabel, selectedBuiltin?.requires_waf_bypass, t],
    )

    return (
      <BaseModal
        modelValue={open}
        size="xl"
        surface="solid"
        contentClass="checkin-accounts-tab__account-modal"
        onUpdateModelValue={setOpen}
        header={renderHeader}
        footer={
          <div className="checkin-accounts-tab__modal-footer">
            <button
              type="button"
              className="checkin-accounts-tab__form-button checkin-accounts-tab__form-button--secondary"
              onClick={close}
            >
              {t('common.cancel')}
            </button>
            <button
              type="submit"
              form="checkin-account-form"
              className="checkin-accounts-tab__form-button checkin-accounts-tab__form-button--primary"
            >
              {editingAccount
                ? t('checkin.accounts.modal.saveChanges')
                : t('checkin.accounts.modal.createAccount')}
            </button>
          </div>
        }
      >
        <div className="checkin-accounts-tab__modal-body">
          <div className="checkin-accounts-tab__modal-intro">
            <span className="checkin-accounts-tab__modal-intro-pill checkin-badge-pill">
              {t('checkin.accounts.modal.introSession')}
            </span>
            <span className="checkin-accounts-tab__modal-intro-pill checkin-badge-pill">
              {t('checkin.accounts.modal.introApiUser')}
            </span>
            <span className="checkin-accounts-tab__modal-intro-pill checkin-badge-pill">
              {t('checkin.accounts.modal.introNoOverwrite')}
            </span>
          </div>
          <div className="checkin-accounts-tab__modal-scroll">
            <AccountFormFields
              form={form}
              providers={providers}
              editing={Boolean(editingAccount)}
              cdkType={selectedBuiltin?.cdk_config?.cdk_type}
              requiresWaf={Boolean(selectedBuiltin?.requires_waf_bypass)}
              wafProviderName={selectedBuiltin?.name}
              isZh={isZh}
              t={t}
              onSubmit={saveAccount}
            />
          </div>
        </div>
      </BaseModal>
    )
  },
)
