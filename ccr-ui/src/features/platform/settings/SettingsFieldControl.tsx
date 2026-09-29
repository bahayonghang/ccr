import { memo } from 'react'
import { useWatch, type Control, type UseFormRegister } from 'react-hook-form'
import type { SettingsField, SettingsValues } from '@/configs/settings'
import type { TranslateFunction } from '@/utils/tf'

interface SettingsFieldControlProps {
  field: SettingsField
  register: UseFormRegister<SettingsValues>
  control: Control<SettingsValues>
  disabled?: boolean
  disabledReason?: string
  t: TranslateFunction
}

const optionalBoolean = (value: unknown) => value === '' || value == null ? null : value === true || value === 'true'
const controlClass = 'rounded-lg border border-border-default bg-bg-base px-3 py-2 disabled:opacity-50'
const booleanOptions = [{ value: 'true', labelKey: 'common.yes' }, { value: 'false', labelKey: 'common.no' }]

function SettingsSelect({ field, register, control, disabled, t }: SettingsFieldControlProps) {
  const value = useWatch({ control, name: field.id })
  const nullableBoolean = field.kind === 'boolean'
  const current = value == null ? '' : String(value)
  const options = nullableBoolean ? booleanOptions : field.options ?? []
  const unknown = !options.some((option) => option.value === current)
  return (
    <select className={controlClass} {...register(field.id, { setValueAs: nullableBoolean ? optionalBoolean : undefined })} disabled={disabled} aria-describedby={field.id + '-help'} value={current}>
      {field.unsetLabelKey || (unknown && current === '') ? (
        <option value="">{field.unsetLabelKey ? t(field.unsetLabelKey) : '—'}</option>
      ) : null}
      {unknown && current !== '' ? <option value={current}>{current}</option> : null}
      {options.map((option) => <option key={option.value} value={option.value}>{t(option.labelKey)}</option>)}
    </select>
  )
}

function SettingsInput(props: SettingsFieldControlProps) {
  const { field, register, disabled, control } = props
  const value = useWatch({ control, name: field.id })
  if (field.kind === 'select' || (field.kind === 'boolean' && field.unsetLabelKey)) return <SettingsSelect {...props} />
  if (field.kind === 'textarea') {
    return <textarea className={controlClass + ' min-h-24 font-mono text-xs'} {...register(field.id)} defaultValue={String(value ?? '')} disabled={disabled} aria-describedby={field.id + '-help'} />
  }
  return (
    <input
      type={field.kind === 'boolean' ? 'checkbox' : field.kind === 'number' ? 'number' : 'text'}
      className={field.kind === 'boolean' ? 'justify-self-start' : controlClass}
      min={field.integerRange?.min}
      max={field.integerRange?.max}
      step={field.integerRange ? 1 : undefined}
      {...register(field.id)}
      defaultValue={field.kind === 'boolean' ? undefined : String(value ?? '')}
      defaultChecked={field.kind === 'boolean' ? value === true : undefined}
      disabled={disabled}
      aria-describedby={field.id + '-help'}
    />
  )
}

export const SettingsFieldControl = memo(function SettingsFieldControl(props: SettingsFieldControlProps) {
  const { field, t, disabledReason } = props
  const description = disabledReason ?? (field.helperKey ? t(field.helperKey) : undefined)
  return (
    <div className="grid gap-1">
      <label className="grid gap-1 text-sm text-text-primary">
        <span>{t(field.labelKey)}</span>
        <SettingsInput {...props} />
      </label>
      <span id={field.id + '-help'} className="text-xs text-text-muted">{description}</span>
    </div>
  )
})
