import { memo, useCallback } from 'react'
import type { OpenCodeProviderConfig } from '@/types'
import { SIcon } from '@/ui'
import { maskSecret } from '@/utils/opencode'
import { dangerBtnClass, secondaryBtnClass } from '../ui-classes'
import { useOpenCodeLocale } from '../locale'

interface OpenCodeProviderCardProps {
  provider: OpenCodeProviderConfig
  enabled: boolean
  onToggle: (provider: OpenCodeProviderConfig) => void
  onEdit: (provider: OpenCodeProviderConfig) => void
  onRemove: (provider: OpenCodeProviderConfig) => void
}

export const OpenCodeProviderCard = memo(function OpenCodeProviderCard({
  provider,
  enabled,
  onToggle,
  onEdit,
  onRemove,
}: OpenCodeProviderCardProps) {
  const { tt } = useOpenCodeLocale()
  const handleToggle = useCallback(() => onToggle(provider), [onToggle, provider])
  const handleEdit = useCallback(() => onEdit(provider), [onEdit, provider])
  const handleRemove = useCallback(() => onRemove(provider), [onRemove, provider])
  const modelCount = Object.keys(provider.models || {}).length

  return (
    <article className="rounded-2xl border border-border-subtle bg-bg-surface p-5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0">
          <div className="mb-3 flex flex-wrap items-center gap-2">
            <span className="rounded-full border border-accent-success/20 bg-accent-success/10 px-3 py-1 text-xs font-semibold text-accent-success">
              {provider.id}
            </span>
            <span
              className={
                enabled
                  ? 'rounded-full bg-accent-success/10 px-3 py-1 text-xs font-semibold text-accent-success'
                  : 'rounded-full bg-accent-warning/10 px-3 py-1 text-xs font-semibold text-accent-warning'
              }
            >
              {enabled ? tt('已启用', 'enabled') : tt('已禁用', 'disabled')}
            </span>
          </div>
          <h2 className="text-lg font-semibold text-text-primary">{provider.name || provider.id}</h2>
          <div className="mt-3 grid gap-3 md:grid-cols-3">
            <div className="rounded-2xl border border-border-default/55 bg-bg-base p-3">
              <span className="text-[0.6875rem] font-semibold text-text-muted">{tt('API key', 'API key')}</span>
              <p className="mt-2 text-sm text-text-primary">{maskSecret(provider.options?.apiKey)}</p>
            </div>
            <div className="rounded-2xl border border-border-default/55 bg-bg-base p-3">
              <span className="text-[0.6875rem] font-semibold text-text-muted">{tt('baseURL', 'baseURL')}</span>
              <p className="mt-2 break-all text-sm text-text-primary">{provider.options?.baseURL || 'default'}</p>
            </div>
            <div className="rounded-2xl border border-border-default/55 bg-bg-base p-3">
              <span className="text-[0.6875rem] font-semibold text-text-muted">{tt('models', 'models')}</span>
              <p className="mt-2 text-sm text-text-primary">{modelCount}</p>
            </div>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <button type="button" className={secondaryBtnClass} onClick={handleToggle}>
            <SIcon name={enabled ? 'PauseCircle' : 'PlayCircle'} size="w-4 h-4" />
            {enabled ? tt('停用', 'Disable') : tt('启用', 'Enable')}
          </button>
          <button type="button" className={secondaryBtnClass} onClick={handleEdit}>
            <SIcon name="Pencil" size="w-4 h-4" />
            {tt('编辑', 'Edit')}
          </button>
          <button type="button" className={dangerBtnClass} onClick={handleRemove}>
            <SIcon name="Trash2" size="w-4 h-4" />
            {tt('删除', 'Delete')}
          </button>
        </div>
      </div>
    </article>
  )
})
