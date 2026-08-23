import type { CodexTf } from '../useCodexLocale'
import type { AccountNameValidationMessage } from '../codexAuthAccounts'

export function preferredNameErrorText(message: AccountNameValidationMessage | null, tf: CodexTf): string | null {
  if (message === 'reserved') {
    return tf('codex.auth.naming.validation.reserved', '"default" is reserved. Please choose another account name.')
  }
  if (message === 'length') {
    return tf('codex.auth.naming.validation.length', 'Account names must stay within 32 characters.')
  }
  if (message === 'charset') {
    return tf('codex.auth.naming.validation.charset', 'Use letters, numbers, underscores, and hyphens only.')
  }
  return null
}
