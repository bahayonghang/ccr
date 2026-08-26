import type { ModelStat, ProjectStat, ProviderBreakdown } from '@/types/usage'
import type { TranslateFunction } from '@/utils/tf'
import { shortenPath } from '@/views/usage/usageOverviewInsights'
import { usageSourceFallbackLabel } from '@/views/usage/usageSources'
import { formatPercent } from '@/views/usage/usageSummaryCards'
import {
  getUsageTokenRowChartTotal,
  type UsageTokenBreakdownRow,
} from '@/views/usage/usageTokenBreakdown'
import type {
  UsageLedgerAlign,
  UsageLedgerCell,
  UsageLedgerColumn,
  UsageLedgerRowData,
  UsageLedgerShareCell,
  UsageLedgerStatusCell,
  UsageLedgerTextCell,
} from './UsageLedger'

export type UsageLedgerFormatters = {
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
}

const PRICING_STATUS_KEYS = {
  priced: 'usage.dashboard.table.statusPriced',
  static: 'usage.dashboard.table.statusStatic',
  snapshot: 'usage.dashboard.table.statusSnapshot',
  mixed: 'usage.dashboard.table.statusMixed',
  legacy_alias: 'usage.dashboard.table.statusLegacyAlias',
  unpriced: 'usage.dashboard.table.statusUnpriced',
} as const

type PricingStatus = keyof typeof PRICING_STATUS_KEYS

function isPricingStatus(value: string): value is PricingStatus {
  return value in PRICING_STATUS_KEYS
}

export function pricingStatusLabel(status: string, t: TranslateFunction): string {
  return isPricingStatus(status) ? t(PRICING_STATUS_KEYS[status]) : status
}

function textCell(
  id: string,
  text: string,
  extra?: { title?: string; secondary?: string; align?: UsageLedgerAlign },
): UsageLedgerTextCell {
  return {
    id,
    kind: 'text',
    text,
    title: extra?.title ?? text,
    secondary: extra?.secondary,
    align: extra?.align ?? 'start',
  }
}

function endCell(id: string, text: string): UsageLedgerTextCell {
  return textCell(id, text, { align: 'end' })
}

function shareCell(id: string, cost: number, totalCost: number): UsageLedgerShareCell {
  if (totalCost <= 0) {
    return { id, kind: 'share', text: '0%', align: 'end', ratio: 0 }
  }
  const ratio = cost / totalCost
  return { id, kind: 'share', text: formatPercent(ratio), align: 'end', ratio }
}

function statusCell(id: string, status: string, t: TranslateFunction): UsageLedgerStatusCell {
  return {
    id,
    kind: 'status',
    text: pricingStatusLabel(status, t),
    align: 'start',
    status,
  }
}

export function modelLedgerColumns(t: TranslateFunction): UsageLedgerColumn[] {
  return [
    { id: 'model', header: t('usage.dashboard.table.model'), align: 'start', colTemplate: 'minmax(14rem, 1.6fr)' },
    { id: 'requests', header: t('usage.dashboard.table.requests'), align: 'end', colTemplate: 'minmax(7rem, 0.7fr)' },
    { id: 'tokens', header: t('usage.dashboard.table.tokens'), align: 'end', colTemplate: 'minmax(8rem, 0.8fr)' },
    { id: 'cost', header: t('usage.dashboard.table.cost'), align: 'end', colTemplate: 'minmax(8rem, 0.8fr)' },
    { id: 'share', header: t('usage.dashboard.table.share'), align: 'end', colTemplate: 'minmax(8rem, 0.9fr)' },
    { id: 'pricing', header: t('usage.dashboard.table.pricingStatus'), align: 'start', colTemplate: 'minmax(8rem, 0.7fr)' },
  ]
}

export function projectLedgerColumns(t: TranslateFunction): UsageLedgerColumn[] {
  return [
    { id: 'project', header: t('usage.dashboard.table.project'), align: 'start', colTemplate: 'minmax(16rem, 1.8fr)' },
    { id: 'requests', header: t('usage.dashboard.table.requests'), align: 'end', colTemplate: 'minmax(7rem, 0.7fr)' },
    { id: 'tokens', header: t('usage.dashboard.table.tokens'), align: 'end', colTemplate: 'minmax(8rem, 0.8fr)' },
    { id: 'cost', header: t('usage.dashboard.table.cost'), align: 'end', colTemplate: 'minmax(8rem, 0.8fr)' },
    { id: 'share', header: t('usage.dashboard.table.share'), align: 'end', colTemplate: 'minmax(8rem, 0.9fr)' },
  ]
}

