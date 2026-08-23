import { useRef } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import type { HistoryEntry } from '@/types'
import { formatRelativeTime } from '@/utils/codexHelpers'
import { SIcon } from './s-icon'
import { Spinner } from './spinner'

interface HistoryListProps {
  entries: HistoryEntry[]
  loading?: boolean
  emptyTitle?: string
  emptyDescription?: string
  recordsLabel?: string
  title?: string
}

const OPERATION_LABEL: Record<string, string> = {
  switch: 'Switched Config',
  init: 'Initialized',
  update: 'Updated Config',
  delete: 'Deleted Config',
  validate: 'Validation Run',
  clean: 'Cleaned Backups',
  import: 'Imported',
  export: 'Exported',
}

const OPERATION_ICON: Record<string, string> = {
  switch: 'GitBranch',
  init: 'CheckCircle',
  update: 'FileEdit',
  delete: 'Trash2',
  validate: 'CheckCircle',
  clean: 'RefreshCw',
  import: 'ArrowRight',
  export: 'ArrowRight',
}

const OPERATION_COLOR: Record<string, string> = {
  switch: 'var(--chart-color-0)',
  init: 'var(--chart-color-1)',
  update: 'var(--chart-color-3)',
  delete: 'var(--chart-color-4)',
  validate: 'var(--chart-color-2)',
  clean: 'var(--chart-color-3)',
  import: 'var(--chart-color-1)',
  export: 'var(--chart-color-0)',
}

export function HistoryList({
  entries,
  loading = false,
  emptyTitle = '暂无历史记录',
  emptyDescription = '配置变更会显示在这里。',
  recordsLabel,
  title = 'History',
}: HistoryListProps) {
  const parentRef = useRef<HTMLDivElement | null>(null)
  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 160,
    overscan: 5,
  })

  return (
    <div className="flex h-[600px] flex-col">
      <div className="mb-4 flex flex-shrink-0 items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-text-primary">{title}</h2>
          <p className="text-sm text-text-primary">
            {recordsLabel ?? `${entries.length} records`}
          </p>
        </div>
      </div>
      {loading ? (
        <div className="flex flex-1 items-center justify-center">
          <Spinner size="xl" className="text-accent-primary" />
        </div>
      ) : null}
      {!loading && entries.length === 0 ? (
        <div className="flex flex-1 flex-col items-center justify-center text-text-muted">
          <div className="glass-surface mb-4 rounded-full p-6">
            <SIcon name="History" size="w-8 h-8" className="opacity-20" />
          </div>
          <p className="text-lg font-medium text-text-primary">{emptyTitle}</p>
          <p className="text-sm">{emptyDescription}</p>
        </div>
      ) : null}
      {!loading && entries.length > 0 ? (
        <div ref={parentRef} className="scrollbar-thin flex-1 overflow-auto pr-2">
          <div
            className="relative w-full"
            style={{ height: `${virtualizer.getTotalSize()}px` }}
          >
            {virtualizer.getVirtualItems().map((row) => {
              const entry = entries[row.index]
              const color = OPERATION_COLOR[entry.operation] || 'var(--color-text-muted)'
              return (
                <div
                  key={entry.id}
                  className="absolute top-0 left-0 w-full pb-3"
                  style={{ transform: `translateY(${row.start}px)` }}
                >
                  <HistoryCard entry={entry} color={color} />
                </div>
              )
            })}
          </div>
        </div>
      ) : null}
    </div>
  )
}

function HistoryCard({ entry, color }: { entry: HistoryEntry; color: string }) {
  const changes = entry.changes ?? []
  return (
    <article
      className="group relative rounded-2xl border border-border-default/40 bg-bg-surface p-4 transition-colors duration-300"
      style={{ borderLeftWidth: 4, borderLeftColor: color }}
    >
      <div className="flex gap-4">
        <div
          className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border"
          style={{
            borderColor: color,
            backgroundColor: 'transparent',
            color,
          }}
        >
          <SIcon name={OPERATION_ICON[entry.operation] || 'GitBranch'} size="w-5 h-5" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="mb-2 flex items-start justify-between">
            <div>
              <h3 className="font-bold text-text-primary">
                {OPERATION_LABEL[entry.operation] || entry.operation}
              </h3>
              <div className="mt-1 flex items-center gap-3 text-xs text-text-primary">
                <span className="inline-flex items-center gap-1">
                  <SIcon name="Clock" size="w-3 h-3" />
                  {formatRelativeTime(entry.timestamp)}
                </span>
                <span className="inline-flex items-center gap-1">
                  <SIcon name="User" size="w-3 h-3" />
                  {entry.actor}
                </span>
              </div>
            </div>
            <span
              className="rounded px-2 py-0.5 text-[10px] font-bold tracking-wider uppercase"
              style={{ color }}
            >
              {entry.operation}
            </span>
          </div>
          {entry.from_config && entry.to_config ? (
            <div className="mb-2 flex items-center gap-2 rounded border border-border-default/15 bg-bg-elevated p-2">
              <code className="rounded bg-accent-danger/10 px-1.5 py-0.5 text-xs text-accent-danger">
                {entry.from_config}
              </code>
              <SIcon name="ArrowRight" size="w-3 h-3" className="text-text-muted" />
              <code className="rounded bg-accent-success/10 px-1.5 py-0.5 text-xs text-accent-success">
                {entry.to_config}
              </code>
            </div>
          ) : null}
          {changes.length > 0 ? (
            <div className="my-2 space-y-1">
              {changes.slice(0, 3).map((change) => (
                <div
                  key={change.key}
                  className="grid grid-cols-[auto_1fr] gap-2 rounded border border-border-default/10 bg-bg-elevated p-1.5 font-mono text-xs"
                >
                  <span className="font-bold text-text-primary">{change.key}</span>
                  <div className="flex items-center gap-1 truncate text-text-muted">
                    <span className="truncate">{change.old_value || '_'}</span>
                    <SIcon name="ArrowRight" size="h-3 w-3" />
                    <span className="truncate text-text-primary">{change.new_value || '_'}</span>
                  </div>
                </div>
              ))}
            </div>
          ) : null}
          <div className="mt-2 border-t border-border-default/10 pt-2 font-mono text-[10px] text-text-muted">
            {entry.id}
          </div>
        </div>
      </div>
    </article>
  )
}
