import { useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { getBudgetStatus, resetBudget, setBudget } from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { PageHeader, PageShell, StatTile } from '@/ui'
import type { BudgetStatus, SetBudgetRequest } from '@/types'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import { hydrateUsageLocale } from '../translate'
import '../styles/budget-view.css'

interface BudgetFormValues {
  enabled: boolean
  daily_limit: number | null
  weekly_limit: number | null
  monthly_limit: number | null
  warn_threshold: number
}

export function BudgetView() {
  const tt = useCallback((zh: string, en: string) => {
    const locale = typeof document === 'undefined' ? 'zh-CN' : document.documentElement.lang
    return locale.startsWith('en') ? en : zh
  }, [])
  const [budgetStatus, setBudgetStatus] = useState<BudgetStatus | null>(null)
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const { register, handleSubmit, reset } = useForm<BudgetFormValues>({
    defaultValues: {
      enabled: false,
      daily_limit: null,
      weekly_limit: null,
      monthly_limit: null,
      warn_threshold: 80,
    },
  })

  const loadData = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const status = await getBudgetStatus()
      setBudgetStatus(status)
      reset({
        enabled: status.enabled,
        daily_limit: status.daily_limit,
        weekly_limit: status.weekly_limit,
        monthly_limit: status.monthly_limit,
        warn_threshold: status.warn_threshold,
      })
    } catch (caught: unknown) {
      setError(getErrorMessage(caught) || tt('加载失败', 'Load failed'))
      logger.error('Failed to load budget:', caught)
    } finally {
      setLoading(false)
    }
  }, [reset, tt])

  useEffect(() => {
    void hydrateUsageLocale()
    void loadData()
  }, [loadData])

  const onSave = handleSubmit(async (values) => {
    setSaving(true)
    try {
      const request: SetBudgetRequest = {
        enabled: values.enabled,
        daily_limit: values.daily_limit,
        weekly_limit: values.weekly_limit,
        monthly_limit: values.monthly_limit,
        warn_threshold: values.warn_threshold,
      }
      await setBudget(request)
      await loadData()
      surfaceNotify.success(tt('配置已保存', 'Configuration saved'))
    } catch (caught: unknown) {
      surfaceNotify.error(`${tt('保存失败', 'Save failed')}: ${getErrorMessage(caught)}`)
      logger.error('Failed to save budget:', caught)
    } finally {
      setSaving(false)
    }
  })

  const handleReset = useCallback(async () => {
    const confirmed = await surfaceNotify.confirm({
      title: tt('重置预算限制', 'Reset budget limits'),
      message: tt('确定要重置所有预算限制吗？', 'Are you sure you want to reset all budget limits?'),
      confirmText: tt('重置', 'Reset'),
      cancelText: tt('取消', 'Cancel'),
      type: 'danger',
    })
    if (!confirmed) return
    setSaving(true)
    try {
      await resetBudget()
      await loadData()
      surfaceNotify.success(tt('预算限制已重置', 'Budget limits reset'))
    } catch (caught: unknown) {
      surfaceNotify.error(`${tt('重置失败', 'Reset failed')}: ${getErrorMessage(caught)}`)
    } finally {
      setSaving(false)
    }
  }, [loadData, tt])

  return (
    <PageShell
      className="budget-view"
      header={(
        <PageHeader
          title={tt('预算管理', 'Budget Management')}
          description={tt('管理成本预算限制和警告阈值', 'Manage spending limits and warning thresholds')}
          actions={(
            <button type="button" disabled={loading} className="budget-primary-button" onClick={loadData}>
              {tt('刷新', 'Refresh')}
            </button>
          )}
        />
      )}
    >
      {loading ? (
        <div className="budget-shell budget-shell--loading">{tt('正在加载预算数据...', 'Loading budget data...')}</div>
      ) : null}
      {error ? (
        <div className="budget-error" role="alert">
          <h2 className="budget-error__title">{tt('加载失败', 'Load failed')}</h2>
          <p>{error}</p>
        </div>
      ) : null}
      {!loading && !error && budgetStatus ? (
        <div className="budget-content">
          <section className="budget-shell">
            <div className="budget-section-header">
              <h2 className="budget-section-title">{tt('预算状态', 'Budget status')}</h2>
              <span className={budgetStatus.enabled ? 'budget-status-pill--on' : 'budget-status-pill--off'}>
                {budgetStatus.enabled ? tt('已启用', 'Enabled') : tt('已禁用', 'Disabled')}
              </span>
            </div>
            <div className="budget-overview-grid">
              <StatTile label={tt('今日成本', 'Today cost')} value={`$${budgetStatus.current_costs.today.toFixed(4)}`} />
              <StatTile label={tt('本周成本', 'This week cost')} value={`$${budgetStatus.current_costs.this_week.toFixed(4)}`} />
              <StatTile label={tt('本月成本', 'This month cost')} value={`$${budgetStatus.current_costs.this_month.toFixed(4)}`} />
            </div>
          </section>
          <section className="budget-shell">
            <form className="budget-form" onSubmit={onSave}>
              <label className="budget-toggle" htmlFor="enabled">
                <input id="enabled" type="checkbox" className="budget-checkbox" {...register('enabled')} />
                <div>
                  <p className="budget-toggle__title">{tt('启用预算控制', 'Enable budget control')}</p>
                </div>
              </label>
              <div className="budget-input-grid">
                <label className="budget-label">
                  {tt('每日限制 ($)', 'Daily limit ($)')}
                  <input id="daily_limit" type="number" step="0.01" min="0" className="budget-input" {...register('daily_limit', { valueAsNumber: true })} />
                </label>
                <label className="budget-label">
                  {tt('每周限制 ($)', 'Weekly limit ($)')}
                  <input id="weekly_limit" type="number" step="0.01" min="0" className="budget-input" {...register('weekly_limit', { valueAsNumber: true })} />
                </label>
                <label className="budget-label">
                  {tt('每月限制 ($)', 'Monthly limit ($)')}
                  <input id="monthly_limit" type="number" step="0.01" min="0" className="budget-input" {...register('monthly_limit', { valueAsNumber: true })} />
                </label>
              </div>
              <label className="budget-label">
                {tt('警告阈值 (%)', 'Warning threshold (%)')}
                <input id="warn_threshold" type="number" min="0" max="100" className="budget-input" {...register('warn_threshold', { valueAsNumber: true })} />
              </label>
              <div className="budget-form-actions">
                <button type="submit" disabled={saving} className="budget-primary-button">
                  {saving ? tt('保存中...', 'Saving...') : tt('保存配置', 'Save settings')}
                </button>
                <button type="button" disabled={saving} className="budget-secondary-button" onClick={handleReset}>
                  {tt('重置所有限制', 'Reset all limits')}
                </button>
              </div>
            </form>
          </section>
        </div>
      ) : null}
    </PageShell>
  )
}