export function providerLedgerColumns(t: TranslateFunction): UsageLedgerColumn[] {
  return [
    { id: 'provider', header: t('usage.dashboard.table.provider'), align: 'start', colTemplate: 'minmax(14rem, 1.8fr)' },
    { id: 'requests', header: t('usage.dashboard.table.requests'), align: 'end', colTemplate: 'minmax(7rem, 0.7fr)' },
    { id: 'tokens', header: t('usage.dashboard.table.tokens'), align: 'end', colTemplate: 'minmax(8rem, 0.8fr)' },
    { id: 'cost', header: t('usage.dashboard.table.cost'), align: 'end', colTemplate: 'minmax(8rem, 0.8fr)' },
    { id: 'share', header: t('usage.dashboard.table.share'), align: 'end', colTemplate: 'minmax(8rem, 0.9fr)' },
  ]
}

export function tokenLedgerColumns(t: TranslateFunction): UsageLedgerColumn[] {
  return [
    { id: 'date', header: t('usage.dashboard.tokens.date'), align: 'start', colTemplate: 'minmax(10rem, 1.1fr)' },
    { id: 'input', header: t('usage.dashboard.table.input'), align: 'end', colTemplate: 'minmax(8rem, 0.9fr)' },
    { id: 'output', header: t('usage.dashboard.table.output'), align: 'end', colTemplate: 'minmax(8rem, 0.9fr)' },
    { id: 'cacheRead', header: t('usage.dashboard.table.cacheRead'), align: 'end', colTemplate: 'minmax(9rem, 1fr)' },
    { id: 'total', header: t('usage.dashboard.tokens.modes.total'), align: 'end', colTemplate: 'minmax(8rem, 0.9fr)' },
  ]
}

function modelCostOf(item: ModelStat): number {
  return item.cost_with_cache ?? item.total_cost
}

export function modelLedgerRows(
  stats: ModelStat[],
  formatters: UsageLedgerFormatters,
  t: TranslateFunction,
): UsageLedgerRowData[] {
  const totalCost = stats.reduce((sum, item) => sum + modelCostOf(item), 0)
  return stats.map((item) => {
    const cost = modelCostOf(item)
    const cells: UsageLedgerCell[] = [
      textCell('model', item.model),
      endCell('requests', item.request_count.toLocaleString()),
      endCell('tokens', formatters.formatTokens(item.total_tokens)),
      endCell('cost', formatters.formatCost(cost)),
      shareCell('share', cost, totalCost),
      statusCell('pricing', item.pricing_status, t),
    ]
    return { id: item.model, cells }
  })
}

export function projectLedgerRows(
  stats: ProjectStat[],
  formatters: UsageLedgerFormatters,
): UsageLedgerRowData[] {
  const totalCost = stats.reduce((sum, item) => sum + item.total_cost, 0)
  return stats.map((item) => ({
    id: item.project_path,
    cells: [
      textCell('project', shortenPath(item.project_path), {
        title: item.project_path,
        secondary: item.project_path,
      }),
      endCell('requests', item.request_count.toLocaleString()),
      endCell('tokens', formatters.formatTokens(item.total_tokens)),
      endCell('cost', formatters.formatCost(item.total_cost)),
      shareCell('share', item.total_cost, totalCost),
    ],
  }))
}

export function providerLedgerRows(
  stats: ProviderBreakdown[],
  formatters: UsageLedgerFormatters,
): UsageLedgerRowData[] {
  const totalCost = stats.reduce((sum, item) => sum + item.cost_with_cache_usd, 0)
  return stats.map((item) => {
    const providerKey = item.provider || 'unknown'
    const name = usageSourceFallbackLabel(providerKey)
    return {
      id: providerKey,
      cells: [
        textCell('provider', name),
        endCell('requests', item.request_count.toLocaleString()),
        endCell('tokens', formatters.formatTokens(item.total_tokens)),
        endCell('cost', formatters.formatCost(item.cost_with_cache_usd)),
        shareCell('share', item.cost_with_cache_usd, totalCost),
      ],
    }
  })
}

export function tokenLedgerRows(
  rows: UsageTokenBreakdownRow[],
  formatTokens: (value: number) => string,
): UsageLedgerRowData[] {
  return rows.map((row) => ({
    id: row.date,
    cells: [
      textCell('date', row.date),
      endCell('input', formatTokens(row.inputTokens)),
      endCell('output', formatTokens(row.assistantOutputTokens)),
      endCell('cacheRead', formatTokens(row.cacheReadTokens)),
      endCell('total', formatTokens(getUsageTokenRowChartTotal(row))),
    ],
  }))
}
