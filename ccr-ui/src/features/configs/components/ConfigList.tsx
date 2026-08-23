import { memo } from 'react'
import type { ConfigItem } from '@/types'
import { SIcon, Spinner } from '@/ui'
import { t } from '../locale'
import { ConfigCard } from './ConfigCard'

interface ConfigListProps {
  configs: ConfigItem[]
  loading: boolean
  error: string | null
  highlightedName: string | null
  onSwitch: (name: string) => void
  onEdit: (name: string) => void
}

export const ConfigList = memo(function ConfigList({
  configs,
  loading,
  error,
  highlightedName,
  onSwitch,
  onEdit,
}: ConfigListProps) {
  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-20 text-text-muted">
        <Spinner size="xl" className="mb-4 text-accent-primary" />
        <span className="animate-pulse font-mono text-sm">{t('common.loading')}</span>
      </div>
    )
  }
  if (error) {
    return (
      <div className="flex items-center gap-3 rounded-xl border border-accent-danger/20 bg-accent-danger/10 p-4 text-accent-danger">
        <SIcon name="AlertCircle" size="w-5 h-5" className="shrink-0" />
        <span>
          {t('configs.operationFailed')}: {error}
        </span>
      </div>
    )
  }
  if (configs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-20 text-text-muted">
        <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-2xl glass-surface">
          <SIcon name="Settings" size="w-8 h-8" className="opacity-20" />
        </div>
        <p>{t('configs.noConfigsInCategory')}</p>
      </div>
    )
  }
  return (
    <div className="flex flex-col gap-2">
      {configs.map((config) => (
        <ConfigCard
          key={config.name}
          config={config}
          highlighted={highlightedName === config.name}
          onSwitch={onSwitch}
          onEdit={onEdit}
        />
      ))}
    </div>
  )
})
