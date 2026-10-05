import { getCurrentEnvironment } from '@/api/runtime/environment'

export type EnvironmentProbe = 'ok' | 'unsupported_environment'

/** Local-only 功能面的环境探针。非 local 时 base 展示 runtime-unavailable。 */
export async function probeLocalEnvironment(): Promise<EnvironmentProbe> {
  const environment = await getCurrentEnvironment()
  if (!environment || environment.env_type === 'local') return 'ok'
  return 'unsupported_environment'
}
