/**
 * Stats domain compatibility surface.
 *
 * Usage V2 commands are registry-generated. Pricing remains legacy JSON until
 * that command module is migrated.
 */

import { invoke } from '@/api/invokeRuntime'
import { asRecord, type UnknownRecord } from '../_shared'

export {
  cancelUsageImportJobV2,
  ensureSessionIndexV2,
  getHomeUsageOverviewV2,
  getSessionIndexJobStatusV2,
  getUsageByModelV2,
  getUsageByProjectV2,
  getUsageByProviderV2,
  getUsageCapabilitiesV2,
  getUsageDashboardV2,
  getUsageHeatmapV2,
  getUsageImportJobStatusV2,
  getUsageLogsV2,
  getUsageSummaryV2,
  getUsageTrendsV2,
  importAllUsageV2,
  importUsageV2,
  startUsageImportJobV2,
} from '../generated/usageV2'
export type { UsageLogsQuery } from '../generated/usageV2'

/** Set pricing while accepting both data.model and data.name. */
export const setPricing = async <T = UnknownRecord>(data: unknown): Promise<T> => {
  const source = asRecord(data)
  const model = String(source.model ?? source.name ?? '')
  return invoke('set_pricing', { model, pricing: data })
}

export const getPricingList = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('get_pricing_list')
}

export const removePricing = async <T = UnknownRecord>(model: string): Promise<T> => {
  return invoke('remove_pricing', { model })
}

export const resetPricing = async <T = UnknownRecord>(): Promise<T> => {
  return invoke('reset_pricing')
}
