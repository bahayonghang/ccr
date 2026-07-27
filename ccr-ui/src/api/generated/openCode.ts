/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'
import type { OpenCodePluginFileRecord } from '@/types/generated/opencode/OpenCodePluginFileRecord'
import type { OpenCodeThemeRecord } from '@/types/generated/opencode/OpenCodeThemeRecord'

export const getOpenCodeSettings = (): Promise<OpenJsonValueDto> => invoke('opencode_get_settings')
export const updateOpenCodeSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_settings', { settings })
export const getOpenCodeTuiSettings = (): Promise<OpenJsonValueDto> => invoke('opencode_get_tui_settings')
export const updateOpenCodeTuiSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_tui_settings', { settings })
export const getOpenCodeKeybindings = (): Promise<OpenJsonValueDto> => invoke('opencode_get_keybindings')
export const updateOpenCodeKeybindings = (keybindings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_keybindings', { keybindings })
export const listOpenCodeThemes = (): Promise<OpenCodeThemeRecord[]> => invoke('opencode_list_themes')
export const listOpenCodeAgents = (): Promise<OpenJsonValueDto> => invoke('opencode_list_agents')
export const addOpenCodeAgent = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_add_agent', { config })
export const updateOpenCodeAgent = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_agent', { config })
export const deleteOpenCodeAgent = (name: string, context?: OpenJsonValueDto): Promise<string> => invoke('opencode_delete_agent', { name, context })
export const listOpenCodeCommands = (): Promise<OpenJsonValueDto> => invoke('opencode_list_commands')
export const addOpenCodeCommand = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_add_command', { config })
export const updateOpenCodeCommand = (config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('opencode_update_command', { config })
export const deleteOpenCodeCommand = (name: string, context?: OpenJsonValueDto): Promise<string> => invoke('opencode_delete_command', { name, context })
export const listOpenCodeLocalPlugins = (): Promise<OpenCodePluginFileRecord[]> => invoke('opencode_list_local_plugins')
