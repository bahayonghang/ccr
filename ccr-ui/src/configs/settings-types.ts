import type { EnvironmentProbe } from '@/configs/probeLocal'
import type { SurfaceNotify } from '@/configs/surfaceNotify'
import type { ConfigLayersResult, RawFileGetResult, RawFileSaveResult } from '@/api/domains/configRawTypes'

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
  unsetLabelKey?: string
}

export interface SettingsSnapshot {
  values: SettingsValues
  /** Original typed response. Keep in the current editor session only. */
  source: unknown
  managedLocks?: Readonly<Record<string, string>>
}

export interface SettingsSaveInput {
  values: SettingsValues
  dirtyKeys: string[]
  snapshot: SettingsSnapshot
  environmentId?: string
}

export type SettingsSaveResult =
  | { status: 'saved' }
  | { status: 'conflict' }
  | { status: 'managed_locked'; message: string }
  | { status: 'unsupported_environment' }

export class SettingsUnavailableError extends Error {
  constructor() {
    super('unsupported_environment')
    this.name = 'SettingsUnavailableError'
  }
}

export class SettingsValidationError extends Error {
  constructor(readonly messageKey: string) {
    super(messageKey)
    this.name = 'SettingsValidationError'
  }
}

export interface SettingsRawSource {
  language: 'json' | 'toml'
  probe: () => Promise<EnvironmentProbe>
  getRaw: () => Promise<RawFileGetResult>
  saveRaw: (content: string, token: string) => Promise<RawFileSaveResult>
  listLayers: () => Promise<ConfigLayersResult>
  backupNoticeKey?: string
  policyNoticeKey?: string
  policyLayerIds?: string[]
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
  rawSource?: SettingsRawSource
  managedNotice?: { titleKey: string; descriptionKey: string; actionKey: string; href: string }
  load: (context?: { environmentId: string }) => Promise<SettingsSnapshot>
  save: (payload: SettingsSaveInput) => Promise<SettingsSaveResult>
}
