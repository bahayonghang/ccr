import type { ModelStat, Platform, UsageArchiveDiagnostics, UsageRecordV2, UsageSummary } from '@/types/usage'

export type UsageDiagnosticsSummary = {
  totalRecords: string
  latestRecordAt: string
  healthLabel: string
  healthDetail: string
  repairRecommended: boolean
  canRepairCodex: boolean
}

export type UsageDiagnosticsMessages = {
  noRecentRecord: string
  rawLogsHint: string
  repairNeeded: string
  codexRepairHint: (unknownRequests: string) => string
  healthy: string
}

export const formatArchiveDetail = (
  archive: UsageArchiveDiagnostics | null | undefined,
  rawLogsHint: string,
) => archive
  ? `Archive ${archive.archived_sessions} · L ${archive.live_sources} / M ${archive.missing_sources} / D ${archive.deleted_sources}`
  : rawLogsHint

export const shouldRecommendCodexRepair = ({
  selectedPlatform,
  summary,
  unknownModelStat,
}: {
  selectedPlatform: string
  summary: UsageSummary | null | undefined
  unknownModelStat: ModelStat | null | undefined
}) => {
  if (selectedPlatform !== 'codex') return false
  if (!summary || summary.total_requests <= 0) return false

  return (unknownModelStat?.request_count ?? 0) > 0 || summary.total_cost_usd === 0
}

export const buildUsageDiagnosticsSummary = ({
  selectedPlatform,
  summary,
  logsRecords,
  logsTotalCount,
  unknownModelStat,
  archive,
  locale,
  messages,
}: {
  selectedPlatform: Platform | ''
  summary: UsageSummary | null | undefined
  logsRecords: UsageRecordV2[]
  logsTotalCount: number
  unknownModelStat: ModelStat | null | undefined
  archive: UsageArchiveDiagnostics | null | undefined
  locale: string
  messages: UsageDiagnosticsMessages
}): UsageDiagnosticsSummary => {
  const latestTimestamp = logsRecords[0]?.recorded_at ?? null
  const latestRecordAt = latestTimestamp
    ? new Date(latestTimestamp).toLocaleString(locale)
    : messages.noRecentRecord
  const archiveDetail = formatArchiveDetail(archive, messages.rawLogsHint)
  const repairRecommended = shouldRecommendCodexRepair({
    selectedPlatform,
    summary,
    unknownModelStat,
  })

  if (repairRecommended) {
    return {
      totalRecords: logsTotalCount.toLocaleString(),
      latestRecordAt,
      healthLabel: messages.repairNeeded,
      healthDetail: `${messages.codexRepairHint((unknownModelStat?.request_count ?? 0).toLocaleString())} · ${archiveDetail}`,
      repairRecommended: true,
      canRepairCodex: true,
    }
  }

  return {
    totalRecords: logsTotalCount.toLocaleString(),
    latestRecordAt,
    healthLabel: messages.healthy,
    healthDetail: archiveDetail,
    repairRecommended: false,
    canRepairCodex: selectedPlatform === 'codex',
  }
}
