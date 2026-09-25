import { useEffect, useState } from 'react'
import type { HomeInsightsResponse } from '@/types/generated/usage/HomeInsightsResponse'
import { SIcon } from '@/ui'
import { getErrorMessage } from '@/utils/errorHandler'
import { scheduleWhenIdle } from '@/utils/scheduling'
import { useHomeInsights } from '../../queries'
import { useUsageT } from '../../translate'
import { InsightsActivityHeatmap } from './InsightsActivityHeatmap'
import { InsightsAgentTrend } from './InsightsAgentTrend'
import { InsightsDistribution } from './InsightsDistribution'
import { InsightsLeaderboards } from './InsightsLeaderboards'
import { InsightsTotalsLine } from './InsightsTotalsLine'
import { InsightsWeekCompare } from './InsightsWeekCompare'
import { insightsFlags } from './insightsPresentation'
import '../../styles/dashboard-insights.css'

interface DashboardInsightsProps {
  isNativeRuntime: boolean
  onImport: () => void
  importing: boolean
}

// 首页 Insights 容器：首屏之后（空闲时）取数，按状态分派，子区块只渲染。
export function DashboardInsights({
  isNativeRuntime,
  onImport,
  importing,
}: DashboardInsightsProps) {
  const t = useUsageT()
  const [idle, setIdle] = useState(false)
  useEffect(() => {
    if (!isNativeRuntime) return undefined
    return scheduleWhenIdle(() => setIdle(true), { timeout: 2400, fallbackDelay: 600 })
  }, [isNativeRuntime])
  const query = useHomeInsights({ enabled: isNativeRuntime && idle })
  const data = isNativeRuntime ? (query.data ?? null) : null

  return (
    <section
      className="dashboard-insights"
      aria-labelledby="dashboard-insights-title"
      data-dashboard-insights
    >
      <header className="dashboard-insights__header">
        <div className="dashboard-insights__lede">
          <h2 id="dashboard-insights-title" className="dashboard-insights__title">
            {t('dashboard.insights.title')}
          </h2>
          <p className="dashboard-insights__description">{t('dashboard.insights.description')}</p>
        </div>
        {data ? (
          <span className="dashboard-insights__range insights-num">{`${data.trend_start} – ${data.as_of}`}</span>
        ) : null}
      </header>
      <InsightsBody
        isNativeRuntime={isNativeRuntime}
        data={data}
        error={query.error ? getErrorMessage(query.error) : null}
        actions={{ onImport, importing }}
      />
    </section>
  )
}

interface InsightsBodyProps {
  isNativeRuntime: boolean
  data: HomeInsightsResponse | null
  error: string | null
  actions: { onImport: () => void; importing: boolean }
}

function InsightsBody({ isNativeRuntime, data, error, actions }: InsightsBodyProps) {
  const t = useUsageT()
  if (!isNativeRuntime) {
    return (
      <div className="insights-panel insights-state" data-state="web-preview">
        <SIcon name="Info" size="w-4 h-4" className="insights-notice__dot" />
        <div>
          <p className="insights-state__title">{t('dashboard.insights.webPreview.title')}</p>
          <p className="insights-caption">{t('dashboard.insights.webPreview.description')}</p>
        </div>
      </div>
    )
  }
  if (error && !data) {
    return (
      <div className="insights-panel insights-state" data-state="error" role="alert">
        <p className="insights-state__title">{t('dashboard.insights.errorTitle')}</p>
        <p className="insights-caption">{error}</p>
      </div>
    )
  }
  if (!data) {
    return (
      <div className="insights-panel insights-state" data-state="loading" aria-busy="true">
        <p className="insights-caption">{t('dashboard.insights.loading')}</p>
      </div>
    )
  }
  const flags = insightsFlags(data)
  if (flags.emptyStore) {
    return (
      <>
        <InsightsTotalsLine data={null} />
        <div className="insights-empty" data-state="empty">
          <p className="insights-state__title">{t('dashboard.insights.empty.title')}</p>
          <p className="insights-caption">{t('dashboard.insights.empty.description')}</p>
          <button
            type="button"
            className="insights-import insights-import--primary"
            onClick={actions.onImport}
            disabled={actions.importing}
          >
            {actions.importing
              ? t('dashboard.insights.empty.importing')
              : t('dashboard.insights.empty.import')}
          </button>
        </div>
      </>
    )
  }
  return (
    <>
      <InsightsTotalsLine data={data} notice={flags.usageOnly} />
      <div className="insights-row insights-row--wide">
        <InsightsActivityHeatmap
          data={data}
          onImport={actions.onImport}
          importing={actions.importing}
        />
        <InsightsWeekCompare data={data} />
      </div>
      {flags.hasRequests ? <InsightsAgentTrend data={data} /> : null}
      <div className={flags.hasRequests ? 'insights-row' : 'insights-row insights-row--single'}>
        {flags.hasRequests ? <InsightsDistribution data={data} /> : null}
        <InsightsLeaderboards data={data} />
      </div>
    </>
  )
}
