export type ProfileEditorFieldKind =
  | 'text'
  | 'mono-text'
  | 'choice'
  | 'secret'
  | 'multi-value'
  | 'boolean'
  | 'number'

export interface ProfileEditorFieldSpec {
  key: string
  labelKey: string
  kind: ProfileEditorFieldKind
  options?: readonly string[]
  hintKey?: string
  /** 由平台按当前表单值决定是否渲染 / 是否必填 */
  visible?: (form: unknown) => boolean
  required?: (form: unknown) => boolean
  readOnly?: boolean
}

export interface ProfileEditorSection {
  id: string
  titleKey?: string
  /** grid 为两列区，row 为整行区，group 为带边框的分组（认证区） */
  layout: 'grid' | 'row' | 'group'
  advanced?: boolean
  fields: readonly ProfileEditorFieldSpec[]
}

export interface ProfileEditorIssue {
  /** 出错分段 id，用于汇总条跳转 */
  section: string
  /** 具体字段；无法定位到单字段时省略 */
  field?: string
  /** 已翻译的错误文案 */
  message: string
}

export type ProfileWriteOutcome =
  | { status: 'ok'; appliedName?: string }
  | { status: 'recovery'; kind: string; message: string }
  | { status: 'blocked'; message: string; forceAllowed: boolean }
  | { status: 'error'; message: string }

export interface ProfileEditorAdapter<TForm = unknown, TRecord = unknown> {
  createEmpty(): TForm
  /** 入参已剥离凭据；返回的表单密钥字段一律为空 */
  fromRecord(record: TRecord): TForm
  sections: readonly ProfileEditorSection[]
  /** 返回校验问题列表，空数组表示通过 */
  validate(
    form: TForm,
    ctx: {
      isEditing: boolean
      originalName: string | null
      existingNames: readonly string[]
      /** 编辑既有 profile 时后端已存有 base URL，用于 Grok 的留空放行分支 */
      hasExistingBaseUrl: boolean
    },
  ): readonly ProfileEditorIssue[]
  /** 平台内部自行组装 create / patch，含 dirty 字段与 credential action */
  submit(
    form: TForm,
    ctx: {
      isEditing: boolean
      originalName: string | null
      apply: boolean
      dirtyFields: ReadonlySet<string>
    },
  ): Promise<ProfileWriteOutcome>
}
