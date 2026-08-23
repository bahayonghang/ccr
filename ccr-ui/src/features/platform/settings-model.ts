import type { SettingsConfig, SettingsField, SettingsValues } from '@/configs/settings'

/** 可见字段的唯一实现：所有平台 settings config 都经此过滤。 */
export function visibleSettingsFields(config: SettingsConfig): SettingsField[] {
  return config.fields.filter((field) => {
    if (!field.requires) return true
    return config.features[field.requires] === true
  })
}

export function settingsDefaultValues(config: SettingsConfig): SettingsValues {
  const values: SettingsValues = {}
  for (const field of visibleSettingsFields(config)) {
    values[field.id] = field.kind === 'boolean' ? false : ''
  }
  return values
}

export function invalidSettingsField(
  config: SettingsConfig,
  values: SettingsValues,
  dirtyKeys: readonly string[],
): string | null {
  for (const field of visibleSettingsFields(config)) {
    if (!field.integerRange) continue
    if (!dirtyKeys.includes(field.id)) continue
    const value = values[field.id]
    if (value === '' || value == null) continue
    const parsed = Number(value)
    if (!Number.isInteger(parsed)) return field.id
    if (parsed < field.integerRange.min) return field.id
    if (parsed > field.integerRange.max) return field.id
  }
  return null
}

/** 保存路径的唯一实现：校验 + config.save。改此处则全部 settings 薄壳同时生效。 */
export async function saveSettingsValues(
  config: SettingsConfig,
  values: SettingsValues,
  dirtyKeys: string[],
): Promise<void> {
  const invalid = invalidSettingsField(config, values, dirtyKeys)
  if (invalid) throw new Error(invalid)
  await config.save({ values, dirtyKeys })
}

export function fieldsForTab(config: SettingsConfig, tab: string): SettingsField[] {
  return visibleSettingsFields(config).filter((field) => field.tab === tab)
}
