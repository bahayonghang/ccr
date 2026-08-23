// Codex Profiles 展示策略单源：base_url 回退、auth_mode 标签、行/检查器/diff 描述符。
// 与 utils/claudeProfiles.ts 对称，卡片 / 列表行 / 检查器 / 确认框全部引用这里的解析结果，
// 避免同一份 fallback 与查表逻辑散落到多个组件。
import type { CodexProfile } from '@/types'
import type { ProfileRowDescriptor } from '@/components/profiles/ProfileListRow.vue'
import type {
  ProfilesInspectorDescriptor,
  ProfilesInspectorField,
} from '@/components/profiles/ProfilesInspector.vue'
import type { ProfileDiffField } from '@/utils/profileDiff'
import { formatBaseUrlDisplay } from '@/utils/text'
import { buildCodexProfilesInsights } from '@/utils/codexProfilesInsights'

/** 字段缺失时的统一展示占位符（卡片/行/检查器共用） */
export const CODEX_FIELD_PLACEHOLDER = '—'

/** 视图注入的翻译函数形状，与 i18n/formatMessage 的 TranslateFn 保持一致 */
type CodexTranslate = (
  key: string,
  values?: Record<string, string | number | boolean | null | undefined>
) => string

/** base_url 展示值：空值回落到官方运行时文案 */
export const resolveCodexBaseUrl = (
  profile: Pick<CodexProfile, 'base_url'>,
  t: CodexTranslate
): string => profile.base_url?.trim() || t('codex.profiles.officialBaseUrl')

/** auth_mode 展示标签（缺省视为 no_auth） */
export const codexAuthModeLabel = (t: CodexTranslate, mode?: string | null): string =>
  t(`codex.profiles.authModes.${mode || 'no_auth'}`)

/** 缺失字段消息（Inspector 健康审计条目） */
const codexMissingMessage = (t: CodexTranslate, missing: string[]): string =>
  missing
    .map((field) =>
      field === 'base_url'
        ? t('codex.profiles.inspector.issues.missingBaseUrl')
        : t('codex.profiles.inspector.issues.missingModel')
    )
    .join(' · ')

/** 「当前 → 目标」diff 字段：base_url（回退解析后）/ model / auth_mode */
export const createCodexDiffFields = (t: CodexTranslate): ProfileDiffField<CodexProfile>[] => [
  {
    key: 'base_url',
    label: t('codex.profiles.fields.baseUrl'),
    value: (profile) => resolveCodexBaseUrl(profile, t),
  },
  {
    key: 'model',
    label: t('codex.profiles.fields.model'),
    // 空串交给 buildProfileDiff 规整为 null，占位符由渲染层决定
    value: (profile) => profile.model?.trim() ?? '',
  },
  {
    key: 'auth_mode',
    label: t('codex.profiles.fields.authMode'),
    value: (profile) => codexAuthModeLabel(t, profile.auth_mode),
  },
]

/** 列表行平台策略：字段解析 + 操作文案 + 编辑图标 */
export const createCodexRowDescriptor = (
  t: CodexTranslate
): ProfileRowDescriptor<CodexProfile> => ({
  // 列表密排：完整 host，仅截断路径
  baseUrl: (profile) => formatBaseUrlDisplay(resolveCodexBaseUrl(profile, t)),
  model: (profile) => profile.model?.trim() || CODEX_FIELD_PLACEHOLDER,
  authMode: (profile) => codexAuthModeLabel(t, profile.auth_mode),
  editIcon: 'Edit2',
  labels: {
    apply: t('codex.profiles.apply'),
    edit: t('codex.actions.edit'),
    delete: t('codex.actions.delete'),
  },
})

/** 检查器预览字段：完整展示，不截断；仅跳过未填写的可选项 */
const codexInspectorFields = (
  profile: CodexProfile,
  t: CodexTranslate
): ProfilesInspectorField[] => {
  const fields: ProfilesInspectorField[] = [
    { label: t('codex.profiles.fields.baseUrl'), value: resolveCodexBaseUrl(profile, t) },
    {
      label: t('codex.profiles.fields.model'),
      value: profile.model?.trim() || CODEX_FIELD_PLACEHOLDER,
      variant: 'accent',
    },
  ]

  const optionalFields: Array<{ label: string; value?: string | null; variant?: 'muted' }> = [
    {
      label: t('codex.profiles.fields.reasoningEffort'),
      value: profile.model_reasoning_effort,
      variant: 'muted',
    },
    { label: t('codex.profiles.fields.wireApi'), value: profile.wire_api, variant: 'muted' },
  ]

  for (const field of optionalFields) {
    const value = field.value?.trim()
    if (value) fields.push({ label: field.label, value, variant: field.variant })
  }

  fields.push({
    label: t('codex.profiles.fields.authMode'),
    value: codexAuthModeLabel(t, profile.auth_mode),
    variant: 'muted',
  })

  if (profile.auth_source?.trim()) {
    fields.push({
      label: t('codex.profiles.fields.authSource'),
      value: profile.auth_source.trim(),
      variant: 'muted',
    })
  }
  // env_key 只在 provider_env_key 模式下写入，有值即代表该模式的运行时约定
  if (profile.env_key?.trim()) {
    fields.push({ label: t('codex.profiles.fields.envKey'), value: profile.env_key.trim() })
  }
  if (profile.provider?.trim()) {
    fields.push({
      label: t('codex.profiles.fields.provider'),
      value: profile.provider.trim(),
      variant: 'muted',
    })
  }

  return fields
}

/** 检查器平台策略：洞察来源 + 预览字段 + diff 字段 + 文案/图标 */
export const createCodexInspectorDescriptor = (
  t: CodexTranslate
): ProfilesInspectorDescriptor<CodexProfile> => ({
  editIcon: 'Edit2',
  useInsights: buildCodexProfilesInsights,
  activeFields: (profile) => codexInspectorFields(profile, t),
  diffFields: createCodexDiffFields(t),
  authModeLabel: (mode) => codexAuthModeLabel(t, mode),
  isDeprecatedMode: (mode) => mode === 'openai_chatgpt' || mode === 'provider_env_key',
  missingMessage: (missing) => codexMissingMessage(t, missing),
  runtimeSummary: (profile) =>
    `${profile.model?.trim() || CODEX_FIELD_PLACEHOLDER} · ${profile.base_url?.trim() || CODEX_FIELD_PLACEHOLDER}`,
  deprecatedMessage: (profile) =>
    t('codex.profiles.inspector.issues.deprecatedAuth', {
      mode: codexAuthModeLabel(t, profile.auth_mode),
    }),
})
