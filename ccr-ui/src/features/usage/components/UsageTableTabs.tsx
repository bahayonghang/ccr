import { useMemo } from 'react'
import { useUsageDashboardContext } from '../UsageDashboardContext'
import { useUsageT } from '../translate'
import { UsageLedger } from './UsageLedger'
import {
  modelLedgerColumns,
  modelLedgerRows,
  projectLedgerColumns,
  projectLedgerRows,
  providerLedgerColumns,
  providerLedgerRows,
} from './usageLedgerRows'
import '../styles/usage-models-tab.css'
import '../styles/usage-projects-tab.css'
import '../styles/usage-providers-tab.css'

export function UsageModelsTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  const columns = useMemo(() => modelLedgerColumns(t), [t])
  const rows = useMemo(
    () => modelLedgerRows(
      ctx.modelStats,
      { formatCost: ctx.formatCost, formatTokens: ctx.formatTokens },
      t,
    ),
    [ctx.formatCost, ctx.formatTokens, ctx.modelStats, t],
  )

  return (
    <section className="models-tab glass-panel rounded-xl p-4">
      <h3>{t('usage.dashboard.models.title')}</h3>
      {rows.length === 0 ? (
        <div className="models-tab__empty">{t('usage.dashboard.table.noData')}</div>
      ) : (
        <UsageLedger columns={columns} rows={rows} />
      )}
    </section>
  )
}

export function UsageProjectsTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  const columns = useMemo(() => projectLedgerColumns(t), [t])
  const rows = useMemo(
    () => projectLedgerRows(ctx.projectStats, {
      formatCost: ctx.formatCost,
      formatTokens: ctx.formatTokens,
    }),
    [ctx.formatCost, ctx.formatTokens, ctx.projectStats],
  )

  return (
    <section className="projects-tab glass-panel rounded-xl p-4">
      <h3>{t('usage.dashboard.projects.title')}</h3>
      {rows.length === 0 ? (
        <div className="projects-tab__empty">{t('usage.dashboard.table.noData')}</div>
      ) : (
        <UsageLedger columns={columns} rows={rows} />
      )}
    </section>
  )
}

export function UsageProvidersTab() {
  const ctx = useUsageDashboardContext()
  const t = useUsageT()
  const columns = useMemo(() => providerLedgerColumns(t), [t])
  const rows = useMemo(
    () => providerLedgerRows(ctx.providerStats, {
      formatCost: ctx.formatCost,
      formatTokens: ctx.formatTokens,
    }),
    [ctx.formatCost, ctx.formatTokens, ctx.providerStats],
  )

  return (
    <section className="providers-tab glass-panel rounded-xl p-4">
      <h3>{t('usage.dashboard.providers.title')}</h3>
      {rows.length === 0 ? (
        <div className="providers-tab__empty">{t('usage.dashboard.table.noData')}</div>
      ) : (
        <UsageLedger columns={columns} rows={rows} />
      )}
    </section>
  )
}
