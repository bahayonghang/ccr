/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'
import type { GrokAuthCurrentResponse } from '@/types/generated/grok/GrokAuthCurrentResponse'
import type { GrokAuthOffResponse } from '@/types/generated/grok/GrokAuthOffResponse'
import type { GrokConfigLayersResponse } from '@/types/generated/grok/GrokConfigLayersResponse'
import type { GrokDashboardCommandResponse } from '@/types/generated/grok/GrokDashboardCommandResponse'
import type { GrokProfileActionResponse } from '@/types/generated/grok/GrokProfileActionResponse'
import type { GrokProfileCommandResponse } from '@/types/generated/grok/GrokProfileCommandResponse'
import type { GrokProfilesCommandResponse } from '@/types/generated/grok/GrokProfilesCommandResponse'
import type { GrokRawConfigResponse } from '@/types/generated/grok/GrokRawConfigResponse'
import type { GrokRawSaveResponse } from '@/types/generated/grok/GrokRawSaveResponse'
import type { GrokSettingsCommandResponse } from '@/types/generated/grok/GrokSettingsCommandResponse'
import type { GrokSettingsPatchDto } from '@/types/generated/grok/GrokSettingsPatchDto'
import type { GrokSettingsUpdateResponse } from '@/types/generated/grok/GrokSettingsUpdateResponse'

export const listGrokProfiles = (): Promise<GrokProfilesCommandResponse> => invoke('grok_list_profiles')
export const getGrokProfile = (name: string): Promise<GrokProfileCommandResponse> => invoke('grok_get_profile', { name })
export const addGrokProfile = (request: OpenJsonValueDto): Promise<GrokProfileActionResponse> => invoke('grok_add_profile', { request })
export const updateGrokProfile = (name: string, patch: OpenJsonValueDto): Promise<GrokProfileActionResponse> => invoke('grok_update_profile', { name, patch })
export const deleteGrokProfile = (name: string, options?: { force?: boolean }): Promise<GrokProfileActionResponse> => invoke('grok_delete_profile', { name, force: options?.force })
export const applyGrokProfile = (name: string): Promise<GrokProfileActionResponse> => invoke('grok_apply_profile', { name })
export const grokProfileOff = (): Promise<GrokProfileActionResponse> => invoke('grok_profile_off')
export const grokAuthCurrent = (): Promise<GrokAuthCurrentResponse> => invoke('grok_auth_current')
export const grokAuthOff = (): Promise<GrokAuthOffResponse> => invoke('grok_auth_off')
export const getGrokSettings = (): Promise<GrokSettingsCommandResponse> => invoke('grok_get_settings')
export const updateGrokSettings = (patch: GrokSettingsPatchDto): Promise<GrokSettingsUpdateResponse> => invoke('grok_update_settings', { patch })
export const getGrokConfigRaw = (): Promise<GrokRawConfigResponse> => invoke('grok_get_config_raw_text')
export const saveGrokConfigRaw = (content: string, token: string): Promise<GrokRawSaveResponse> => invoke('grok_save_config_raw_text', { content, token })
export const listGrokConfigLayers = (): Promise<GrokConfigLayersResponse> => invoke('grok_list_config_layers')
export const getGrokDashboardOverview = (): Promise<GrokDashboardCommandResponse> => invoke('grok_get_dashboard_overview')
