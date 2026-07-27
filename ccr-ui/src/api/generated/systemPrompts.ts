/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'

export const listSystemPrompts = (platform: string): Promise<OpenJsonValueDto> => invoke('system_prompts_list', { platform })
export const getSystemPrompt = (platform: string, id: string): Promise<OpenJsonValueDto> => invoke('system_prompts_get', { platform, id })
export const saveSystemPrompt = (platform: string, id: string, content: string, token: string): Promise<OpenJsonValueDto> => invoke('system_prompts_save', { platform, id, content, token })
export const createSystemPrompt = (platform: string, id: string): Promise<OpenJsonValueDto> => invoke('system_prompts_create', { platform, id })
