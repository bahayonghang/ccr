import type { EnvironmentProbe } from '@/configs/probeLocal'
import type { SurfaceNotify } from '@/configs/surfaceNotify'

export type SettingsFieldKind = 'text' | 'textarea' | 'number' | 'boolean' | 'select'
export type SettingsFeatureName =
  | 'rawSource'
  | 'localOnly'
  | 'dirtyPatch'
  | 'managedLocks'
  | 'dualFile'
export type SettingsScalar = string | number | boolean | null
export type SettingsValues = Record<string, SettingsScalar>

export interface SettingsFieldOption {
  value: string
  labelKey: string
}

export interface SettingsField {
  id: string
  tab: string
  kind: SettingsFieldKind
  labelKey: string
  helperKey?: string
  options?: readonly SettingsFieldOption[]
  requires?: SettingsFeatureName
  listValue?: boolean
  integerRange?: { min: number; max: number }
}

export interface SettingsTab {
  id: string
  labelKey: string
}

export interface SettingsFeatures {
  rawSource?: boolean
  localOnly?: boolean
  dirtyPatch?: boolean
  managedLocks?: boolean
  dualFile?: boolean
}

export interface SettingsConfig {
  cacheKey: string
  homePath: string
  module: string
  i18nPrefix: string
  titleKey: string
  subtitleKey: string
  tabs: readonly SettingsTab[]
  fields: readonly SettingsField[]
  features: SettingsFeatures
  notify: SurfaceNotify
  probe?: () => Promise<EnvironmentProbe>
  load: () => Promise<SettingsValues>
  save: (payload: { values: SettingsValues; dirtyKeys: string[] }) => Promise<void>
}
