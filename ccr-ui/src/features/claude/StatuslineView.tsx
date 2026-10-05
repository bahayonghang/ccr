import { useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import { getStatusline, updateStatusline } from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { ClaudeSubnav } from '@/features/claude/ClaudeSubnav'
import { t } from '@/features/claude/locale'
import { PageHeader, PageShell, SIcon, Spinner } from '@/ui'
import { logger } from '@/utils/logger'

interface StatuslineForm {
  enabled: boolean
  command: string
}

export function StatuslineView() {
  const [loading, setLoading] = useState(true)
  const form = useForm<StatuslineForm>({ defaultValues: { enabled: false, command: '' } })
  const { register, handleSubmit, reset, formState } = form

  const loadConfig = useCallback(async () => {
    setLoading(true)
    try {
      const config = await getStatusline()
      reset({ enabled: Boolean(config.enabled), command: config.command ?? '' })
    } catch (error) {
      logger.error('Failed to load statusline config:', error)
      surfaceNotify.error(t('common.loadFailed'))
      reset({ enabled: false, command: '' })
    } finally {
      setLoading(false)
    }
  }, [reset])

  useEffect(() => {
    void loadConfig()
  }, [loadConfig])

  const onValid = useCallback(async (values: StatuslineForm) => {
    try {
      await updateStatusline(values)
      surfaceNotify.success(t('common.saveSuccess'))
    } catch (error) {
      logger.error('Failed to save statusline config:', error)
      surfaceNotify.error(t('common.operationFailed'))
    }
  }, [])
  const onSubmit = useMemo(() => handleSubmit(onValid), [handleSubmit, onValid])

  return (
    <PageShell
      header={<PageHeader title={t('statusline.pageTitle')} />}
      subnav={<ClaudeSubnav />}
    >
      {loading ? (
        <div className="py-20 text-center text-text-muted" role="status">
          <Spinner size="lg" className="mx-auto mb-4 text-accent-secondary" />
          <span>{t('common.loading')}</span>
        </div>
      ) : (
        <div className="space-y-6">
          <div className="rounded-2xl border border-border-default/25 bg-bg-surface p-6 shadow-sm">
            <h3 className="mb-4 flex items-center text-lg font-bold text-text-primary">
              <SIcon name="Settings" size="w-5 h-5" className="mr-2 text-accent-secondary" />
              {t('statusline.configuration')}
            </h3>
            <div className="space-y-6">
              <div className="flex items-center justify-between rounded-xl border border-border-default/30 bg-bg-elevated p-4">
                <div>
                  <p id="enabled-label" className="font-semibold text-text-primary">
                    {t('statusline.enabled')}
                  </p>
                  <p id="enabled-description" className="mt-1 text-sm text-text-muted">
                    {t('statusline.enabledDescription')}
                  </p>
                </div>
                <label className="inline-flex cursor-pointer items-center">
                  <input
                    id="statusline-enabled"
                    type="checkbox"
                    className="h-5 w-5 rounded border-border-default text-accent-secondary"
                    aria-labelledby="enabled-label"
                    aria-describedby="enabled-description"
                    {...register('enabled')}
                  />
                </label>
              </div>
              <div className="rounded-xl border border-border-default/30 bg-bg-elevated p-4">
                <label htmlFor="statusline-command" className="mb-2 block font-semibold text-text-primary">
                  {t('statusline.command')}
                </label>
                <p id="command-description" className="mb-3 text-sm text-text-muted">
                  {t('statusline.commandDescription')}
                </p>
                <input
                  id="statusline-command"
                  type="text"
                  className="w-full rounded-lg border border-border-default bg-bg-surface px-4 py-3 font-mono text-sm outline-none focus:border-accent-secondary focus:ring-2 focus:ring-accent-secondary/20"
                  placeholder={t('statusline.commandPlaceholder')}
                  aria-describedby="command-description command-help"
                  {...register('command')}
                />
                <p id="command-help" className="mt-2 text-xs text-text-muted">
                  {t('statusline.commandHelp')}
                </p>
              </div>
            </div>
            <div className="mt-6 flex justify-end border-t border-border-default/30 pt-4">
              <button
                type="button"
                className="flex min-h-11 items-center rounded-lg bg-accent-secondary px-6 py-2.5 font-medium text-[color:var(--color-accent-primary-contrast)] shadow-md disabled:opacity-60"
                disabled={formState.isSubmitting}
                aria-busy={formState.isSubmitting}
                onClick={onSubmit}
              >
                <SIcon name="Save" size="w-4 h-4" className="mr-2" />
                {formState.isSubmitting ? t('common.saving') : t('common.save')}
              </button>
            </div>
          </div>
          <div className="rounded-2xl border border-border-default/25 bg-bg-surface p-6 shadow-sm" role="region" aria-labelledby="about-title">
            <h3 id="about-title" className="mb-4 flex items-center text-lg font-bold text-text-primary">
              <SIcon name="Info" size="w-5 h-5" className="mr-2 text-accent-secondary" />
              {t('statusline.about')}
            </h3>
            <div className="text-sm text-text-secondary">
              <p>{t('statusline.aboutDescription')}</p>
              <ul className="mt-3 space-y-2" role="list">
                <li className="flex items-start gap-2">
                  <span className="text-accent-secondary" aria-hidden="true">•</span>
                  {t('statusline.feature1')}
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-accent-secondary" aria-hidden="true">•</span>
                  {t('statusline.feature2')}
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-accent-secondary" aria-hidden="true">•</span>
                  {t('statusline.feature3')}
                </li>
              </ul>
            </div>
          </div>
        </div>
      )}
    </PageShell>
  )
}
