import { useUsageDashboardContext } from '../UsageDashboardContext'
import { useUsageT } from '../translate'
import { usageSourceFallbackLabel } from '@/views/usage/usageSources'
import '../styles/usage-models-tab.css'
import '../styles/usage-projects-tab.css'
import '../styles/usage-providers-tab.css'

export function UsageModelsTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  return (
    <section className="models-tab glass-panel rounded-xl p-4">
      <h3>{t('usage.dashboard.models.title')}</h3>
      <div className="models-tab__table">
        {ctx.modelStats.map((item) => (
          <article key={item.model} className="models-tab__row">
            <strong title={item.model}>{item.model}</strong>
            <span>{ctx.formatTokens(item.total_tokens)}</span>
            <span>{ctx.formatCost(item.cost_with_cache ?? item.total_cost)}</span>
          </article>
        ))}
      </div>
    </section>
  )
}

export function UsageProjectsTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  return (
    <section className="projects-tab glass-panel rounded-xl p-4">
      <h3>{t('usage.dashboard.projects.title')}</h3>
      <div>
        {ctx.projectStats.map((item) => (
          <article key={item.project_path} className="projects-tab__row">
            <strong title={item.project_path}>{item.project_path}</strong>
            <span>{ctx.formatTokens(item.total_tokens)}</span>
            <span>{ctx.formatCost(item.total_cost)}</span>
          </article>
        ))}
      </div>
    </section>
  )
}

export function UsageProvidersTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  return (
    <section className="providers-tab glass-panel rounded-xl p-4">
      <h3>{t('usage.dashboard.providers.title')}</h3>
      <div>
        {ctx.providerStats.map((item) => (
          <article key={item.provider ?? 'unknown'} className="providers-tab__row">
            <strong>{usageSourceFallbackLabel(item.provider ?? 'unknown')}</strong>
            <span>{ctx.formatTokens(item.total_tokens)}</span>
            <span>{ctx.formatCost(item.cost_with_cache_usd)}</span>
          </article>
        ))}
      </div>
    </section>
  )
}
