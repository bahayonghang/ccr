// Codex Profiles 派生洞察：薄包装，注入 Codex 平台差异（四 auth 模式、弃用模式、单模型缺失判定）
// 后委托平台无关核心 useProfilesInsights。公共 API 保持稳定，供 ProfilesInspector 复用。
import type { Ref } from 'vue'
import type { CodexProfile, CodexProfileAuthMode } from '@/types'
import { useProfilesInsights, type ProfilesInsights } from './useProfilesInsights'

export type {
  ProviderBreakdownItem,
  AuthModeBreakdownItem,
  TagFrequencyItem,
  MissingFieldIssue,
  DuplicateRuntimeIssue,
} from './useProfilesInsights'

export type MissingField = 'base_url' | 'model'

export type CodexProfilesInsights = ProfilesInsights<
  CodexProfile,
  CodexProfileAuthMode,
  MissingField
>

const ALL_AUTH_MODES: readonly CodexProfileAuthMode[] = [
  'openai_chatgpt',
  'openai_api_key',
  'provider_env_key',
  'no_auth',
]

const DEPRECATED_AUTH_MODES = new Set<CodexProfileAuthMode>([
  'openai_chatgpt',
  'provider_env_key',
])

const isBlank = (value: string | null | undefined): boolean => !value || value.trim().length === 0

const profileAuthMode = (profile: CodexProfile): CodexProfileAuthMode =>
  profile.auth_mode ?? 'no_auth'

/**
 * base_url 为空的 profile 是否仍然合规：
 * 仅 openai_chatgpt 模式允许空 base_url（官方 ChatGPT 登录运行时）。
 */
const requiresBaseUrl = (profile: CodexProfile): boolean =>
  profileAuthMode(profile) !== 'openai_chatgpt'

const missingFieldsOf = (profile: CodexProfile): MissingField[] => {
  const missing: MissingField[] = []
  if (requiresBaseUrl(profile) && isBlank(profile.base_url)) missing.push('base_url')
  if (isBlank(profile.model)) missing.push('model')
  return missing
}

const primaryRuntimeModel = (profile: CodexProfile): string => profile.model?.trim() ?? ''

export function useCodexProfilesInsights(profiles: Ref<CodexProfile[]>): CodexProfilesInsights {
  return useProfilesInsights<CodexProfile, CodexProfileAuthMode, MissingField>(profiles, {
    authModes: ALL_AUTH_MODES,
    deprecatedAuthModes: DEPRECATED_AUTH_MODES,
    authModeOf: profileAuthMode,
    missingFieldsOf,
    primaryRuntimeModel,
  })
}
