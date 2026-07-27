/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'

export const getGeminiSettings = (): Promise<OpenJsonValueDto> => invoke('gemini_get_settings')
export const updateGeminiSettings = (settings: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_update_settings', { settings })
export const listGeminiMcpServers = (): Promise<OpenJsonValueDto> => invoke('gemini_list_mcp_servers')
export const addGeminiMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_add_mcp_server', { name, config })
export const updateGeminiMcpServer = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_update_mcp_server', { name, config })
export const deleteGeminiMcpServer = (name: string): Promise<string> => invoke('gemini_delete_mcp_server', { name })
export const listGeminiSlashCommands = (): Promise<OpenJsonValueDto> => invoke('gemini_list_slash_commands')
export const addGeminiSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_add_slash_command', { name, config })
export const updateGeminiSlashCommand = (name: string, config: OpenJsonValueDto): Promise<OpenJsonValueDto> => invoke('gemini_update_slash_command', { name, config })
export const deleteGeminiSlashCommand = (name: string): Promise<string> => invoke('gemini_delete_slash_command', { name })
export const listGeminiExtensions = (): Promise<OpenJsonValueDto> => invoke('gemini_list_extensions')
