// Claude Profiles 派生洞察（纯函数薄包装）：注入 Claude 平台差异（双 auth 模式、
// 多模型缺失判定）后委托 utils/profilesInsights 核心。Ref 入参改为数组。
import type { ClaudeProfile, ClaudeProfileAuthMode } from '@/types'
import { resolveClaudePrimaryModel } from '@/utils/claudeProfileFields'
import {
  buildProfilesInsights,
  type ProfilesInsightsResult,
} from '@/utils/profilesInsights'

export type {
  ProviderBreakdownItem,
  AuthModeBreakdownItem,
  TagFrequencyItem,
  MissingFieldIssue,
  DuplicateRuntimeIssue,
} from '@/utils/profilesInsights'

export type ClaudeMissingField = 'base_url' | 'model' | 'account'

export type ClaudeProfilesInsightsResult = ProfilesInsightsResult<
  ClaudeProfile,
  ClaudeProfileAuthMode,
  ClaudeMissingField
>

// Claude 仅两种 auth 模式，无废弃项；固定顺序便于稳定布局
const ALL_AUTH_MODES: readonly ClaudeProfileAuthMode[] = ['subscription', 'api_key']

const isBlank = (value: string | null | undefined): boolean => !value || value.trim().length === 0

const profileAuthMode = (profile: ClaudeProfile): ClaudeProfileAuthMode =>
  profile.auth_mode ?? 'subscription'

/** 是否配置了任意模型。Claude 多模型：主模型与四个映射全空才算缺失。 */
const hasAnyModel = (profile: ClaudeProfile): boolean =>
  Boolean(resolveClaudePrimaryModel(profile, ''))

/**
 * base_url 为空是否需要报告：仅 api_key 模式必须有 base_url；
 * subscription 模式空 base_url 合法（回落本机官方登录）。
 */
const requiresBaseUrl = (profile: ClaudeProfile): boolean => profileAuthMode(profile) === 'api_key'

const missingFieldsOf = (profile: ClaudeProfile): ClaudeMissingField[] => {
  const missing: ClaudeMissingField[] = []
  if (requiresBaseUrl(profile) && isBlank(profile.base_url)) missing.push('base_url')
  if (!hasAnyModel(profile)) missing.push('model')
  if (isBlank(profile.account)) missing.push('account')
  return missing
}

export function buildClaudeProfilesInsights(
  profiles: ClaudeProfile[],
): ClaudeProfilesInsightsResult {
  return buildProfilesInsights<ClaudeProfile, ClaudeProfileAuthMode, ClaudeMissingField>(
    profiles,
    {
      authModes: ALL_AUTH_MODES,
      authModeOf: profileAuthMode,
      missingFieldsOf,
      primaryRuntimeModel: (profile) => resolveClaudePrimaryModel(profile, ''),
    },
  )
}
