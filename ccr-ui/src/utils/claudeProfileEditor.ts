// Claude Profile 编辑器的表单装配：初始值 / 回填 / 请求体构建 / 模板补丁。
// 与 Codex 的 utils/codexProfileEditor.ts 对称，视图只保留流程编排（打开/保存/确认）。
import type { ClaudeProfile, ClaudeProfileRequest } from '@/types'
import type { ClaudeProfileEditorForm } from '@/types/claudeProfileEditor'
import type {
  ProviderTemplateDraftContext,
  ProviderTemplateSelection,
} from '@/types/providerTemplates'
import { mapTemplateToClaudeProfilePatch } from '@/utils/providerTemplates'

/** 后端接受的思考强度取值；其余值一律回落到「使用模型默认」 */
const VALID_EFFORT_LEVELS: readonly string[] = ['low', 'medium', 'high', 'xhigh', 'max']

const normalizeOptional = (value: string): string | undefined => {
  const trimmed = value.trim()
  return trimmed ? trimmed : undefined
}

/** 逗号分隔标签串 → 去空标签数组；全空返回 undefined（不写入该字段） */
export const parseClaudeProfileTags = (input: string): string[] | undefined => {
  const tags = input
    .split(',')
    .map((tag) => tag.trim())
    .filter(Boolean)

  return tags.length > 0 ? tags : undefined
}

/** 空白表单初值 */
export const createClaudeProfileForm = (): ClaudeProfileEditorForm => ({
  name: '',
  description: '',
  auth_mode: 'subscription',
  base_url: '',
  auth_token: '',
  default_opus_model: '',
  default_sonnet_model: '',
  default_haiku_model: '',
  default_fable_model: '',
  default_opus_model_name: '',
  default_sonnet_model_name: '',
  default_haiku_model_name: '',
  default_fable_model_name: '',
  subagent_model: '',
  custom_model_option: '',
  custom_model_option_name: '',
  effort_level: '',
  claude_code_auto_compact_window: '',
  api_timeout_ms: '',
  claude_code_disable_nonessential_traffic: '',
  provider: '',
  provider_type: '',
  account: '',
  tagsInput: '',
  enabled: true,
})

/** 就地重置为空白表单（保持 reactive 对象引用不变） */
export const resetClaudeProfileForm = (form: ClaudeProfileEditorForm): void => {
  Object.assign(form, createClaudeProfileForm())
}

/** 用既有 profile 回填表单（编辑场景） */
export const fillClaudeProfileForm = (
  form: ClaudeProfileEditorForm,
  profile: ClaudeProfile
): void => {
  const rawEffort = profile.effort_level || ''

  Object.assign(form, {
    name: profile.name,
    description: profile.description || '',
    auth_mode: profile.auth_mode || 'subscription',
    base_url: profile.base_url || '',
    auth_token: profile.auth_token || '',
    default_opus_model: profile.default_opus_model || '',
    default_sonnet_model: profile.default_sonnet_model || '',
    default_haiku_model: profile.default_haiku_model || '',
    default_fable_model: profile.default_fable_model || '',
    default_opus_model_name: profile.default_opus_model_name || '',
    default_sonnet_model_name: profile.default_sonnet_model_name || '',
    default_haiku_model_name: profile.default_haiku_model_name || '',
    default_fable_model_name: profile.default_fable_model_name || '',
    subagent_model: profile.subagent_model || '',
    custom_model_option: profile.custom_model_option || '',
    custom_model_option_name: profile.custom_model_option_name || '',
    effort_level: VALID_EFFORT_LEVELS.includes(rawEffort) ? rawEffort : '',
    claude_code_auto_compact_window: profile.claude_code_auto_compact_window || '',
    api_timeout_ms: profile.api_timeout_ms || '',
    claude_code_disable_nonessential_traffic:
      profile.claude_code_disable_nonessential_traffic || '',
    provider: profile.provider || '',
    provider_type: profile.provider_type || '',
    account: profile.account || '',
    tagsInput: (profile.tags || []).join(', '),
    enabled: profile.enabled !== false,
  } satisfies ClaudeProfileEditorForm)
}

