import { memo, useCallback } from 'react'
import type { ConfigItem } from '@/types'
import { SIcon } from '@/ui'
import { t } from '../locale'
import { providerKind } from '../lib/configList'
import { ConfigBadges, ConfigMetaChips } from './ConfigBadges'
import '../styles/config-card.css'

interface ConfigCardProps {
  config: ConfigItem
  highlighted: boolean
  onSwitch: (name: string) => void
  onEdit: (name: string) => void
}

const ACCENT: Record<string, string> = {
  official: 'bg-cyan-400 group-hover:bg-cyan-300',
  third: 'bg-violet-400 group-hover:bg-violet-300',
  uncategorized: 'bg-amber-400 group-hover:bg-amber-300',
}

const AVATAR: Record<string, string> = {
  official: 'bg-gradient-to-br from-cyan-500 to-cyan-700',
  third: 'bg-gradient-to-br from-violet-500 to-violet-700',
  uncategorized: 'bg-gradient-to-br from-amber-500 to-amber-700',
}

const NAME: Record<string, string> = {
  official: 'text-cyan-400 group-hover:text-cyan-300',
  third: 'text-violet-400 group-hover:text-violet-300',
  uncategorized: 'text-amber-400 group-hover:text-amber-300',
}

export const ConfigCard = memo(function ConfigCard({
  config,
  highlighted,
  onSwitch,
  onEdit,
}: ConfigCardProps) {
  const kind = providerKind(config)
  const handleEdit = useCallback(() => {
    onEdit(config.name)
  }, [config.name, onEdit])
  const handleSwitch = useCallback(() => {
    onSwitch(config.name)
  }, [config.name, onSwitch])

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
      <div className={`accent-bar w-[0.1875rem] shrink-0 transition-colors duration-300 ${ACCENT[kind]}`} />
      <button
        type="button"
        className="config-row-main flex min-w-0 flex-1 items-center gap-4 px-5 py-4 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/35"
        aria-label={`${t('common.edit')}: ${config.name}`}
        onClick={handleEdit}
      >
        <div
          className={`avatar-wrapper relative flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-sm font-bold text-white shadow-md transition-transform duration-300 group-hover:scale-105 ${AVATAR[kind]}`}
        >
          {config.provider?.[0]?.toUpperCase() || 'C'}
          {config.is_current ? (
            <span className="absolute -top-1 -right-1 h-3 w-3 rounded-full border-2 border-bg-elevated bg-emerald-400 shadow-sm">
              <span className="absolute inset-0 rounded-full bg-emerald-400 opacity-60 animate-ping" />
            </span>
          ) : null}
        </div>
        <div className="min-w-0 flex-1 space-y-1.5">
          <div className="flex min-w-0 items-center gap-2.5">
            <h3 className={`truncate font-display text-sm font-bold transition-colors duration-300 ${NAME[kind]}`}>
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
        {config.is_current ? (
          <span className="inline-flex cursor-default items-center gap-1 rounded-lg bg-emerald-400/5 px-3 py-1.5 text-xs font-bold text-emerald-400/70">
            <SIcon name="CheckCircle" size="w-3.5 h-3.5" />
            {t('configs.inUse')}
          </span>
        ) : (
          <button
            type="button"
            className="switch-btn rounded-lg px-3.5 py-1.5 text-xs font-bold opacity-0 transition-[color,background-color,opacity] duration-200 group-hover:opacity-100 focus:opacity-100"
            aria-label={`${t('configs.switch')}: ${config.name}`}
            title={t('configs.switch')}
            onClick={handleSwitch}
          >
            <SIcon name="ArrowRightLeft" size="w-3.5 h-3.5" className="mr-1 inline-block" />
            {t('configs.switch')}
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
