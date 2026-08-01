import type { DailyTrend, ModelStat, ProjectStat, UsageDashboardResponse, UsageSummary } from './usage'

export type PlatformUsageId = 'codex' | 'antigravity' | 'opencode'
export type PlatformUsageMetric = 'cost' | 'tokens' | 'requests'
export type PlatformUsageTone = 'codex' | 'antigravity' | 'opencode' | 'neutral'
export type PlatformUsagePricingState = 'available' | 'token_only' | 'empty'

export interface PlatformUsageInsightLabels {
  costCard: string
  tokensCard: string
  requestsCard: string
  cost: string
  tokens: string
  requests: string
  input: string
  output: string
  cacheRead: string
  cacheCreation: string
  models: string
  projects: string
  topModel: string
  topProject: string
  noModel: string
  noProject: string
  pricingUnavailable: string
  tokenOnly: string
  requestUnit: string
  modelUnit: string
  projectUnit: string
  generatedAt: string
}

export interface PlatformUsageInsightSpec {
  platform: PlatformUsageId
  label: string
  tone: PlatformUsageTone
  eyebrow: string
  title: string
  description: string
  windowLabel: string
  sourceLabel: string
  primaryActionLabel: string
  primaryActionTo: string
  emptyTitle: string
  emptyDescription: string
  errorTitle: string
  retryLabel: string
  tabs: Record<PlatformUsageMetric | 'breakdown', string>
  modelRankTitle: string
  projectRankTitle: string
}

export interface PlatformUsageKpiCard {
  id: 'cost' | 'tokens' | 'requests'
  label: string
  value: string
  detail: string
  meta: string
  icon: string
  tone: PlatformUsageTone
  pricingState?: PlatformUsagePricingState
}

export interface PlatformUsageRankRow {
  id: string
  label: string
  title: string
  detail: string
  value: number
  displayValue: string
  share: number
}

export interface PlatformUsageInsightPresentation {
  summary: UsageSummary | null
  trends: DailyTrend[]
  modelStats: ModelStat[]
  projectStats: ProjectStat[]
  cards: PlatformUsageKpiCard[]
  modelRows: PlatformUsageRankRow[]
  projectRows: PlatformUsageRankRow[]
  topModelLabel: string
  topProjectLabel: string
  pricingState: PlatformUsagePricingState
  empty: boolean
  generatedAt: string | null
}

export type PlatformUsageDashboardData = Pick<
  UsageDashboardResponse,
  'summary' | 'trends' | 'model_stats' | 'project_stats' | 'generated_at'
>
