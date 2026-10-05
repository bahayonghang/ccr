import { tt } from '@/features/claude/locale'
import type {
  ClaudeAuthSourceObservation,
  ClaudeLoginState,
  ClaudeRuntimeSummary,
} from '@/types'

const KIND_LABELS: Record<ClaudeAuthSourceObservation['kind'], [string, string]> = {
  bedrock: ['Amazon Bedrock', 'Amazon Bedrock'],
  vertex: ['Google Vertex AI', 'Google Vertex AI'],
  foundry: ['Microsoft Foundry', 'Microsoft Foundry'],
  anthropic_auth_token: ['ANTHROPIC_AUTH_TOKEN', 'ANTHROPIC_AUTH_TOKEN'],
  anthropic_api_key: ['ANTHROPIC_API_KEY', 'ANTHROPIC_API_KEY'],
  api_key_helper: ['apiKeyHelper', 'apiKeyHelper'],
  claude_code_oauth_token: ['CLAUDE_CODE_OAUTH_TOKEN', 'CLAUDE_CODE_OAUTH_TOKEN'],
  subscription_oauth: ['官方订阅 OAuth', 'Official subscription OAuth'],
  primary_api_key: ['primaryApiKey', 'primaryApiKey'],
}

const LOCATION_LABELS: Record<ClaudeAuthSourceObservation['location'], [string, string]> = {
  process_env: ['当前进程环境', 'Current process environment'],
  settings_env: ['settings.json env', 'settings.json env'],
  settings_root: ['settings.json 顶层', 'settings.json root'],
  state_file: ['Claude state file', 'Claude state file'],
  credentials_file: ['credentials file', 'credentials file'],
}

const CONFIDENCE_LABELS: Record<ClaudeAuthSourceObservation['confidence'], [string, string]> = {
  confirmed: ['已确认', 'Confirmed'],
  potential: ['潜在', 'Potential'],
  unobservable: ['不可观测', 'Unobservable'],
}

const OWNERSHIP_LABELS: Record<ClaudeAuthSourceObservation['ownership'], [string, string]> = {
  ccr_managed: ['CCR 托管', 'CCR-managed'],
  user_owned: ['用户自有', 'User-owned'],
  external_runtime: ['外部运行时', 'External runtime'],
}

const UNOBSERVABLE_LABELS: Record<string, [string, string]> = {
  other_shell_environment: ['其他 shell 的环境变量', 'Environment variables in other shells'],
  project_settings_for_unknown_working_directories: [
    '未知工作目录下的项目级 settings',
    'Project settings under unknown working directories',
  ],
  external_process_cli_arguments: ['外部 Claude Code 进程的 CLI 参数', 'CLI arguments of external Claude Code processes'],
  managed_settings_dynamic_policy: ['组织级 managed settings 动态策略', 'Dynamic organization-managed settings policy'],
  api_key_helper_result_and_external_secret_store: [
    'apiKeyHelper 返回值与外部 secret store',
    'apiKeyHelper output and external secret stores',
  ],
  macos_keychain_contents: ['macOS Keychain 内容', 'macOS Keychain contents'],
}

const RUNTIME_MODE_LABELS: Record<string, [string, string]> = {
  profile_with_auth: ['Profile + 官方订阅', 'Profile + official subscription'],
  profile_pending_auth: ['Profile 等待官方订阅', 'Profile waiting for official subscription'],
  profile_only: ['Profile 驱动（API key）', 'Profile-driven (API key)'],
  runtime_only: ['仅官方订阅运行时', 'Official subscription runtime only'],
  unresolved: ['未解析', 'Unresolved'],
}

export function authSourceKindLabel(kind: ClaudeAuthSourceObservation['kind']): string {
  return tt(...KIND_LABELS[kind])
}

export function authSourceLocationLabel(location: ClaudeAuthSourceObservation['location']): string {
  return tt(...LOCATION_LABELS[location])
}

export function authConfidenceLabel(confidence: ClaudeAuthSourceObservation['confidence']): string {
  return tt(...CONFIDENCE_LABELS[confidence])
}

export function authEvidenceLabel(evidence: ClaudeAuthSourceObservation['evidence']): string {
  return evidence === 'issue_report' ? tt('Issue 报告行为', 'Issue-reported behavior') : tt('官方契约', 'Official contract')
}

export function authOwnershipLabel(ownership: ClaudeAuthSourceObservation['ownership']): string {
  return tt(...OWNERSHIP_LABELS[ownership])
}

export function formatAuthSource(source: ClaudeAuthSourceObservation): string {
  return `${authSourceKindLabel(source.kind)} · ${authSourceLocationLabel(source.location)}`
}

export function loginStateLabel(state: ClaudeLoginState | null | undefined): string {
  if (!state) return tt('未登录', 'Not logged in')
  if (state.type === 'LoggedInSaved') {
    return tt(`已登录（已保存为 ${state.account_name}）`, `Logged in (saved as ${state.account_name})`)
  }
  if (state.type === 'LoggedInUnsaved') return tt('已登录（未保存）', 'Logged in (unsaved)')
  if (state.type === 'ApiKeyActive') return tt('当前由 API key profile 控制', 'Currently controlled by the API key profile')
  return tt('未登录', 'Not logged in')
}

export function runtimeModeLabel(summary: ClaudeRuntimeSummary | null): string {
  const pair = RUNTIME_MODE_LABELS[summary?.mode ?? 'unresolved'] ?? RUNTIME_MODE_LABELS.unresolved
  return tt(...pair)
}

export function currentProfileLabel(summary: ClaudeRuntimeSummary | null): string {
  if (!summary?.current_profile_name) return tt('未绑定', 'Unbound')
  return summary.current_profile_auth_mode
    ? `${summary.current_profile_name} · ${summary.current_profile_auth_mode}`
    : summary.current_profile_name
}

export function unobservableLabels(items: string[]): string[] {
  return items.map((item) => (UNOBSERVABLE_LABELS[item] ? tt(...UNOBSERVABLE_LABELS[item]) : item))
}

export function formatAuthDate(date: string): string {
  try {
    return new Date(date).toLocaleString(tt('zh-CN', 'en-US'))
  } catch {
    return date
  }
}

export function extractAuthError(error: unknown): string {
  if (error instanceof Error && error.message) return error.message
  if (typeof error === 'string') return error
  return tt('请求失败', 'Request failed')
}
