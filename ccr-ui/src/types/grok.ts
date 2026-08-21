export type { GrokActivationDto } from './generated/grok/GrokActivationDto'
export type { GrokAuthCurrentResponse } from './generated/grok/GrokAuthCurrentResponse'
export type { GrokAuthModeDto } from './generated/grok/GrokAuthModeDto'
export type { GrokAuthOffResponse } from './generated/grok/GrokAuthOffResponse'
export type { GrokCliSettingsDto } from './generated/grok/GrokCliSettingsDto'
export type { GrokConfigLayerDto } from './generated/grok/GrokConfigLayerDto'
export type { GrokConfigLayersResponse } from './generated/grok/GrokConfigLayersResponse'
export type { GrokCustomModelDto } from './generated/grok/GrokCustomModelDto'
export type { GrokDashboardCommandResponse } from './generated/grok/GrokDashboardCommandResponse'
export type { GrokDashboardOverview } from './generated/grok/GrokDashboardOverview'
export type { GrokDeleteBlockedReasonDto } from './generated/grok/GrokDeleteBlockedReasonDto'
export type { GrokHintsSettingsDto } from './generated/grok/GrokHintsSettingsDto'
export type { GrokModelsSettingsDto } from './generated/grok/GrokModelsSettingsDto'
export type { GrokProfileActionResponse } from './generated/grok/GrokProfileActionResponse'
export type { GrokProfileCommandResponse } from './generated/grok/GrokProfileCommandResponse'
export type { GrokProfileDto } from './generated/grok/GrokProfileDto'
export type { GrokProfileKindDto } from './generated/grok/GrokProfileKindDto'
export type { GrokProfilesCommandResponse } from './generated/grok/GrokProfilesCommandResponse'
export type { GrokProfilesResponse } from './generated/grok/GrokProfilesResponse'
export type { GrokRawConfigResponse } from './generated/grok/GrokRawConfigResponse'
export type { GrokRawSaveResponse } from './generated/grok/GrokRawSaveResponse'
export type { GrokSessionSettingsDto } from './generated/grok/GrokSessionSettingsDto'
export type { GrokSettingsCommandResponse } from './generated/grok/GrokSettingsCommandResponse'
export type { GrokSettingsPatchDto } from './generated/grok/GrokSettingsPatchDto'
export type { GrokSettingsResponse } from './generated/grok/GrokSettingsResponse'
export type { GrokSettingsUpdateResponse } from './generated/grok/GrokSettingsUpdateResponse'
export type { GrokUiSettingsDto } from './generated/grok/GrokUiSettingsDto'

export type GrokCredentialAction =
  | 'preserve'
  | 'replace_api_key'
  | 'replace_env_key'
  | 'clear'

export type GrokApiBackend = 'chat_completions' | 'responses' | 'messages'
export type GrokReasoningEffort = 'none' | 'minimal' | 'low' | 'medium' | 'high' | 'xhigh' | 'max'

export interface GrokProfileCreateRequest {
  name: string
  description: string | null
  profile_kind: import('./generated/grok/GrokProfileKindDto').GrokProfileKindDto
  base_url?: string
  model?: string
  provider: string | null
  enabled: boolean
  tags: string[] | null
  api_backend?: GrokApiBackend
  context_window?: number
  supports_backend_search?: boolean
  reasoning_effort?: GrokReasoningEffort
  credential_action: GrokCredentialAction
  api_key?: string
  env_key?: string
}

export interface GrokProfilePatch {
  name?: string
  description?: string | null
  profile_kind?: import('./generated/grok/GrokProfileKindDto').GrokProfileKindDto
  base_url?: string | null
  model?: string | null
  provider?: string | null
  enabled?: boolean
  tags?: string[] | null
  api_backend?: GrokApiBackend | null
  context_window?: number | null
  supports_backend_search?: boolean | null
  reasoning_effort?: GrokReasoningEffort | null
  credential_action?: GrokCredentialAction
  api_key?: string
  env_key?: string
}
