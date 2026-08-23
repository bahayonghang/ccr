import { memo } from 'react'
import type { ConverterResponse } from '@/types'
import { SIcon } from '@/ui'
import { t } from '../../locale'

interface ConverterResultProps {
  result: ConverterResponse
  onCopy: () => void
  onDownload: () => void
}

export const ConverterResult = memo(function ConverterResult({ result, onCopy, onDownload }: ConverterResultProps) {
  return (
    <div className="converter-results">
      <div className="converter-card">
        <h2 className="converter-card__title converter-card__title--section">{t('converter.conversionStats')}</h2>
        <div className="converter-stats-grid">
          <div className="converter-stat">
            <div className="converter-stat__value">{result.stats?.mcp_servers || 0}</div>
            <div className="converter-stat__label">{t('converter.mcpServersCount')}</div>
          </div>
          <div className="converter-stat">
            <div className="converter-stat__value">{result.stats?.slash_commands || 0}</div>
            <div className="converter-stat__label">{t('converter.slashCommandsCount')}</div>
          </div>
          <div className="converter-stat">
            <div className="converter-stat__value">{result.stats?.agents || 0}</div>
            <div className="converter-stat__label">{t('converter.agentsCount')}</div>
          </div>
          <div className="converter-stat">
            <div className="converter-stat__value">{result.stats?.profiles || 0}</div>
            <div className="converter-stat__label">{t('converter.profilesCount')}</div>
          </div>
          <div className="converter-stat">
            <div className="converter-stat__value">
              <SIcon name={result.stats?.base_config ? 'Check' : 'X'} size="w-6 h-6" className="mx-auto" />
            </div>
            <div className="converter-stat__label">{t('converter.baseConfig')}</div>
          </div>
        </div>
        {result.warnings && result.warnings.length > 0 ? (
          <div className="converter-warning-panel">
            <div className="converter-warning-panel__title">{t('converter.warnings')}</div>
            <ul className="converter-warning-list">
              {result.warnings.map((warning) => (
                <li key={warning} className="converter-warning-list__item">
                  {warning}
                </li>
              ))}
            </ul>
          </div>
        ) : null}
      </div>
      <div className="converter-card">
        <div className="converter-toolbar">
          <div>
            <h2 className="converter-card__title converter-card__title--compact">{t('converter.conversionResult')}</h2>
            <p className="converter-section-copy converter-section-copy--compact">
              {t('converter.resultFormat', { format: result.format?.toUpperCase() || '' })}
            </p>
          </div>
          <div className="converter-toolbar__actions">
            <button type="button" className="converter-toolbar-button converter-toolbar-button--label" onClick={onCopy}>
              <SIcon name="Copy" size="w-4 h-4" />
              {t('converter.copy')}
            </button>
            <button type="button" className="converter-toolbar-button converter-toolbar-button--label" onClick={onDownload}>
              <SIcon name="Download" size="w-4 h-4" />
              {t('converter.download')}
            </button>
          </div>
        </div>
        <textarea value={result.content} readOnly className="converter-textarea converter-textarea--result" />
      </div>
    </div>
  )
})
