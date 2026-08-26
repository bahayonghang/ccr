import type { ProfileWriteOutcome } from '@/configs/profileEditorAdapter'
import type { GrokProfileActionResponse } from '@/types'

const appliedNameOf = (response: GrokProfileActionResponse): string | undefined => {
  switch (response.status) {
    case 'created':
    case 'updated':
      return response.profile.name
    case 'renamed':
      return response.new_name
    case 'rename_apply_failed':
    case 'rename_cleanup_failed':
    case 'deleted':
    case 'blocked':
    case 'applied':
    case 'off':
    case 'unsupported_environment':
      return undefined
    default: {
      const _never: never = response
      return _never
    }
  }
}

/** 把 Grok 写入响应映射为外壳可消费的 `ProfileWriteOutcome`，不拼接表单密钥。 */
export const mapGrokProfileWriteOutcome = (
  response: GrokProfileActionResponse,
): ProfileWriteOutcome => {
  switch (response.status) {
    case 'created':
    case 'updated':
    case 'renamed':
      return { status: 'ok', appliedName: appliedNameOf(response) }
    case 'rename_apply_failed':
    case 'rename_cleanup_failed':
      return { status: 'recovery', kind: response.status, message: response.message }
    case 'blocked':
      return {
        status: 'blocked',
        message: response.message,
        forceAllowed: response.reason !== 'unsafe_missing_entry_state',
      }
    case 'unsupported_environment':
      return { status: 'error', message: response.env_type }
    case 'deleted':
    case 'applied':
    case 'off':
      return { status: 'error', message: response.status }
    default: {
      const _never: never = response
      return { status: 'error', message: String(_never) }
    }
  }
}
