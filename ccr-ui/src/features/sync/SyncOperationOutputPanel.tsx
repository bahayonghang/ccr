import { useCallback, useMemo, useState } from 'react'
import { copyText } from '@/utils/clipboard'
import type { SyncOperationOutput } from '@/types/syncSelection'
import { SIcon } from '@/ui'
import { useSyncT } from './locale'
import './styles/sync-output.css'

interface SyncOperationOutputPanelProps {
  output: SyncOperationOutput | null
  onClear: () => void
}

export function SyncOperationOutputPanel({ output, onClear }: SyncOperationOutputPanelProps) {
  const t = useSyncT()
  const [copied, setCopied] = useState(false)
  const statusClass = `sync-output-card--${output?.status ?? 'success'}`
  const statusIcon = output?.status === 'success' ? 'CheckCircle' : output?.status === 'partial' ? 'AlertTriangle' : 'XCircle'
  const successRatioText = useMemo(() => {
    const successCount = output?.successCount ?? 0
    const total = output?.total
    if (typeof total === 'number') return t('sync.output.successRatio', { success: successCount, total })
    return t('sync.output.successCountText', { success: successCount })
  }, [output, t])
  const durationText = output?.durationMs == null ? '—' : `${output.durationMs}ms`

  const copyRawDetails = useCallback(async () => {
    if (!output) return
    if (await copyText(output.rawLog)) {
      setCopied(true)
      window.setTimeout(() => setCopied(false), 1500)
    }
  }, [output])
  const handleCopy = useCallback(() => {
    void copyRawDetails()
  }, [copyRawDetails])

  if (!output) return null

  return (
    <section className={`sync-output-card ${statusClass}`}>
      <header className="sync-output-card__header">
        <div className="sync-output-card__heading">
          <div className="sync-output-card__icon">
            <SIcon name={statusIcon} size="w-5 h-5" />
          </div>
          <div>
            <p className="sync-output-card__eyebrow">{t('sync.output.title')}</p>
            <h2>{output.title}</h2>
            <p className="sync-output-card__summary">{output.summary}</p>
          </div>
        </div>
        <button type="button" className="sync-output-card__close" aria-label={t('common.close')} onClick={onClear}>
          <SIcon name="XCircle" size="w-4 h-4" />
        </button>
      </header>
      <div className="sync-output-card__metrics">
        <div className="sync-output-card__metric">
          <span>{t('sync.output.successMetric')}</span>
          <strong>{successRatioText}</strong>
        </div>
        <div className="sync-output-card__metric">
          <span>{t('sync.output.failedMetric')}</span>
          <strong>{output.failedCount}</strong>
        </div>
        <div className="sync-output-card__metric">
          <span>{t('sync.output.durationMetric')}</span>
          <strong>{durationText}</strong>
        </div>
      </div>
      {output.suggestions.length > 0 ? (
        <section className="sync-output-card__advice">
          <p className="sync-output-card__section-title">{t('sync.output.suggestionsTitle')}</p>
          <ul>
            {output.suggestions.map((suggestion) => (
              <li key={suggestion}>{suggestion}</li>
            ))}
          </ul>
        </section>
      ) : null}
      {output.failures.length > 0 ? (
        <section className="sync-output-card__failures">
          <p className="sync-output-card__section-title">{t('sync.output.failuresTitle', { count: output.failures.length })}</p>
          {output.failures.map((failure) => (
            <article key={`${failure.assetId ?? failure.assetName}-${failure.message}`} className="sync-output-card__failure">
              <header>
                <strong>{failure.assetName}</strong>
                <span>{failure.reason}</span>
              </header>
              <p className="sync-output-card__failure-message">{failure.message}</p>
            </article>
          ))}
        </section>
      ) : null}
      <details className="sync-output-card__raw">
        <summary>{t('sync.output.rawDetails')}</summary>
        <div className="sync-output-card__raw-body">
          <button type="button" className="sync-output-card__copy" onClick={handleCopy}>
            <SIcon name="Copy" size="w-4 h-4" />
            {copied ? t('sync.output.copied') : t('sync.output.copyRaw')}
          </button>
          <pre>{output.rawLog}</pre>
        </div>
      </details>
    </section>
  )
}
