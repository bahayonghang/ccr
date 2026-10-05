import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { getPricingList, removePricing, resetPricing, setPricing } from '@/api'
import { PageHeader, PageShell, buttonClass } from '@/ui'
import type { ModelPricing, SetPricingRequest } from '@/types'
import { logger } from '@/utils/logger'
import { hydrateUsageLocale, useUsageT } from '../translate'
import '../styles/pricing-view.css'

type PriceForm = {
  model: string
  input_price: number
  output_price: number
  cache_read_price: number | null
  cache_write_price: number | null
}

type RawPricingListResponse = {
  items?: Array<{ model?: string; pricing?: Partial<ModelPricing> }>
  pricings?: Partial<ModelPricing>[]
  models?: Record<string, Partial<ModelPricing>>
  default_pricing?: Partial<ModelPricing> | null
  total?: number
}

type PendingAction = { type: 'delete'; model: string } | { type: 'reset' }

const formatPrice = (value?: number | null) => (typeof value === 'number' ? `$${value.toFixed(4)}` : '—')

export function PricingView() {
  const t = useUsageT()
  const [pricingData, setPricingData] = useState<RawPricingListResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [statusMessage, setStatusMessage] = useState<string | null>(null)
  const [showForm, setShowForm] = useState(false)
  const [isEditing, setIsEditing] = useState(false)
  const [pendingAction, setPendingAction] = useState<PendingAction | null>(null)
  const { register, handleSubmit, reset } = useForm<PriceForm>({
    defaultValues: { model: '', input_price: 0, output_price: 0, cache_read_price: null, cache_write_price: null },
  })

  const loadData = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      setPricingData(await getPricingList<RawPricingListResponse>())
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : String(caught))
      logger.error('Failed to load legacy CCR pricing:', caught)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void hydrateUsageLocale()
    void loadData()
  }, [loadData])

  const items = useMemo(() => {
    if (!pricingData) return []
    const raw = Array.isArray(pricingData.items)
      ? pricingData.items.map((item) => ({
        model: item.model ?? item.pricing?.model ?? '',
        input_price: Number(item.pricing?.input_price ?? 0),
        output_price: Number(item.pricing?.output_price ?? 0),
        cache_read_price: item.pricing?.cache_read_price,
        cache_write_price: item.pricing?.cache_write_price,
      }))
      : []
    return raw.filter((item) => item.model.length > 0).sort((left, right) => left.model.localeCompare(right.model))
  }, [pricingData])

  const onSave = handleSubmit(async (values) => {
    const model = values.model.trim()
    if (!model) {
      setError(t('pricing.messages.modelRequired'))
      return
    }
    setSaving(true)
    try {
      const request: SetPricingRequest = {
        model,
        input_price: values.input_price,
        output_price: values.output_price,
        cache_read_price: values.cache_read_price ?? undefined,
        cache_write_price: values.cache_write_price ?? undefined,
      }
      await setPricing(request)
      await loadData()
      setShowForm(false)
      setStatusMessage(isEditing ? t('pricing.messages.updated', { model }) : t('pricing.messages.created', { model }))
    } catch (caught) {
      setError(t('pricing.messages.saveFailed', { error: String(caught) }))
    } finally {
      setSaving(false)
    }
  })

  const showAdd = useCallback(() => {
    reset()
    setIsEditing(false)
    setShowForm(true)
  }, [reset])
  const hideForm = useCallback(() => setShowForm(false), [])
  const clearPending = useCallback(() => setPendingAction(null), [])

  const confirmPending = useCallback(async () => {
    if (!pendingAction) return
    setSaving(true)
    try {
      if (pendingAction.type === 'delete') {
        await removePricing(pendingAction.model)
        setStatusMessage(t('pricing.messages.removed', { model: pendingAction.model }))
      } else {
        await resetPricing()
        setStatusMessage(t('pricing.messages.reset'))
      }
      setPendingAction(null)
      await loadData()
    } catch (caught) {
      setError(String(caught))
    } finally {
      setSaving(false)
    }
  }, [loadData, pendingAction, t])

  return (
    <PageShell
      className="pricing-view"
      header={(
        <PageHeader
          title={t('pricing.title')}
          description={t('pricing.subtitle')}
          status={<span className="pricing-badge">{t('pricing.legacyBadge')}</span>}
          actions={(
            <button type="button" disabled={loading} className={buttonClass({ variant: 'primary', className: 'pricing-button' })} onClick={loadData}>
              {t('pricing.actions.refresh')}
            </button>
          )}
        />
      )}
    >
      {statusMessage ? <div className="pricing-status pricing-status--success">{statusMessage}</div> : null}
      {error ? <div className="pricing-status pricing-status--error">{error}</div> : null}
      {loading && !pricingData ? <div className="pricing-loading">{t('pricing.states.loading')}</div> : (
        <main className="pricing-content">
          <section className="pricing-card pricing-card--models">
            <div className="pricing-section-heading">
              <h2>{t('pricing.models.title')}</h2>
              <button type="button" className={buttonClass({ variant: 'primary', className: 'pricing-button' })} onClick={showAdd}>
                {t('pricing.actions.add')}
              </button>
            </div>
            {items.map((pricing) => (
              <PricingRow
                key={pricing.model}
                pricing={pricing}
                editLabel={t('pricing.actions.edit')}
                removeLabel={t('pricing.actions.remove')}
                onEdit={reset}
                onShowForm={setShowForm}
                onEditing={setIsEditing}
                onDelete={setPendingAction}
              />
            ))}
          </section>
          {showForm ? (
            <section className="pricing-card">
              <form className="pricing-form" onSubmit={onSave}>
                <label className="pricing-field">
                  <span>{t('pricing.form.model')}</span>
                  <input className="pricing-input" disabled={isEditing || saving} required {...register('model')} />
                </label>
                <label className="pricing-field">
                  <span>{t('pricing.fields.input')}</span>
                  <input className="pricing-input" type="number" step="0.000001" min="0" {...register('input_price', { valueAsNumber: true })} />
                </label>
                <label className="pricing-field">
                  <span>{t('pricing.fields.output')}</span>
                  <input className="pricing-input" type="number" step="0.000001" min="0" {...register('output_price', { valueAsNumber: true })} />
                </label>
                <div className="pricing-form__actions">
                  <button type="submit" disabled={saving} className={buttonClass({ variant: 'primary', className: 'pricing-button' })}>
                    {saving ? t('pricing.actions.saving') : t('pricing.actions.save')}
                  </button>
                  <button type="button" className="pricing-button pricing-button--secondary" onClick={hideForm}>
                    {t('pricing.actions.cancel')}
                  </button>
                </div>
              </form>
            </section>
          ) : null}
          {pendingAction ? (
            <section className="pricing-confirm" role="dialog">
              <h2>{pendingAction.type === 'delete' ? t('pricing.confirm.deleteTitle', { model: pendingAction.model }) : t('pricing.confirm.resetTitle')}</h2>
              <button type="button" className="pricing-button pricing-button--danger" onClick={confirmPending}>
                {t('pricing.confirm.confirm')}
              </button>
              <button type="button" onClick={clearPending}>{t('pricing.actions.cancel')}</button>
            </section>
          ) : null}
        </main>
      )}
    </PageShell>
  )
}