/** 表单 → 写入请求体（model / small_fast_model 由多模型映射取代，恒为 null） */
export const buildClaudeProfileRequest = (form: ClaudeProfileEditorForm): ClaudeProfileRequest => ({
  name: form.name.trim(),
  description: normalizeOptional(form.description),
  auth_mode: form.auth_mode,
  base_url: normalizeOptional(form.base_url),
  auth_token: normalizeOptional(form.auth_token),
  model: null,
  small_fast_model: null,
  default_opus_model: normalizeOptional(form.default_opus_model) ?? null,
  default_sonnet_model: normalizeOptional(form.default_sonnet_model) ?? null,
  default_haiku_model: normalizeOptional(form.default_haiku_model) ?? null,
  default_fable_model: normalizeOptional(form.default_fable_model) ?? null,
  default_opus_model_name: normalizeOptional(form.default_opus_model_name) ?? null,
  default_sonnet_model_name: normalizeOptional(form.default_sonnet_model_name) ?? null,
  default_haiku_model_name: normalizeOptional(form.default_haiku_model_name) ?? null,
  default_fable_model_name: normalizeOptional(form.default_fable_model_name) ?? null,
  subagent_model: normalizeOptional(form.subagent_model) ?? null,
  custom_model_option: normalizeOptional(form.custom_model_option) ?? null,
  custom_model_option_name: normalizeOptional(form.custom_model_option_name) ?? null,
  effort_level: normalizeOptional(form.effort_level) ?? null,
  claude_code_auto_compact_window: normalizeOptional(form.claude_code_auto_compact_window) ?? null,
  api_timeout_ms: normalizeOptional(form.api_timeout_ms) ?? null,
  claude_code_disable_nonessential_traffic:
    normalizeOptional(form.claude_code_disable_nonessential_traffic) ?? null,
  provider: normalizeOptional(form.provider),
  provider_type: normalizeOptional(form.provider_type),
  account: normalizeOptional(form.account),
  tags: parseClaudeProfileTags(form.tagsInput),
  enabled: form.enabled,
})

/** 当前表单快照 → Provider 模板选择器的草稿上下文 */
export const buildClaudeTemplateDraft = (
  form: ClaudeProfileEditorForm
): ProviderTemplateDraftContext => ({
  platform: 'claude',
  defaultName: form.provider || form.name || 'Claude provider',
  name: form.provider || form.name,
  category: 'third_party',
  baseUrls: form.base_url.trim() ? [form.base_url.trim()] : [],
  modelCatalog: [
    form.default_opus_model,
    form.default_sonnet_model,
    form.default_haiku_model,
    form.default_fable_model,
    form.subagent_model,
  ].filter(Boolean),
  platformOverride: {
    baseUrl: form.base_url,
    provider: form.provider,
    providerType: form.provider_type,
    defaultOpusModel: form.default_opus_model,
    defaultSonnetModel: form.default_sonnet_model,
    defaultHaikuModel: form.default_haiku_model,
    defaultFableModel: form.default_fable_model,
    subagentModel: form.subagent_model,
    claudeCodeAutoCompactWindow: form.claude_code_auto_compact_window,
    apiTimeoutMs: form.api_timeout_ms,
    claudeCodeDisableNonessentialTraffic: form.claude_code_disable_nonessential_traffic,
    description: form.description,
  },
})

/**
 * 把 Provider 模板补丁写进表单。
 * 返回是否触发了 auth_mode 自动改写（模板带 base_url 即视为第三方/中转端点，
 * 必须切到 api_key，否则 apply 时后端会清空 ANTHROPIC_*），供视图给出可见提示。
 */
export const applyClaudeTemplateToForm = (
  form: ClaudeProfileEditorForm,
  selection: ProviderTemplateSelection
): boolean => {
  const patch = mapTemplateToClaudeProfilePatch(selection.template, selection.endpoint)

  form.base_url = patch.base_url || ''
  form.provider = patch.provider || selection.template.name
  form.provider_type = patch.provider_type || ''
  form.default_opus_model = patch.default_opus_model || ''
  form.default_sonnet_model = patch.default_sonnet_model || ''
  form.default_haiku_model = patch.default_haiku_model || ''
  form.default_fable_model = patch.default_fable_model || ''
  form.subagent_model = patch.subagent_model || ''
  form.claude_code_auto_compact_window = patch.claude_code_auto_compact_window || ''
  form.api_timeout_ms = patch.api_timeout_ms || ''
  form.claude_code_disable_nonessential_traffic =
    patch.claude_code_disable_nonessential_traffic || ''

  if (!form.name.trim()) {
    form.name = patch.suggestedName || selection.template.id
  }
  if (!form.description.trim() && patch.description) {
    form.description = patch.description
  }

  const switchedAuthMode = Boolean(patch.base_url) && form.auth_mode !== 'api_key'
  if (patch.base_url) form.auth_mode = 'api_key'

  return switchedAuthMode
}
