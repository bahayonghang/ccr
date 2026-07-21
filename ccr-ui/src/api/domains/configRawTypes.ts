export interface RawFileOk {
  status: 'ok'
  content: string
  token: string
  path: string
  exists: boolean
}

export interface UnsupportedEnvironment {
  status: 'unsupported_environment'
  envType: string
}

export type RawFileGetResult = RawFileOk | UnsupportedEnvironment

export interface RawFileSaved {
  status: 'saved'
  token: string
}

export interface RawFileConflict {
  status: 'conflict'
}

export interface RawFileInvalid {
  status: 'invalid'
  kind: 'syntax' | 'semantic'
  message: string
  line?: number
  column?: number
}

export type RawFileSaveResult =
  | RawFileSaved
  | RawFileConflict
  | RawFileInvalid
  | UnsupportedEnvironment

export interface RawProfilesSaved extends RawFileSaved {
  profiles_count: number
}

export interface RawProfilesActivationConflict {
  status: 'activation_conflict'
  current: string
}

export type RawProfilesSaveResult =
  | RawProfilesSaved
  | RawFileConflict
  | RawFileInvalid
  | RawProfilesActivationConflict
  | UnsupportedEnvironment

export interface ConfigLayer {
  id: string
  label: string
  path: string | null
  exists: boolean | null
  size: number | null
  mtime: number | null
  editable: boolean
}

export type ConfigLayersResult =
  | { layers: ConfigLayer[] }
  | UnsupportedEnvironment
