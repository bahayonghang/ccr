import type { KeyboardEvent, ReactElement } from 'react'
import type { ProfileEditorFieldKind, ProfileEditorFieldSpec } from '@/configs/profileEditorAdapter'
import { useShellT } from '@/shell/i18n'
import { Switch } from '@/ui'

export interface ProfileEditorFieldsProps {
  field: ProfileEditorFieldSpec
  form: unknown
  onChange: (key: string, value: unknown) => void
}

const asRecord = (form: unknown): Record<string, unknown> => {
  if (form && typeof form === 'object' && !Array.isArray(form)) {
    return form as Record<string, unknown>
  }
  return {}
}

const asString = (value: unknown): string => (typeof value === 'string' ? value : '')

const asBoolean = (value: unknown): boolean => value === true

const tagsOf = (value: unknown): string[] => {
  if (Array.isArray(value)) {
    return value.filter((item): item is string => typeof item === 'string' && item.length > 0)
  }
  return asString(value)
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)
}

const writeTags = (tags: string[]): string => tags.join(', ')

function FieldLabel({ field, required }: { field: ProfileEditorFieldSpec; required: boolean }) {
  const t = useShellT()
  return (
    <span className="cp-label">
      {t(field.labelKey)}
      {required ? <span className="pe-required"> *</span> : null}
    </span>
  )
}

function TextField({
  field,
  value,
  required,
  mono,
  secret,
  number,
  onChange,
}: {
  field: ProfileEditorFieldSpec
  value: string
  required: boolean
  mono?: boolean
  secret?: boolean
  number?: boolean
  onChange: (value: string) => void
}) {
  const t = useShellT()
  return (
    <label className="pe-field" data-field={field.key} data-kind={field.kind}>
      <FieldLabel field={field} required={required} />
      <input
        type={secret ? 'password' : 'text'}
        className={mono ? 'cp-input pe-input--mono' : 'cp-input'}
        value={value}
        readOnly={field.readOnly}
        inputMode={number ? 'numeric' : undefined}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
      {field.hintKey ? <span className="pe-hint">{t(field.hintKey)}</span> : null}
      {secret ? <span className="pe-hint">{t('profileEditor.secretLeaveEmpty')}</span> : null}
    </label>
  )
}

function ChoiceField({
  field,
  value,
  required,
  onChange,
}: {
  field: ProfileEditorFieldSpec
  value: string
  required: boolean
  onChange: (value: string) => void
}) {
  const t = useShellT()
  const options = field.options ?? []
  return (
    <div className="pe-field" data-field={field.key} data-kind={field.kind}>
      <FieldLabel field={field} required={required} />
      <div className="cp-pill-row">
        {options.map((option) => (
          <button
            key={option}
            type="button"
            className={value === option ? 'cp-pill cp-pill--active' : 'cp-pill'}
            aria-pressed={value === option}
            disabled={field.readOnly}
            onClick={() => onChange(option)}
          >
            {option}
          </button>
        ))}
      </div>
      <input
        type="text"
        className="cp-input"
        value={value}
        readOnly={field.readOnly}
        placeholder={t('profileEditor.choiceFilter')}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
    </div>
  )
}

function MultiValueField({
  field,
  value,
  required,
  onChange,
}: {
  field: ProfileEditorFieldSpec
  value: unknown
  required: boolean
  onChange: (value: string) => void
}) {
  const t = useShellT()
  const tags = tagsOf(value)
  const options = field.options ?? []
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (field.readOnly || event.key !== 'Enter') return
    event.preventDefault()
    const next = event.currentTarget.value.trim()
    if (!next || tags.includes(next)) return
    onChange(writeTags([...tags, next]))
    event.currentTarget.value = ''
  }
  return (
    <div className="pe-field" data-field={field.key} data-kind={field.kind}>
      <FieldLabel field={field} required={required} />
      <div className="cp-pill-row">
        {options.map((option) => {
          const selected = tags.includes(option)
          return (
            <button
              key={option}
              type="button"
              className={selected ? 'cp-pill cp-pill--active' : 'cp-pill'}
              aria-pressed={selected}
              disabled={field.readOnly}
              onClick={() => {
                const next = selected ? tags.filter((tag) => tag !== option) : [...tags, option]
                onChange(writeTags(next))
              }}
            >
              {option}
            </button>
          )
        })}
      </div>
      <input
        type="text"
        className="cp-input"
        placeholder={t('profileEditor.addValue')}
        readOnly={field.readOnly}
        onKeyDown={onKeyDown}
      />
    </div>
  )
}

function BooleanField({
  field,
  value,
  required,
  onChange,
}: {
  field: ProfileEditorFieldSpec
  value: boolean
  required: boolean
  onChange: (value: boolean) => void
}) {
  return (
    <label className="pe-field pe-field--switch" data-field={field.key} data-kind={field.kind}>
      <FieldLabel field={field} required={required} />
      <Switch
        checked={value}
        disabled={field.readOnly}
        onCheckedChange={(checked) => onChange(checked === true)}
      />
    </label>
  )
}

const renderKind = ({
  kind,
  field,
  raw,
  required,
  onChange,
}: {
  kind: ProfileEditorFieldKind
  field: ProfileEditorFieldSpec
  raw: unknown
  required: boolean
  onChange: (value: unknown) => void
}): ReactElement => {
  switch (kind) {
    case 'text':
      return <TextField field={field} value={asString(raw)} required={required} onChange={onChange} />
    case 'mono-text':
      return (
        <TextField field={field} value={asString(raw)} required={required} mono onChange={onChange} />
      )
    case 'secret':
      return (
        <TextField
          field={field}
          value={asString(raw)}
          required={required}
          secret
          onChange={onChange}
        />
      )
    case 'number':
      return (
        <TextField
          field={field}
          value={asString(raw)}
          required={required}
          number
          onChange={onChange}
        />
      )
    case 'choice':
      return (
        <ChoiceField field={field} value={asString(raw)} required={required} onChange={onChange} />
      )
    case 'multi-value':
      return (
        <MultiValueField field={field} value={raw} required={required} onChange={onChange} />
      )
    case 'boolean':
      return (
        <BooleanField field={field} value={asBoolean(raw)} required={required} onChange={onChange} />
      )
    default: {
      const _never: never = kind
      return _never
    }
  }
}

/** 七种字段 kind 的渲染分派；组件内不比较平台名。 */
export function ProfileEditorFields({ field, form, onChange }: ProfileEditorFieldsProps) {
  const values = asRecord(form)
  if (field.visible && !field.visible(form)) return null
  const required = field.required?.(form) === true
  return (
    <div data-required={required ? 'true' : undefined}>
      {renderKind({
        kind: field.kind,
        field,
        raw: values[field.key],
        required,
        onChange: (value) => onChange(field.key, value),
      })}
    </div>
  )
}
