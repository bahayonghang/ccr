import { useEffect, useState } from 'react'
import { isTauriEnvironment, TauriRuntimeApi } from '@/api/runtime/environment'
import { SIcon } from '@/ui'
import { logger } from '@/utils/logger'
import { t } from '../locale'

export function ConfigsRuntimeBadge() {
  const [isTauri, setIsTauri] = useState(false)
  const [tauriVersion, setTauriVersion] = useState<string | null>(null)

  useEffect(() => {
    const desktop = isTauriEnvironment()
    setIsTauri(desktop)
    if (!desktop) return
    void TauriRuntimeApi.getTauriVersion()
      .then(setTauriVersion)
      .catch((error) => logger.error('Failed to get Tauri version:', error))
  }, [])

  const className = isTauri
    ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300'
    : 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300'

  return (
    <div className={`inline-flex items-center gap-2 rounded-lg px-3 py-1.5 text-sm font-medium ${className}`}>
      <SIcon name={isTauri ? 'Monitor' : 'Globe'} className="opacity-70" size="w-4 h-4" />
      <span>{isTauri ? t('common.environment.desktopApp') : t('common.environment.webVersion')}</span>
      {isTauri && tauriVersion ? (
        <span className="rounded bg-bg-surface px-1.5 py-0.5 text-xs">
          {t('common.versionPrefix')}
          {tauriVersion}
        </span>
      ) : null}
    </div>
  )
}
