/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { ClaudeAuthActionResponse } from '@/types/generated/claude_auth/ClaudeAuthActionResponse'
import type { ClaudeAuthCurrentResponse } from '@/types/generated/claude_auth/ClaudeAuthCurrentResponse'
import type { ClaudeAuthListResponse } from '@/types/generated/claude_auth/ClaudeAuthListResponse'

export type ClaudeAuthSaveRequest = {
  name: string
  description?: string | null
  force?: boolean
}

export const listClaudeAuthAccounts = (): Promise<ClaudeAuthListResponse> =>
  invoke('claude_list_auth_accounts')

export const getClaudeAuthCurrent = (): Promise<ClaudeAuthCurrentResponse> =>
  invoke('claude_get_auth_current')

export const saveClaudeAuth = (request: ClaudeAuthSaveRequest): Promise<ClaudeAuthActionResponse> =>
  invoke('claude_save_auth', {
    name: request.name,
    description: request.description ?? null,
    force: request.force ?? false,
  })

export const switchClaudeAuth = (name: string): Promise<ClaudeAuthActionResponse> =>
  invoke('claude_switch_auth', { name })

export const deleteClaudeAuth = (name: string): Promise<ClaudeAuthActionResponse> =>
  invoke('claude_delete_auth', { name })
