import { invoke } from '@tauri-apps/api/core'
import type { UnsupportedEnvironment } from './configRawTypes'

export interface SystemPromptFile {
  id: string
  labelKey: string
  path: string
  exists: boolean
  size: number | null
  mtime: number | null
  editable: boolean
  limitHint: number | null
}

export interface SystemPromptRule {
  name: string
  path: string
  size: number | null
}

export type SystemPromptsListResult =
  | { status: 'ok'; files: SystemPromptFile[]; rules: SystemPromptRule[] }
  | UnsupportedEnvironment

export type SystemPromptGetResult =
  | {
      status: 'ok'
      content: string
      token: string
      path: string
      exists: boolean
      limitHint: number | null
    }
  | UnsupportedEnvironment

export type SystemPromptWriteResult =
  | { status: 'saved'; token: string; warning?: 'size'; limitHint?: number }
  | { status: 'conflict' }
  | UnsupportedEnvironment

export const listSystemPrompts = async (platform: string): Promise<SystemPromptsListResult> => {
  return invoke('system_prompts_list', { platform })
}

export const getSystemPrompt = async (
  platform: string,
  id: string,
): Promise<SystemPromptGetResult> => {
  return invoke('system_prompts_get', { platform, id })
}

export const saveSystemPrompt = async (
  platform: string,
  id: string,
  content: string,
  token: string,
): Promise<SystemPromptWriteResult> => {
  return invoke('system_prompts_save', { platform, id, content, token })
}

export const createSystemPrompt = async (
  platform: string,
  id: string,
): Promise<SystemPromptWriteResult> => {
  return invoke('system_prompts_create', { platform, id })
}
