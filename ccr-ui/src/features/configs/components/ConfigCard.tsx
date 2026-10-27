import { memo, useCallback } from 'react'
import type { ConfigItem } from '@/types'
import { SIcon } from '@/ui'
import { useAppT } from '@/i18n'
import { ConfigBadges, ConfigMetaChips } from './ConfigBadges'
import '../styles/config-card.css'

interface ConfigCardProps {
  config: ConfigItem
  highlighted: boolean
  onSwitch: (name: string) => void
  onEnable: (name: string) => void
  onEdit: (name: string) => void
}

export const ConfigCard = memo(function ConfigCard({
  config,
  highlighted,
  onSwitch,
  onEnable,
  onEdit,
}: ConfigCardProps) {
  const t = useAppT()
  const handleEdit = useCallback(() => {
    onEdit(config.name)
  }, [config.name, onEdit])
  const handleSwitch = useCallback(() => {
    if (config.enabled === false) onEnable(config.name)
    else onSwitch(config.name)
  }, [config.enabled, config.name, onEnable, onSwitch])
  const actionLabel = config.enabled === false ? t('configs.enable') : t('configs.switch')

  const rowClass = [
    'config-row group relative flex items-stretch overflow-hidden rounded-xl transition-colors duration-300',
    config.is_current ? 'config-row--current' : '',
    config.enabled === false ? 'config-row--disabled' : '',
    highlighted ? 'highlight-pulse' : '',
  ]
    .filter(Boolean)
    .join(' ')

  return (
    <div className={rowClass} data-config-name={config.name}>
      <div className="accent-bar w-[0.1875rem] shrink-0 bg-border-strong transition-colors duration-300 group-hover:bg-accent-primary" />
      <button
        type="button"
        className="config-row-main flex min-w-0 flex-1 items-center gap-4 px-5 py-4 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary"
        aria-label={`${t('common.edit')}: ${config.name}`}
        onClick={handleEdit}
      >
        <div
          className="avatar-wrapper relative flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-bg-overlay text-sm font-bold text-text-primary transition-transform duration-300 group-hover:scale-105"
        >
          {config.provider?.[0]?.toUpperCase() || 'C'}
          {config.is_current ? (
            <span className="absolute -top-1 -right-1 h-3 w-3 rounded-full border-2 border-bg-elevated bg-success shadow-sm">
              <span className="absolute inset-0 rounded-full bg-success opacity-60 animate-ping" />
            </span>
          ) : null}
        </div>
        <div className="min-w-0 flex-1 space-y-1.5">
          <div className="flex min-w-0 items-center gap-2.5">
            <h3 className="truncate font-display text-sm font-bold text-text-primary transition-colors duration-300 group-hover:text-accent-primary">
              {config.name}
            </h3>
            <ConfigBadges config={config} />
          </div>
          <p className="truncate text-xs leading-relaxed text-text-primary">
            {config.description || t('configs.noDescription')}
          </p>
        </div>
        <ConfigMetaChips config={config} />
      </button>
      <div className="flex shrink-0 items-center gap-2 pr-4">
        {config.is_current && config.enabled !== false ? (
          <span className="inline-flex cursor-default items-center gap-1 rounded-lg bg-success/5 px-3 py-1.5 text-xs font-bold text-success">
            <SIcon name="CheckCircle" size="w-3.5 h-3.5" />
            {t('configs.inUse')}
          </span>
        ) : (
          <button
            type="button"
            className="switch-btn rounded-lg px-3.5 py-1.5 text-xs font-bold opacity-0 transition-[color,background-color,opacity] duration-200 group-hover:opacity-100 focus:opacity-100"
            aria-label={`${actionLabel}: ${config.name}`}
            title={actionLabel}
            onClick={handleSwitch}
          >
            <SIcon name="ArrowRightLeft" size="w-3.5 h-3.5" className="mr-1 inline-block" />
            {actionLabel}
          </button>
        )}
        <button
          type="button"
          className={`edit-btn rounded-lg p-2 opacity-0 transition-[color,background-color,opacity] duration-200 group-hover:opacity-100 focus:opacity-100 ${config.is_current ? 'opacity-60' : ''}`}
          aria-label={`${t('common.edit')}: ${config.name}`}
          title={t('common.edit')}
          onClick={handleEdit}
        >
          <SIcon name="Settings" size="w-4 h-4" />
        </button>
      </div>
    </div>
  )
})
