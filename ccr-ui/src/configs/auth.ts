import { grokAuthCurrent, grokAuthOff } from '@/api/domains/grok'
import { claudeAuthOff, getClaudeAuthCurrent } from '@/api/domains/claude'
import { codexAuthOff, getCodexAuthCurrent } from '@/api/domains/codex'
import { probeLocalEnvironment, type EnvironmentProbe } from '@/configs/probeLocal'
import { surfaceNotify, type SurfaceNotify } from '@/configs/surfaceNotify'

export interface AuthSessionState {
  loggedIn: boolean
  canAuthOff: boolean
  detail?: string
}

export interface AuthOffResult {
  changed: boolean
  unsupported?: boolean
}

export interface AuthSessionConfig {
  cacheKey: string
  homePath: string
  module: string
  i18nPrefix: string
  titleKey: string
  subtitleKey: string
  confirmOffKey: string
  sessionFileLabelKey?: string
  features: { localOnly?: boolean }
  notify: SurfaceNotify
  probe?: () => Promise<EnvironmentProbe>
  load: () => Promise<AuthSessionState>
  authOff: () => Promise<AuthOffResult>
}

const asRecord = (value: unknown): Record<string, unknown> =>
  value && typeof value === 'object' ? (value as Record<string, unknown>) : {}

export const grokAuthConfig: AuthSessionConfig = {
  cacheKey: 'auth-grok',
  homePath: '/grok/auth',
  module: 'grok',
  i18nPrefix: 'grok.auth',
  titleKey: 'grok.auth.title',
  subtitleKey: 'grok.auth.subtitle',
  confirmOffKey: 'auth.confirmOffGrok',
  sessionFileLabelKey: 'grok.auth.sessionFile',
  features: { localOnly: true },
  notify: surfaceNotify,
  probe: probeLocalEnvironment,
  load: async () => {
    const response = await grokAuthCurrent()
    if (response.status === 'unsupported_environment') {
      return { loggedIn: false, canAuthOff: false }
    }
    return { loggedIn: response.logged_in, canAuthOff: response.can_auth_off }
  },
  authOff: async () => {
    const result = await grokAuthOff()
    if (result.status === 'unsupported_environment') return { changed: false, unsupported: true }
    return { changed: result.changed }
  },
}

export const claudeAuthSessionConfig: AuthSessionConfig = {
  cacheKey: 'auth-claude-session',
  homePath: '/claude-code/auth',
  module: 'claude-code',
  i18nPrefix: 'claude.auth',
  titleKey: 'claude.auth.title',
  subtitleKey: 'claude.auth.subtitle',
  confirmOffKey: 'auth.confirmOffClaude',
  features: {},
  notify: surfaceNotify,
  load: async () => {
    const current = await getClaudeAuthCurrent()
    const source = asRecord(current)
    const loggedIn = source.logged_in === true || source.loggedIn === true
    return { loggedIn, canAuthOff: loggedIn, detail: typeof source.account_name === 'string' ? source.account_name : undefined }
  },
  authOff: async () => {
    await claudeAuthOff()
    return { changed: true }
  },
}

export const codexAuthSessionConfig: AuthSessionConfig = {
  cacheKey: 'auth-codex-session',
  homePath: '/codex/auth',
  module: 'codex',
  i18nPrefix: 'codex.auth',
  titleKey: 'codex.auth.title',
  subtitleKey: 'codex.auth.subtitle',
  confirmOffKey: 'auth.confirmOffCodex',
  features: {},
  notify: surfaceNotify,
  load: async () => {
    const current = await getCodexAuthCurrent()
    const source = asRecord(current)
    const loggedIn = source.logged_in === true || source.loggedIn === true
    return { loggedIn, canAuthOff: loggedIn }
  },
  authOff: async () => {
    await codexAuthOff()
    return { changed: true }
  },
}

export const authSessionConfigs = {
  grok: grokAuthConfig,
  claude: claudeAuthSessionConfig,
  codex: codexAuthSessionConfig,
} as const
