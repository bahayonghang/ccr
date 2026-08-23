import { useBackendHealth } from '@/composables/useBackendHealth'
import { isTauriEnvironment } from '@/api/runtime/environment'
import { useShellT } from '@/shell/i18n'
import { SIcon } from '@/ui/s-icon'

export function BackendStatusBanner() {
  const { status, errorMessage, checkHealth } = useBackendHealth()
  const t = useShellT()
  if (!isTauriEnvironment() || status !== 'error') return null

  return (
    <div className="mx-6 mt-4 rounded-xl border border-red-200/80 bg-red-50/80 px-4 py-3 text-sm text-red-800 shadow-sm dark:border-red-800/70 dark:bg-red-900/30 dark:text-red-200">
      <div className="flex items-start gap-3">
        <SIcon name="AlertTriangle" size="h-4 w-4" className="mt-0.5" />
        <div className="flex-1">
          <div className="font-semibold">{t('common.backend.bannerTitle')}</div>
          <div className="mt-1 text-xs opacity-80">
            {errorMessage || t('common.backend.bannerFallback')}
          </div>
          <div className="mt-1 text-xs opacity-80">{t('common.backend.bannerHint')}</div>
        </div>
        <button
          type="button"
          className="shrink-0 rounded-lg border border-red-200 bg-white/80 px-3 py-1 text-xs font-semibold text-red-700 dark:border-red-700/60 dark:bg-red-900/30 dark:text-red-200"
          onClick={() => void checkHealth()}
        >
          {t('common.backend.retry')}
        </button>
      </div>
    </div>
  )
}
