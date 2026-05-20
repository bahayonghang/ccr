import type {
  PlatformUsageId,
  PlatformUsageInsightLabels,
  PlatformUsageInsightSpec,
  PlatformUsageTone,
} from '@/types/platformUsageInsight'

type Translate = (key: string) => string

const platformToneMap: Record<PlatformUsageId, PlatformUsageTone> = {
  codex: 'codex',
  gemini: 'antigravity',
  opencode: 'opencode',
}

const platformCopyKeyMap: Record<PlatformUsageId, 'codex' | 'antigravity' | 'opencode'> = {
  codex: 'codex',
  gemini: 'antigravity',
  opencode: 'opencode',
}

export const buildPlatformUsageI18nLabels = (t: Translate): Partial<PlatformUsageInsightLabels> => ({
  costCard: t('platformUsage.cards.cost'),
  tokensCard: t('platformUsage.cards.tokens'),
  requestsCard: t('platformUsage.cards.requests'),
  cost: t('platformUsage.metrics.cost'),
  tokens: t('platformUsage.metrics.tokens'),
  requests: t('platformUsage.metrics.requests'),
  input: t('platformUsage.metrics.input'),
  output: t('platformUsage.metrics.output'),
  cacheRead: t('platformUsage.metrics.cacheRead'),
  cacheCreation: t('platformUsage.metrics.cacheCreation'),
  models: t('platformUsage.metrics.models'),
  projects: t('platformUsage.metrics.projects'),
  topModel: t('platformUsage.metrics.topModel'),
  topProject: t('platformUsage.metrics.topProject'),
  noModel: t('platformUsage.empty.noModel'),
  noProject: t('platformUsage.empty.noProject'),
  pricingUnavailable: t('platformUsage.empty.pricingUnavailable'),
  tokenOnly: t('platformUsage.empty.tokenOnly'),
  requestUnit: t('platformUsage.units.requests'),
  modelUnit: t('platformUsage.units.models'),
  projectUnit: t('platformUsage.units.projects'),
  generatedAt: t('platformUsage.generatedAt'),
})

export const buildPlatformUsageSpec = (
  t: Translate,
  platform: PlatformUsageId,
): PlatformUsageInsightSpec => {
  const copyKey = platformCopyKeyMap[platform]

  return {
    platform,
    label: t(`platformUsage.platforms.${copyKey}.label`),
    tone: platformToneMap[platform],
    eyebrow: t('platformUsage.eyebrow'),
    title: t(`platformUsage.platforms.${copyKey}.title`),
    description: t(`platformUsage.platforms.${copyKey}.description`),
    windowLabel: t('platformUsage.window30'),
    sourceLabel: t('platformUsage.source'),
    primaryActionLabel: t('platformUsage.openDashboard'),
    primaryActionTo: `/usage?platform=${platform}`,
    emptyTitle: t(`platformUsage.platforms.${copyKey}.emptyTitle`),
    emptyDescription: t(`platformUsage.platforms.${copyKey}.emptyDescription`),
    errorTitle: t('platformUsage.errorTitle'),
    retryLabel: t('platformUsage.retry'),
    tabs: {
      cost: t('platformUsage.tabs.cost'),
      tokens: t('platformUsage.tabs.tokens'),
      requests: t('platformUsage.tabs.requests'),
      breakdown: t('platformUsage.tabs.breakdown'),
    },
    modelRankTitle: t('platformUsage.rank.models'),
    projectRankTitle: t('platformUsage.rank.projects'),
  }
}
