import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import { SIcon } from '@/ui'
import { useUsageT } from '../../translate'
import { compactLabel } from '../DashboardCostMetric'
import { buildTotals, EM_DASH, fullNumber, type TotalsItem } from './insightsPresentation'

const LABEL_KEYS: Record<TotalsItem['key'], string> = {
  sessions: 'dashboard.insights.totals.sessions',
  tokens: 'dashboard.insights.totals.tokens',
  requests: 'dashboard.insights.totals.requests',
  agents: 'dashboard.insights.totals.agents',
  projects: 'dashboard.insights.totals.projects',
  activeDays: 'dashboard.insights.totals.activeDays',
}

interface InsightsTotalsLineProps {
  /** null：空库 / Web 预览，所有值显示 `—` */
  data: HomeInsightsResponse | null
  /** 会话未索引但有用量：显示索引说明行（由 insightsFlags.usageOnly 给出）。 */
  notice?: boolean
}

export function InsightsTotalsLine({ data, notice = false }: InsightsTotalsLineProps) {
  const t = useUsageT()
  const items: TotalsItem[] = data
    ? buildTotals(data)
    : (Object.keys(LABEL_KEYS) as TotalsItem['key'][]).map((key) => ({
        key,
        value: null,
        unindexed: false,
      }))
  const unindexedTitle = t('dashboard.insights.unindexedTitle')

  return (
    <div className="insights-totals-wrap">
      <dl className="insights-totals" aria-label={t('dashboard.insights.totals.ariaLabel')}>
        {items.map((item) => (
          <div key={item.key} className="insights-totals__group">
            <dt className="insights-totals__label">{t(LABEL_KEYS[item.key])}</dt>
            <dd
              className="insights-totals__value"
              title={
                item.value === null
                  ? item.unindexed
                    ? unindexedTitle
                    : undefined
                  : fullNumber(item.value)
              }
            >
              {item.value === null ? EM_DASH : compactLabel(null, item.value)}
            </dd>
          </div>
        ))}
      </dl>
      {data && notice ? (
        <p className="insights-notice">
          <SIcon name="Info" size="w-3.5 h-3.5" className="insights-notice__dot" />
          {t('dashboard.insights.unindexedNotice')}
        </p>
      ) : null}
    </div>
  )
}
