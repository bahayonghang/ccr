import { useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useSetClaudeObserverSubscription } from '@/features/claude/queries'
import { t } from '@/features/claude/locale'
import { BaseModal } from '@/ui'
import { getErrorMessage } from '@/utils/errorHandler'
import type { SubscriptionDto } from '@/types/claudeObserver'

interface SubscriptionForm {
  mode: string
  plan: string
  monthlyUsd: number
}

interface SubscriptionDialogProps {
  modelValue: boolean
  current: SubscriptionDto | null
  onClose: () => void
}

const MODE_OPTIONS = [
  { value: 'auto', labelKey: 'claudeCode.observer.subscription.modeAuto' },
  { value: 'api_key', labelKey: 'claudeCode.observer.subscription.modeApiKey' },
  { value: 'subscription', labelKey: 'claudeCode.observer.subscription.modeSubscription' },
] as const

const PLAN_OPTIONS = [
  { value: 'free_pro', labelKey: 'claudeCode.observer.subscription.planFreePro' },
  { value: 'max5x', labelKey: 'claudeCode.observer.subscription.planMax5x' },
  { value: 'max20x', labelKey: 'claudeCode.observer.subscription.planMax20x' },
  { value: 'team', labelKey: 'claudeCode.observer.subscription.planTeam' },
  { value: 'enterprise', labelKey: 'claudeCode.observer.subscription.planEnterprise' },
  { value: 'custom', labelKey: 'claudeCode.observer.subscription.planCustom' },
] as const

const fieldClass =
  'w-full rounded-xl border border-border-default bg-[var(--surface-card-bg)] px-3 py-2 text-sm text-text-primary'

export function SubscriptionDialog({ modelValue, current, onClose }: SubscriptionDialogProps) {
  const mutation = useSetClaudeObserverSubscription()
  const [error, setError] = useState<string | null>(null)
  const form = useForm<SubscriptionForm>({
    defaultValues: { mode: 'auto', plan: 'free_pro', monthlyUsd: 0 },
  })
  const { register, handleSubmit, reset, watch } = form
  const mode = watch('mode')
  const subscriptionDisabled = mode !== 'subscription'

  useEffect(() => {
    if (!modelValue) return
    reset({
      mode: current?.mode ?? 'auto',
      plan: current?.plan ?? 'free_pro',
      monthlyUsd: current?.monthly_usd ?? 0,
    })
    setError(null)
  }, [current, modelValue, reset])

  const onValid = useCallback(
    async (values: SubscriptionForm) => {
      setError(null)
      try {
        await mutation.mutateAsync({
          mode: values.mode,
          plan: values.plan,
          monthlyUsd: Number.isFinite(values.monthlyUsd) ? values.monthlyUsd : 0,
        })
        onClose()
      } catch (err) {
        setError(getErrorMessage(err))
      }
    },
    [mutation, onClose],
  )
  const onSubmit = useMemo(() => handleSubmit(onValid), [handleSubmit, onValid])
  const handleOpenChange = useCallback(
    (open: boolean) => {
      if (!open) onClose()
    },
    [onClose],
  )

  return (
    <BaseModal
      modelValue={modelValue}
      title={t('claudeCode.observer.subscription.dialogTitle')}
      size="sm"
      onUpdateModelValue={handleOpenChange}
      onClose={onClose}
      footer={
        <div className="flex justify-end gap-2">
          <button
            type="button"
            className="rounded-xl border border-border-default px-4 py-2 text-sm font-semibold text-text-secondary"
            disabled={mutation.isPending}
            onClick={onClose}
          >
            {t('common.cancel')}
          </button>
          <button
            type="button"
            className="rounded-xl bg-accent-primary px-4 py-2 text-sm font-semibold text-[color:var(--color-accent-primary-contrast)]"
            disabled={mutation.isPending}
            onClick={onSubmit}
          >
            {mutation.isPending ? t('claudeCode.observer.subscription.saving') : t('common.save')}
          </button>
        </div>
      }
    >
      <div className="grid gap-3.5">
        <label className="grid gap-1">
          <span className="text-xs font-semibold tracking-wide text-text-secondary">
            {t('claudeCode.observer.subscription.fieldMode')}
          </span>
          <select className={fieldClass} {...register('mode')}>
            {MODE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {t(option.labelKey)}
              </option>
            ))}
          </select>
        </label>
        <label className="grid gap-1">
          <span className="text-xs font-semibold tracking-wide text-text-secondary">
            {t('claudeCode.observer.subscription.fieldPlan')}
          </span>
          <select className={fieldClass} disabled={subscriptionDisabled} {...register('plan')}>
            {PLAN_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {t(option.labelKey)}
              </option>
            ))}
          </select>
        </label>
        <label className="grid gap-1">
          <span className="text-xs font-semibold tracking-wide text-text-secondary">
            {t('claudeCode.observer.subscription.fieldMonthlyUsd')}
          </span>
          <input
            type="number"
            min={0}
            step={1}
            className={fieldClass}
            disabled={subscriptionDisabled}
            {...register('monthlyUsd', { valueAsNumber: true })}
          />
        </label>
        {error ? <p className="m-0 text-xs text-accent-danger">{error}</p> : null}
      </div>
    </BaseModal>
  )
}
