import type { LoginState } from '@/types'
import type { TranslateFunction } from '@/utils/tf'
import type { CodexTf } from '../useCodexLocale'

export function loginStateText(state: LoginState | null | undefined, t: TranslateFunction, tf: CodexTf): string {
  if (!state) return t('codex.auth.loginState.notLoggedIn')
  if (state.type === 'LoggedInSaved') {
    return tf('codex.auth.loginState.loggedInSaved', 'Logged in ({name})', { name: state.account_name })
  }
  if (state.type === 'LoggedInUnsaved') return t('codex.auth.loginState.loggedInUnsaved')
  if (state.type === 'ApiKeyActive') return t('codex.auth.loginState.apiKeyActive')
  if (state.type === 'ProviderKeyActive') {
    return tf('codex.auth.loginState.providerKeyActive', 'Provider Key ({envKey})', { envKey: state.env_key })
  }
  if (state.type === 'Unknown') {
    return tf('codex.auth.loginState.unknown', 'Unknown state ({type})', { type: state.raw_type })
  }
  return t('codex.auth.loginState.notLoggedIn')
}
