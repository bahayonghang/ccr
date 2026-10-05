import { memo, useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import {
  createCheckinProvider,
  updateCheckinProvider,
  deleteCheckinProvider as apiDeleteProvider,
  openWafLogin,
  getWafCookieStatus,
} from '@/api'
import { getErrorMessage } from '@/types/api'
import type {
  BuiltinProvider,
  CheckinProvider,
  WafCookieRecoveryResult,
  WafCookieStatus,
} from '@/types/checkin'
import { logger } from '@/utils/logger'
import { BaseModal, SIcon, buttonClass } from '@/ui'
import {
  filterAvailableBuiltinProviders,
  resolveBuiltinProvider,
} from '../lib/builtinProviderLookup'
import { getProviderLoginUrl } from '../lib/checkinFormat'
import { checkinNotify } from '../lib/checkinNotify'
import { formatWafCookieRecoveryFailure } from '../lib/wafFormat'
import { useCheckinT } from '../hooks/useCheckinT'
import '../styles/providers.css'

interface CheckinProvidersTabProps {
  providers: CheckinProvider[]
  builtinProviders: BuiltinProvider[]
  onAddBuiltin?: (id: string) => void
  onRefresh?: () => void
}

interface ProviderFormValues {
  name: string
  base_url: string
  checkin_path: string
  balance_path: string
  user_info_path: string
  auth_header: string
  auth_prefix: string
}

const emptyProviderForm = (): ProviderFormValues => ({
  name: '',
  base_url: '',
  checkin_path: '/api/user/checkin',
  balance_path: '/api/user/self',
  user_info_path: '/api/user/self',
  auth_header: 'Authorization',
  auth_prefix: 'Bearer ',
})

const BuiltinCard = memo(function BuiltinCard({
  provider,
  onAdd,
  addLabel,
  builtInLabel,
}: {
  provider: BuiltinProvider
  onAdd: (id: string) => void
  addLabel: string
  builtInLabel: string
}) {
  const handleAdd = useCallback(() => {
    onAdd(provider.id)
  }, [onAdd, provider.id])
  return (
    <div className="checkin-providers__builtin-card">
      <div className="checkin-providers__builtin-card-header">
        <div className="checkin-providers__builtin-card-main">
          <span>{provider.icon}</span>
          <div>
            <h3>{provider.name}</h3>
            <span className="checkin-providers__builtin-badge checkin-badge-pill">{builtInLabel}</span>
            <p>{provider.domain}</p>
          </div>
        </div>
        <button type="button" className={buttonClass({ variant: 'primary', className: 'checkin-providers__primary-button' })} onClick={handleAdd}>
          {addLabel}
        </button>
      </div>
      <p>{provider.description}</p>
    </div>
  )
})

export function CheckinProvidersTab({
  providers,
  builtinProviders,
  onAddBuiltin,
  onRefresh,
}: CheckinProvidersTabProps) {
  const t = useCheckinT()
  const available = filterAvailableBuiltinProviders(builtinProviders, providers)
  const [wafStatusMap, setWafStatusMap] = useState<Record<string, WafCookieStatus | undefined>>({})
  const [wafLoadingMap, setWafLoadingMap] = useState<Record<string, boolean>>({})
  const [showModal, setShowModal] = useState(false)
  const [editing, setEditing] = useState<CheckinProvider | null>(null)
  const form = useForm<ProviderFormValues>({ defaultValues: emptyProviderForm() })

  const loadWafStatus = useCallback(async (providerId: string) => {
    try {
      const status = await getWafCookieStatus<WafCookieStatus>(providerId)
      setWafStatusMap((current) => ({ ...current, [providerId]: status }))
    } catch (error: unknown) {
      logger.warn('Failed to load WAF status', error)
    }
  }, [])

  useEffect(() => {
    const wafProviders = providers.filter(
      (provider) => resolveBuiltinProvider(builtinProviders, provider)?.requires_waf_bypass === true,
    )
    wafProviders.forEach((provider) => {
      void loadWafStatus(provider.id)
    })
  }, [builtinProviders, loadWafStatus, providers])

  const openCreate = useCallback(() => {
    setEditing(null)
    form.reset(emptyProviderForm())
    setShowModal(true)
  }, [form])

  const openEdit = useCallback(
    (provider: CheckinProvider) => {
      setEditing(provider)
      form.reset({
        name: provider.name,
        base_url: provider.base_url,
        checkin_path: provider.checkin_path,
        balance_path: provider.balance_path,
        user_info_path: provider.user_info_path,
        auth_header: provider.auth_header,
        auth_prefix: provider.auth_prefix,
      })
      setShowModal(true)
    },
    [form],
  )

  const closeModal = useCallback(() => setShowModal(false), [])

  const saveProvider = form.handleSubmit(async (values) => {
    try {
      if (editing) await updateCheckinProvider(editing.id, values)
      else await createCheckinProvider(values)
      setShowModal(false)
      checkinNotify.success(editing ? '提供商已更新' : '提供商已添加')
      onRefresh?.()
    } catch (error: unknown) {
      checkinNotify.error('保存失败: ' + getErrorMessage(error, '未知错误'))
    }
  })

  const deleteProvider = useCallback(
    async (id: string) => {
      const confirmed = await checkinNotify.confirm({
        title: '删除提供商',
        message: '确定要删除此提供商吗？相关账号也会被删除。',
        confirmText: '删除',
        cancelText: '取消',
        type: 'danger',
        surface: 'solid',
      })
      if (!confirmed) return
      try {
        await apiDeleteProvider(id)
        checkinNotify.success('提供商已删除')
        onRefresh?.()
      } catch (error: unknown) {
        checkinNotify.error('删除失败: ' + getErrorMessage(error, '未知错误'))
      }
    },
    [onRefresh],
  )

  const startWafLogin = useCallback(
    async (provider: CheckinProvider) => {
      setWafLoadingMap((current) => ({ ...current, [provider.id]: true }))
      try {
        const result = await openWafLogin<WafCookieRecoveryResult>(
          getProviderLoginUrl(provider),
          provider.id,
        )
        await loadWafStatus(provider.id)
        if (result.persisted) {
          checkinNotify.success(`${provider.name} ${t('checkin.providers.cachedCookie')}`)
        } else {
          checkinNotify.error(`获取 WAF Cookie 失败: ${formatWafCookieRecoveryFailure(result)}`)
        }
      } catch (error: unknown) {
        checkinNotify.error('获取 WAF Cookie 失败: ' + getErrorMessage(error, '未知错误'))
      } finally {
        setWafLoadingMap((current) => ({ ...current, [provider.id]: false }))
      }
    },
    [loadWafStatus, t],
  )

  const handleAddBuiltin = useCallback(
    (id: string) => {
      onAddBuiltin?.(id)
    },
    [onAddBuiltin],
  )

  return (
    <div className="checkin-providers">
      {available.length > 0 ? (
        <div>
          <div className="checkin-providers__section-header">
            <h2 className="checkin-providers__section-title">{t('checkin.providers.builtinTitle')}</h2>
            <span>({available.length})</span>
          </div>
          <div className="checkin-providers__builtin-grid">
            {available.map((provider) => (
              <BuiltinCard
                key={provider.id}
                provider={provider}
                onAdd={handleAddBuiltin}
                addLabel={t('common.add')}
                builtInLabel={t('checkin.providers.builtInBadge')}
              />
            ))}
          </div>
        </div>
      ) : null}

      <div>
        <div className="checkin-providers__section-header">
          <h2 className="checkin-providers__section-title">{t('checkin.providers.addedTitle')}</h2>
          <button type="button" className={buttonClass({ variant: 'primary', className: 'checkin-providers__primary-button' })} onClick={openCreate}>
            {t('checkin.providers.customAdd')}
          </button>
        </div>
        {providers.length === 0 ? (
          <div className="checkin-providers__empty-state">{t('checkin.providers.emptyTitle')}</div>
        ) : (
          <div className="checkin-providers__provider-grid">
            {providers.map((provider) => (
              <AddedProviderCard
                key={provider.id}
                provider={provider}
                requiresWaf={
                  resolveBuiltinProvider(builtinProviders, provider)?.requires_waf_bypass === true
                }
                hasCookie={wafStatusMap[provider.id]?.has_cookie === true}
                loading={wafLoadingMap[provider.id] === true}
                tLabel={t}
                onEdit={openEdit}
                onDelete={deleteProvider}
                onWaf={startWafLogin}
              />
            ))}
          </div>
        )}
      </div>

      <BaseModal modelValue={showModal} title={editing ? t('checkin.providers.editProvider') : t('checkin.providers.addProvider')} surface="solid" onUpdateModelValue={setShowModal}>
        <form className="checkin-providers__modal-panel" onSubmit={saveProvider}>
          <label className="checkin-providers__field-label">{t('checkin.providers.nameLabel')}</label>
          <input className="checkin-providers__field-input" required {...form.register('name')} />
          <label className="checkin-providers__field-label">{t('checkin.providers.baseUrlLabel')}</label>
          <input className="checkin-providers__field-input" type="url" required {...form.register('base_url')} />
          <div className="checkin-providers__field-grid">
            <div>
              <label>{t('checkin.providers.checkinPathLabel')}</label>
              <input className="checkin-providers__field-input" {...form.register('checkin_path')} />
            </div>
            <div>
              <label>{t('checkin.providers.balancePathLabel')}</label>
              <input className="checkin-providers__field-input" {...form.register('balance_path')} />
            </div>
          </div>
          <div className="checkin-providers__modal-actions">
            <button type="button" onClick={closeModal}>
              {t('common.cancel')}
            </button>
            <button type="submit" className={buttonClass({ variant: 'primary', className: 'checkin-providers__primary-button' })}>
              {t('common.save')}
            </button>
          </div>
        </form>
      </BaseModal>
    </div>
  )
}

const AddedProviderCard = memo(function AddedProviderCard({
  provider,
  requiresWaf,
  hasCookie,
  loading,
  tLabel,
  onEdit,
  onDelete,
  onWaf,
}: {
  provider: CheckinProvider
  requiresWaf: boolean
  hasCookie: boolean
  loading: boolean
  tLabel: (key: string) => string
  onEdit: (provider: CheckinProvider) => void
  onDelete: (id: string) => void
  onWaf: (provider: CheckinProvider) => void
}) {
  const handleEdit = useCallback(() => onEdit(provider), [onEdit, provider])
  const handleDelete = useCallback(() => onDelete(provider.id), [onDelete, provider.id])
  const handleWaf = useCallback(() => onWaf(provider), [onWaf, provider])
  return (
    <div
      className={`checkin-providers__provider-card ${
        provider.enabled
          ? 'checkin-providers__provider-card--enabled'
          : 'checkin-providers__provider-card--disabled'
      }`}
    >
      <div className="checkin-providers__provider-card-header">
        <div>
          <h3>{provider.name}</h3>
          <p>{provider.base_url}</p>
        </div>
        <div>
          <button type="button" onClick={handleEdit} title={tLabel('common.edit')}>
            {tLabel('common.edit')}
          </button>
          <button type="button" onClick={handleDelete} title={tLabel('common.delete')}>
            {tLabel('common.delete')}
          </button>
        </div>
      </div>
      {requiresWaf ? (
        <div className="checkin-providers__waf-card">
          <p>{tLabel('checkin.providers.wafTitle')}</p>
          <span className={`checkin-providers__tag checkin-badge-pill ${hasCookie ? 'checkin-providers__tag--success' : 'checkin-providers__tag--warning'}`}>
            {hasCookie ? tLabel('checkin.providers.cachedCookie') : tLabel('checkin.providers.uncachedCookie')}
          </span>
          <button
            type="button"
            className="checkin-providers__waf-action"
            disabled={loading}
            onClick={handleWaf}
          >
            <SIcon name="RefreshCw" size="w-3.5 h-3.5" className={loading ? 'animate-spin' : undefined} />
            {loading
              ? tLabel('checkin.providers.loading')
              : hasCookie
                ? tLabel('checkin.providers.reloadCookie')
                : tLabel('checkin.providers.getCookie')}
          </button>
        </div>
      ) : null}
    </div>
  )
})
