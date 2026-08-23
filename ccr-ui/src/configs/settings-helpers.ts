import type { SettingsField, SettingsFieldOption, SettingsScalar } from '@/configs/settings-types'

export const asString = (value: unknown): string => (value == null ? '' : String(value))
export const asBool = (value: unknown): boolean => value === true
export const asList = (value: unknown): string =>
  Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string').join('\n')
    : ''

export const splitList = (value: SettingsScalar): string[] =>
  typeof value === 'string' ? value.split('\n').map((item) => item.trim()).filter(Boolean) : []

export const boolField = (id: string, tab: string, labelKey: string): SettingsField => ({
  id,
  tab,
  kind: 'boolean',
  labelKey,
})

export const textField = (id: string, tab: string, labelKey: string): SettingsField => ({
  id,
  tab,
  kind: 'text',
  labelKey,
})

export const selectField = (spec: {
  id: string
  tab: string
  labelKey: string
  options: readonly SettingsFieldOption[]
}): SettingsField => ({
  id: spec.id,
  tab: spec.tab,
  kind: 'select',
  labelKey: spec.labelKey,
  options: spec.options,
})
