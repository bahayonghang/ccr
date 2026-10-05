import { memo, useMemo } from 'react'
import DOMPurify from 'dompurify'
import type { LedgerChannel } from './commands-model'

interface LedgerLineProps {
  channel: LedgerChannel
  index: number
  html: string
}

export const LedgerLine = memo(function LedgerLine({ channel, index, html }: LedgerLineProps) {
  const sanitized = useMemo(() => DOMPurify.sanitize(html), [html])
  return (
    <div className={`commands-terminal__line commands-terminal__line--${channel}`}>
      <span className="commands-terminal__channel">{channel}</span>
      <code
        className="commands-terminal__text"
        data-index={index}
        dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(sanitized) }}
      />
    </div>
  )
})
