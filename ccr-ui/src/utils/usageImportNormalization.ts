import type {
  ImportAllUsageResponse,
  UsageImportResult,
  UsagePlatform,
  UsageImportJobSnapshot,
  UsageImportJobStatus,
  UsageImportSummary,
} from '@/types/usage'

const terminalImportStatuses: Record<UsageImportJobStatus, boolean> = {
  pending: false,
  running: false,
  recent_ready: false,
  cancel_requested: false,
  finished: true,
  failed: true,
  cancelled: true,
  timed_out: true,
  cleanup_failed: true,
}

export const isUsageImportJobTerminal = (job: UsageImportJobSnapshot | null): boolean =>
  job !== null && terminalImportStatuses[job.status]

export const isUsageImportJobFailed = (status: UsageImportJobStatus): boolean =>
  ['failed', 'timed_out', 'cleanup_failed'].includes(status)

export const shouldApplyUsageImportJob = (
  current: UsageImportJobSnapshot | null,
  next: UsageImportJobSnapshot,
): boolean => current?.job_id !== next.job_id || (
  !isUsageImportJobTerminal(current) && next.updated_at >= current.updated_at
)

export const isOptionalAbsentImportResult = (result: UsageImportResult): boolean =>
  result.is_optional_absent === true

export const toUserVisibleImportResult = (result: UsageImportResult): UsageImportResult => {
  if (!isOptionalAbsentImportResult(result)) return result

  return {
    ...result,
    completed: true,
    error: null,
  }
}

export const normalizeUserVisibleImportResults = (results: UsageImportResult[]): UsageImportResult[] =>
  results.map(toUserVisibleImportResult)

export const normalizeUserVisibleImportJob = (
  job: UsageImportJobSnapshot,
): UsageImportJobSnapshot => {
  const results = normalizeUserVisibleImportResults(job.results)

  return {
    ...job,
    results,
  }
}

export const buildImportSummary = (results: UsageImportResult[]): UsageImportSummary => {
  const successCount = results.filter(result => !result.error).length
  const failureCount = results.length - successCount
  const importedRecords = results.reduce((sum, result) => sum + result.records_imported, 0)
  const processedFiles = results.reduce((sum, result) => sum + result.files_processed, 0)
  const hasPartial = results.some(result =>
    Boolean(result.error)
    || !result.completed
    || (result.files_processed > 0 && result.records_imported === 0 && result.records_skipped > 0),
  )

  return {
    success_count: successCount,
    failure_count: failureCount,
    imported_records: importedRecords,
    processed_files: processedFiles,
    has_partial: hasPartial,
  }
}

export const getUsageImportJobSummary = (job: UsageImportJobSnapshot): UsageImportSummary | null =>
  job.summary ?? (job.results.length > 0 ? buildImportSummary(job.results) : null)

export const isImportAllUsageResponse = (
  payload: UsageImportResult | ImportAllUsageResponse,
): payload is ImportAllUsageResponse => {
  return 'results' in payload && Array.isArray(payload.results)
}

export const normalizeImportResponse = (
  payload: UsageImportResult | ImportAllUsageResponse,
  platformOverride?: UsagePlatform,
): ImportAllUsageResponse => {
  if (isImportAllUsageResponse(payload)) {
    const results = normalizeUserVisibleImportResults(payload.results)
    return {
      ...payload,
      results,
      summary: buildImportSummary(results),
    }
  }

  const result: UsageImportResult = platformOverride
    ? { ...payload, platform: platformOverride }
    : payload
  const results = normalizeUserVisibleImportResults([result])
  return {
    results,
    summary: buildImportSummary(results),
  }
}
