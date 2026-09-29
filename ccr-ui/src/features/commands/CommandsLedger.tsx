import { memo, useCallback } from 'react'
import { SIcon } from '@/ui'
import { LedgerLine } from './LedgerLine'
import type { useCommandsPage } from './useCommandsPage'

type Page = ReturnType<typeof useCommandsPage>

function JobNotice({ page }: { page: Page }) {
  const onRefresh = useCallback(() => { void page.handleRefreshJob() }, [page])
  if (!page.jobExpired && !page.jobError && !page.liveUpdatesUnavailable) return null
  return (
    <div className="commands-notice" role="alert">
      <p>{page.jobExpired ? page.t('commands.jobExpired') : page.jobError}</p>
      {page.liveUpdatesUnavailable ? <p>{page.t('commands.liveUpdatesUnavailable')}</p> : null}
      {page.currentSnapshot && !page.jobExpired ? <button type="button" onClick={onRefresh}>{page.t('common.retry')}</button> : null}
    </div>
  )
}

function HistoryNotice({ page }: { page: Page }) {
  const onRefresh = useCallback(() => { void page.handleRefreshHistory() }, [page])
  const status = page.historyWrite?.status
  return <>
    {status === 'pending' || status === 'saved' ? (
      <p role="status">{page.t(status === 'pending' ? 'commands.historyPending' : 'commands.historySaved')}</p>
    ) : null}
    {page.failedHistoryWrites.map((write) => (
      <div key={write.summary.job_id} className="commands-notice" role="alert">
        <strong>{page.t('commands.historySaveFailed')}</strong>
        <p>{write.summary.command} · {write.summary.job_id} · {write.error}</p>
        <button type="button" onClick={onRefresh}>{page.t('common.refresh')}</button>
      </div>
    ))}
  </>
}

export const CommandsLedger = memo(function CommandsLedger({ page }: { page: Page }) {
  const onCopy = useCallback(() => {
    void page.handleCopyOutput()
  }, [page])
  const hasOutput = page.ledgerLines.length > 0
  const statusClass = page.currentSnapshot ? `commands-status--${page.currentSnapshot.status}` : ''
  return (
    <section className="commands-panel commands-ledger">
      <div className="commands-panel__header commands-panel__header--wide">
        <div>
          <p className="commands-panel__eyebrow">{page.t('commands.ledgerEyebrow')}</p>
          <h2 className="commands-panel__title">{page.t('commands.output')}</h2>
          <p className="commands-panel__subtitle">
            {page.currentSnapshot
              ? page.t('commands.ledgerSubtitleActive', { job: page.currentSnapshot.job_id.slice(0, 18), command: `ccr ${page.currentSnapshot.command}` })
              : page.t('commands.ledgerSubtitleIdle')}
          </p>
        </div>
        <div className="commands-panel__actions">
          <button type="button" className="rounded-lg border border-border-default px-3 py-1.5 text-xs" disabled={!hasOutput} onClick={onCopy}>{page.t('commands.copy')}</button>
          <button type="button" className="rounded-lg border border-border-default px-3 py-1.5 text-xs" disabled={!page.currentSnapshot || page.isRunning || page.submitting} onClick={page.handleClearOutput}>{page.t('commands.clear')}</button>
        </div>
      </div>
      <JobNotice page={page} />
      <HistoryNotice page={page} />
      {page.currentSnapshot ? (
        <div className="commands-ledger__metrics">
          <div className="commands-ledger__metric">
            <span>{page.t('commands.jobStatus')}</span>
            <strong className={statusClass}>{page.t(`commands.status.${page.jobExpired ? 'unavailable' : page.currentSnapshot.status}`)}</strong>
          </div>
          <div className="commands-ledger__metric">
            <span>{page.t('commands.duration')}</span>
            <strong>{page.currentSnapshot.duration_ms == null ? '—' : `${page.currentSnapshot.duration_ms}ms`}</strong>
          </div>
          <div className="commands-ledger__metric">
            <span>{page.t('commands.exitCode')}</span>
            <strong>{page.currentSnapshot.exit_code ?? '—'}</strong>
          </div>
          <div className="commands-ledger__metric">
            <span>{page.t('commands.terminalOutput')}</span>
            <strong>{page.t('commands.linesCount', { count: page.ledgerLines.length })}</strong>
          </div>
        </div>
      ) : null}
      {page.isRunning ? (
        <div className="commands-ledger__status-strip" role="status" aria-live="polite">
          <span className="commands-ledger__pulse" />
          <span>{page.t('commands.processing')}</span>
        </div>
      ) : null}
      {hasOutput ? (
        <div className="commands-terminal">
          {page.ledgerTruncated ? <div className="commands-terminal__truncated">{page.t('commands.ledgerTruncated', { count: page.MAX_LEDGER_LINES })}</div> : null}
          {page.ledgerLines.map((line) => (
            <LedgerLine key={`${line.channel}-${line.index}`} channel={line.channel} index={line.index} html={line.safeHtml} />
          ))}
        </div>
      ) : !page.isRunning ? (
        <div className="commands-ledger-empty" role="status">
          <SIcon name="FileX" size="w-6 h-6" />
          <strong>{page.t('commands.readyTitle')}</strong>
          <p>{page.t('commands.readyDescription')}</p>
        </div>
      ) : null}
    </section>
  )
})
