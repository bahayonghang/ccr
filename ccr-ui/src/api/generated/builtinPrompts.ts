/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { BuiltinPromptDto } from '@/types/generated/builtin_prompts/BuiltinPromptDto'

export const listBuiltinPrompts = (): Promise<BuiltinPromptDto[]> => invoke('list_builtin_prompts')
export const getBuiltinPrompt = (id: string): Promise<BuiltinPromptDto | null> => invoke('get_builtin_prompt', { id })
export const getBuiltinPromptsByCategory = (category: string): Promise<BuiltinPromptDto[]> => invoke('get_builtin_prompts_by_category', { category })
