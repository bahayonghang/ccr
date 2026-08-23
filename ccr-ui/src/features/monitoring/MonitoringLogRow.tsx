import { memo, useMemo } from 'react'
import DOMPurify from 'dompurify'
import { createAnsiRenderer } from '@/utils/ansiRenderer'
import { redactLogText } from '@/utils/logRedact'
import { formatTime, getLevelClass } from './monitoring-format'
import type { MonitoringEntry } from './monitoring-types'

const renderer = createAnsiRenderer()

interface MonitoringLogRowProps {
  log: MonitoringEntry
  locale: string
}

export const MonitoringLogRow = memo(function MonitoringLogRow({ log, locale }: MonitoringLogRowProps) {
  const html = useMemo(() => {
    const redacted = redactLogText(log.message)
    return DOMPurify.sanitize(renderer.renderLine(redacted))
  }, [log.message])

  return (
    <div
      data-testid="monitoring-log-row"
      className="grid grid-cols-[4.5rem_3.875rem_5.875rem_5.875rem_minmax(0,1fr)] gap-2 border-b border-border-default/25 px-3 py-2 last:border-b-0 hover:bg-bg-elevated/60"
    >
      <span className="tabular-nums text-text-muted">{formatTime(locale, log.timestamp)}</span>
      <span>
        <span className={`rounded-full px-2 py-0.5 text-[0.625rem] font-bold uppercase ${getLevelClass(log.level)}`}>
          {log.level}
        </span>
      </span>
      <span className="truncate text-text-muted" title={log.channel}>
        {log.channel}
      </span>
      <span className="truncate text-text-muted" title={log.source}>
        {log.source}
      </span>
      <span
        className="min-w-0 break-words leading-5 text-text-secondary line-clamp-2"
        title={log.message}
        dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(html) }}
      />
    </div>
  )
})
