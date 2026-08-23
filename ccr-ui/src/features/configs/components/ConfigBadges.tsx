import type { ConfigItem } from '@/types'
import { SIcon } from '@/ui'
import { t } from '../locale'

export function ConfigBadges({ config }: { config: ConfigItem }) {
  return (
    <div className="flex shrink-0 gap-1.5">
      {config.is_current ? (
        <span className="inline-flex items-center gap-1 rounded-full bg-emerald-400/10 px-2 py-0.5 text-[0.625rem] font-bold tracking-wider text-emerald-400 uppercase ring-1 ring-emerald-400/20">
          {t('configs.currentBadge')}
        </span>
      ) : null}
      {config.is_default ? (
        <span className="inline-flex items-center rounded-full bg-amber-400/10 px-2 py-0.5 text-[0.625rem] font-bold tracking-wider text-amber-400 uppercase ring-1 ring-amber-400/20">
          {t('configs.defaultBadge')}
        </span>
      ) : null}
    </div>
  )
}

export function ConfigMetaChips({ config }: { config: ConfigItem }) {
  if (!config.model && !config.provider && !(config.usage_count ?? 0)) return null
  return (
    <div className="hidden shrink-0 items-center gap-2 md:flex">
      {config.model ? (
        <div className="meta-chip" title={config.model}>
          <SIcon name="Sparkles" size="w-3 h-3" className="text-accent-primary opacity-70" />
          <span className="max-w-[7.5rem] truncate">{config.model}</span>
        </div>
      ) : null}
      {config.provider ? (
        <div className="meta-chip">
          <SIcon name="Building2" size="w-3 h-3" className="opacity-50" />
          <span>{config.provider}</span>
        </div>
      ) : null}
      {(config.usage_count ?? 0) > 0 ? (
        <div className="meta-chip">
          <SIcon name="TrendingUp" size="w-3 h-3" className="text-accent-secondary opacity-70" />
          <span className="font-bold text-accent-secondary">{config.usage_count}</span>
        </div>
      ) : null}
    </div>
  )
}
