import { getGrokDashboardOverview as getGrokDashboardOverviewTyped } from '../generated/grok'
import type { GrokDashboardCommandResponse } from '@/types/grok'

export interface GrokDashboardRequestOptions {
  force?: boolean
}

export const isGrokDashboardResponse = (
  value: unknown,
): value is GrokDashboardCommandResponse => {
  if (!value || typeof value !== 'object' || !('status' in value)) return false
  const status = (value as { status?: unknown }).status
  return status === 'ok' || status === 'unsupported_environment'
}

// The force flag controls the composable cache. The generated command itself is read-only.
export const getGrokDashboardOverview = async (
  _options: GrokDashboardRequestOptions = {},
): Promise<GrokDashboardCommandResponse> => getGrokDashboardOverviewTyped()

// Profiles commands are added by the Grok Profiles child task.
// Settings commands are added by the Grok Settings child task.
