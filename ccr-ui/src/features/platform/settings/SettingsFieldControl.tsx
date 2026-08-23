import { memo } from 'react'
import type { UseFormRegister } from 'react-hook-form'
import type { SettingsField, SettingsValues } from '@/configs/settings'
import type { TranslateFunction } from '@/utils/tf'

interface SettingsFieldControlProps {
  field: SettingsField
  register: UseFormRegister<SettingsValues>
  t: TranslateFunction
}

export const SettingsFieldControl = memo(function SettingsFieldControl({
  field,
  register,
  t,
}: SettingsFieldControlProps) {
  const label = t(field.labelKey)
  const helper = field.helperKey ? t(field.helperKey) : undefined

  if (field.kind === 'boolean') {
    return (
      <label className="flex items-center gap-3 text-sm text-text-primary">
        <input type="checkbox" {...register(field.id)} />
        <span>{label}</span>
      </label>
    )
  }

  if (field.kind === 'select') {
    return (
      <label className="grid gap-1 text-sm text-text-primary">
        <span>{label}</span>
        <select
          className="rounded-xl border border-border-default bg-bg-base px-3 py-2"
          {...register(field.id)}
        >
          {(field.options ?? []).map((option) => (
            <option key={option.value} value={option.value}>
              {t(option.labelKey)}
            </option>
          ))}
        </select>
        {helper ? <span className="text-xs text-text-muted">{helper}</span> : null}
      </label>
    )
  }

  if (field.kind === 'textarea') {
    return (
      <label className="grid gap-1 text-sm text-text-primary">
        <span>{label}</span>
        <textarea
          className="min-h-24 rounded-xl border border-border-default bg-bg-base px-3 py-2 font-mono text-xs"
          {...register(field.id)}
        />
        {helper ? <span className="text-xs text-text-muted">{helper}</span> : null}
      </label>
    )
  }

  const inputType = field.kind === 'number' ? 'number' : 'text'
  return (
    <label className="grid gap-1 text-sm text-text-primary">
      <span>{label}</span>
      <input
        type={inputType}
        className="rounded-xl border border-border-default bg-bg-base px-3 py-2"
        {...register(field.id)}
      />
      {helper ? <span className="text-xs text-text-muted">{helper}</span> : null}
    </label>
  )
})
