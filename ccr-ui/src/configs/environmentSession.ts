import type { QueryClient } from '@tanstack/react-query'
import { getCurrentEnvironment, type EnvironmentInfo } from '@/api/runtime/environment'

export const currentEnvironmentKey = ['current-environment'] as const

export function environmentRevision(client: QueryClient) {
  return client.getQueryState(currentEnvironmentKey)?.dataUpdateCount
}

export function sameEnvironment(left: EnvironmentInfo | undefined, right: EnvironmentInfo | undefined) {
  return Boolean(left && right && left.id === right.id && left.env_type === right.env_type)
}

export class EnvironmentSessionError extends Error {
  constructor() {
    super('settingsSession.environmentChanged')
    this.name = 'EnvironmentSessionError'
  }
}

/** Reads and drafts are valid only while their acknowledged environment is current. */
export function environmentIsReady(client: QueryClient, expected: EnvironmentInfo | undefined) {
  const current = client.getQueryState<EnvironmentInfo>(currentEnvironmentKey)
  return current?.status === 'success' && current.fetchStatus === 'idle'
    && !current.isInvalidated && sameEnvironment(current.data, expected)
}

export async function verifyEnvironment(client: QueryClient, expected: EnvironmentInfo) {
  if (!environmentIsReady(client, expected)) throw new EnvironmentSessionError()
  const actual = await getCurrentEnvironment()
  if (!sameEnvironment(actual, expected)) {
    client.setQueryData(currentEnvironmentKey, actual)
    throw new EnvironmentSessionError()
  }
  if (!environmentIsReady(client, expected)) throw new EnvironmentSessionError()
}
