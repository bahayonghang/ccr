import { memo, useCallback, type ChangeEvent, type KeyboardEvent } from 'react'
import { useUsageDashboardContext } from '../UsageDashboardContext'
import { useUsageT } from '../translate'
import { useVirtualList } from '../virtual/useVirtualList'
import '../styles/usage-logs-tab.css'

const SKELETON_ROWS = ['sk-1', 'sk-2', 'sk-3', 'sk-4', 'sk-5', 'sk-6', 'sk-7', 'sk-8', 'sk-9', 'sk-10', 'sk-11', 'sk-12']

interface LogRowData {
  id: string
  time: string
  platform: string
  model: string
  input: string
  output: string
  cost: string
}

const LogRow = memo(function LogRow({ row }: { row: LogRowData }) {
  return (
    <div className="diagnostics-tab__row diagnostics-tab__row--item">
      <div className="diagnostics-tab__cell diagnostics-tab__cell--time">{row.time}</div>
      <div className="diagnostics-tab__cell">{row.platform}</div>
      <div className="diagnostics-tab__cell diagnostics-tab__cell--model" title={row.model}>{row.model}</div>
      <div className="diagnostics-tab__cell is-right">{row.input}</div>
      <div className="diagnostics-tab__cell is-right">{row.output}</div>
      <div className="diagnostics-tab__cell is-right">{row.cost}</div>
    </div>
  )
})

export function UsageLogsTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  const records = ctx.logsRecords
  const { parentRef, virtualizer } = useVirtualList<HTMLDivElement>({
    count: records.length,
    estimateSize: () => 44,
    overscan: 10,
  })

  const handleFilter = useCallback((event: ChangeEvent<HTMLInputElement>) => {
    ctx.updateLogModelFilter(event.target.value)
  }, [ctx])

  const handleSearch = useCallback(() => {
    void ctx.loadLogs('reset')
  }, [ctx])

  const handleKey = useCallback((event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') void ctx.loadLogs('reset')
  }, [ctx])

  const handlePrev = useCallback(() => {
    void ctx.loadLogs('prev')
  }, [ctx])

  const handleNext = useCallback(() => {
    void ctx.loadLogs('next')
  }, [ctx])

  const unknownModel = t('usage.dashboard.diagnostics.unknownModel')

  return (
    <section className="diagnostics-tab glass-panel rounded-xl p-4">
      <div className="diagnostics-tab__head">
        <div>
          <h3 className="diagnostics-tab__title">{t('usage.dashboard.logs.title')}</h3>
          <p className="diagnostics-tab__subtitle">{t('usage.dashboard.logs.subtitle')}</p>
        </div>
        <div className="diagnostics-tab__filter-rail">
          <label className="diagnostics-tab__filter-field">
            <span>{t('usage.dashboard.logs.filterLabel')}</span>
            <input
              defaultValue={ctx.logModelFilter}
              placeholder={t('usage.dashboard.logs.filterPlaceholder')}
              className="toolbar-select diagnostics-tab__filter-input"
              onChange={handleFilter}
              onKeyUp={handleKey}
            />
          </label>
          <button type="button" className="diagnostics-tab__filter-action" onClick={handleSearch}>
            {t('usage.dashboard.logs.search')}
          </button>
        </div>
      </div>
      <div className="diagnostics-tab__ledger" aria-busy={ctx.logsLoading}>
        <div className="diagnostics-tab__header diagnostics-tab__row">
          <div>{t('usage.dashboard.table.time')}</div>
          <div>{t('usage.dashboard.table.platform')}</div>
          <div>{t('usage.dashboard.table.model')}</div>
          <div className="is-right">{t('usage.dashboard.table.input')}</div>
          <div className="is-right">{t('usage.dashboard.table.output')}</div>
          <div className="is-right">{t('usage.dashboard.table.cost')}</div>
        </div>
        {ctx.logsLoading ? (
          <div className="diagnostics-tab__body" aria-hidden="true">
            {SKELETON_ROWS.map((key) => (
              <div key={key} className="diagnostics-tab__row diagnostics-tab__row--item">
                <span className="diagnostics-tab__skeleton" />
              </div>
            ))}
          </div>
        ) : records.length === 0 ? (
          <div className="diagnostics-tab__state">
            <strong className="diagnostics-tab__state-title">{ctx.diagnosticsEmptyMessage}</strong>
            <span className="diagnostics-tab__state-detail">{ctx.diagnosticsEmptyDetail}</span>
          </div>
        ) : (
          <div ref={parentRef} className="diagnostics-tab__body" style={{ height: '24rem', overflow: 'auto' }}>
            <div style={{ height: `${virtualizer.getTotalSize()}px`, position: 'relative' }}>
              {virtualizer.getVirtualItems().map((virtualRow) => {
                const record = records[virtualRow.index]
                if (!record) return null
                return (
                  <div
                    key={record.id}
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: '100%',
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                  >
                    <LogRow
                      row={{
                        id: record.id,
                        time: new Date(record.recorded_at).toLocaleString(),
                        platform: record.platform,
                        model: record.model || unknownModel,
                        input: ctx.formatTokens(record.input_tokens),
                        output: ctx.formatTokens(record.output_tokens),
                        cost: ctx.formatCost(record.cost_with_cache_usd),
                      }}
                    />
                  </div>
                )
              })}
            </div>
          </div>
        )}
      </div>
      {ctx.showLogsPager ? (
        <div className="diagnostics-tab__pager">
          <span className="diagnostics-tab__pager-status">
            {t('usage.dashboard.logs.pageStatus', {
              page: ctx.logsPage,
              pages: ctx.hasLogsTotal ? ctx.logsTotalPages : '?',
            })}
          </span>
          <button type="button" className="diagnostics-tab__pager-button" disabled={!ctx.canPrevLogs} onClick={handlePrev}>
            {t('usage.dashboard.logs.prev')}
          </button>
          <button type="button" className="diagnostics-tab__pager-button" disabled={!ctx.canNextLogs} onClick={handleNext}>
            {t('usage.dashboard.logs.next')}
          </button>
        </div>
      ) : null}
    </section>
  )
}
