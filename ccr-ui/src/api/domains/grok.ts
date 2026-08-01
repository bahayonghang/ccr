import * as grokClient from '../generated/grok'
import { toOpenJsonValue } from '../_shared'
import type {
  GrokDashboardCommandResponse,
  GrokProfileActionResponse,
  GrokProfileCommandResponse,
  GrokProfileCreateRequest,
  GrokProfilePatch,
  GrokProfilesCommandResponse,
} from '@/types/grok'

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
): Promise<GrokDashboardCommandResponse> => grokClient.getGrokDashboardOverview()

const assertStatus = <T extends { status: string }>(
  response: T,
  allowed: readonly string[],
  label: string,
): T => {
  if (!allowed.includes(response.status)) throw new Error(`${label} response is invalid`)
  return response
}

export const listGrokProfiles = async (): Promise<GrokProfilesCommandResponse> => (
  assertStatus(await grokClient.listGrokProfiles(), ['ok', 'unsupported_environment'], 'Grok profiles')
)

export const getGrokProfile = async (name: string): Promise<GrokProfileCommandResponse> => (
  assertStatus(await grokClient.getGrokProfile(name), ['ok', 'unsupported_environment'], 'Grok profile')
)

export const addGrokProfile = async (
  request: GrokProfileCreateRequest,
): Promise<GrokProfileActionResponse> => assertStatus(
  await grokClient.addGrokProfile(toOpenJsonValue(request, 'Grok profile create payload')),
  ['created', 'unsupported_environment'],
  'Grok profile create',
)

export const updateGrokProfile = async (
  name: string,
  patch: GrokProfilePatch,
): Promise<GrokProfileActionResponse> => assertStatus(
  await grokClient.updateGrokProfile(name, toOpenJsonValue(patch, 'Grok profile patch payload')),
  [
    'updated',
    'renamed',
    'rename_apply_failed',
    'rename_cleanup_failed',
    'unsupported_environment',
  ],
  'Grok profile update',
)

export const deleteGrokProfile = async (
  name: string,
  options: { force?: boolean } = {},
): Promise<GrokProfileActionResponse> => assertStatus(
  await grokClient.deleteGrokProfile(name, options),
  ['deleted', 'blocked', 'unsupported_environment'],
  'Grok profile delete',
)

export const applyGrokProfile = async (name: string): Promise<GrokProfileActionResponse> => (
  assertStatus(
    await grokClient.applyGrokProfile(name),
    ['applied', 'unsupported_environment'],
    'Grok profile apply',
  )
)

export const grokProfileOff = async (): Promise<GrokProfileActionResponse> => assertStatus(
  await grokClient.grokProfileOff(),
  ['off', 'unsupported_environment'],
  'Grok profile off',
)

// Settings commands are added by the Grok Settings child task.
