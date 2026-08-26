// Profiles 共享层描述符类型。utils 组装策略，组件只消费这些形状。
// 放在 utils 是为了满足分层：utils 不得导入 components/ 或 features/。
import type { ProfileDiffField } from '@/utils/profileDiff'
import type { ProfilesInsightsResult } from '@/utils/profilesInsights'

/** Row 渲染所需的最小 profile 形状（两平台共有字段） */
export interface ProfileRowProfile {
  name: string
  description?: string | null
  enabled?: boolean | null
  tags?: readonly string[] | null
}

/** 平台注入的行渲染策略：解析展示字段 + 操作文案 + 编辑图标 */
export interface ProfileRowDescriptor<P> {
  /** 解析展示用 base_url（空回退官方文案） */
  baseUrl: (profile: P) => string
  /** 解析展示用主模型（Claude 多模型回退；Codex 单 model） */
  model: (profile: P) => string
  /** 解析展示用 auth 模式标签 */
  authMode: (profile: P) => string
  /** 编辑按钮图标：Claude 'Pencil' / Codex 'Edit2' */
  editIcon: string
  labels: { apply: string; edit: string; delete: string }
}

/** Inspector 直接读取的最小 profile 形状（两平台共有字段） */
export interface ProfilesInspectorProfile {
  name: string
  description?: string | null
  tags?: readonly string[] | null
}

/** 预览面板的单个字段（平台决定字段集合/顺序/样式） */
export interface ProfilesInspectorField {
  label: string
  value: string
  variant?: 'accent' | 'muted'
}

/** 平台注入的检查器策略：洞察来源 + 字段列表 + diff 字段 + 文案 + 图标 */
export interface ProfilesInspectorDescriptor<P extends ProfilesInspectorProfile> {
  /** 编辑按钮图标：Claude 'Pencil' / Codex 'Edit2' */
  editIcon: string
  /** 平台洞察（纯函数；入参为当前列表，不再接收 Vue Ref） */
  useInsights: (profiles: P[]) => ProfilesInsightsResult<P, string, string>
  /** 预览 profile 的字段列表（previewProfile 非空时调用） */
  activeFields: (profile: P) => ProfilesInspectorField[]
  /** 参与「当前 → 预览目标」diff 的字段（base_url/model/auth_mode 三行） */
  diffFields: readonly ProfileDiffField<P>[]
  /** auth 分布条标签 */
  authModeLabel: (mode: string) => string
  /** 该 auth 模式是否弃用（Claude 恒 false → 不加 warn 类） */
  isDeprecatedMode: (mode: string) => boolean
  /** 缺失字段消息（已 join） */
  missingMessage: (missing: string[]) => string
  /** 重复运行时条目摘要 */
  runtimeSummary: (profile: P) => string
  /** 已弃用 auth 条目消息（Codex 提供；Claude 无弃用概念，可省略） */
  deprecatedMessage?: (profile: P) => string
}