const PricingRow = memo(function PricingRow({
  pricing,
  editLabel,
  removeLabel,
  onEdit,
  onShowForm,
  onEditing,
  onDelete,
}: {
  pricing: ModelPricing
  editLabel: string
  removeLabel: string
  onEdit: (values: PriceForm) => void
  onShowForm: (open: boolean) => void
  onEditing: (editing: boolean) => void
  onDelete: (action: PendingAction) => void
}) {
  const handleEdit = useCallback(() => {
    onEdit({
      model: pricing.model,
      input_price: pricing.input_price,
      output_price: pricing.output_price,
      cache_read_price: pricing.cache_read_price ?? null,
      cache_write_price: pricing.cache_write_price ?? null,
    })
    onEditing(true)
    onShowForm(true)
  }, [onEdit, onEditing, onShowForm, pricing])
  const handleDelete = useCallback(() => {
    onDelete({ type: 'delete', model: pricing.model })
  }, [onDelete, pricing.model])
  return (
    <article className="pricing-row">
      <h3>{pricing.model}</h3>
      <span>{formatPrice(pricing.input_price)}</span>
      <span>{formatPrice(pricing.output_price)}</span>
      <button type="button" onClick={handleEdit}>{editLabel}</button>
      <button type="button" onClick={handleDelete}>{removeLabel}</button>
    </article>
  )
})
