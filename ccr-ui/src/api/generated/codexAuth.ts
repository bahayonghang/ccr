/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { CodexAuthActionResponse } from '@/types/generated/codex_auth/CodexAuthActionResponse'
import type { CodexAuthCurrentResponse } from '@/types/generated/codex_auth/CodexAuthCurrentResponse'
import type { CodexAuthListResponse } from '@/types/generated/codex_auth/CodexAuthListResponse'
import type { CodexAuthImportPayload } from '@/types/generated/codex_auth/CodexAuthImportPayload'
import type { CodexAuthMutationResponse } from '@/types/generated/codex_auth/CodexAuthMutationResponse'
import type { CodexAuthProcessResponse } from '@/types/generated/codex_auth/CodexAuthProcessResponse'
import type { CodexAuthRenameResponse } from '@/types/generated/codex_auth/CodexAuthRenameResponse'
import type { CodexApiKeyAddPayload } from '@/types/generated/codex_auth/CodexApiKeyAddPayload'
import type { CodexModelProviderDeleteResponse } from '@/types/generated/codex_auth/CodexModelProviderDeleteResponse'
import type { CodexModelProvidersResponse } from '@/types/generated/codex_auth/CodexModelProvidersResponse'
import type { CodexModelProviderSaveResponse } from '@/types/generated/codex_auth/CodexModelProviderSaveResponse'
import type { CodexModelProviderUpsertPayload } from '@/types/generated/codex_auth/CodexModelProviderUpsertPayload'

import type { CodexOAuthStartResponse } from '@/types/generated/codex_auth/CodexOAuthStartResponse'
import type { OAuthPortReleaseReport } from '@/types/generated/codex_auth/OAuthPortReleaseReport'

export type CodexAuthSaveRequest = { name: string; description?: string; force?: boolean }

export const listCodexAuthAccounts = (): Promise<CodexAuthListResponse> => invoke('codex_list_auth_accounts')
export const getCodexAuthCurrent = (): Promise<CodexAuthCurrentResponse> => invoke('codex_get_auth_current')
export const saveCodexAuth = (request: CodexAuthSaveRequest): Promise<CodexAuthActionResponse> =>
  invoke('codex_save_auth', { name: request.name, description: request.description ?? null, force: request.force ?? false })
export const switchCodexAuth = (name: string): Promise<CodexAuthActionResponse> => invoke('codex_switch_auth', { name })
export const deleteCodexAuth = (name: string): Promise<CodexAuthActionResponse> => invoke('codex_delete_auth', { name })
export const renameCodexAuth = (oldName: string, newName: string, force = false): Promise<CodexAuthRenameResponse> =>
  invoke('codex_rename_auth', { oldName, newName, force })
export const detectCodexProcess = (): Promise<CodexAuthProcessResponse> => invoke('codex_detect_process')
export const codexOAuthLoginStart = (): Promise<CodexOAuthStartResponse> => invoke('codex_oauth_login_start')
export const codexOAuthLoginCompleted = (loginId: string, preferredAccountName?: string | null): Promise<CodexAuthMutationResponse> =>
  invoke('codex_oauth_login_completed', { loginId, preferredAccountName: preferredAccountName ?? null })
export const codexOAuthLoginCancel = (loginId?: string | null): Promise<void> =>
  invoke('codex_oauth_login_cancel', { loginId: loginId ?? null })
export const codexOAuthSubmitCallbackUrl = (loginId: string, callbackUrl: string): Promise<void> =>
  invoke('codex_oauth_submit_callback_url', { loginId, callbackUrl })
export const codexIsOAuthPortInUse = (): Promise<boolean> => invoke('codex_is_oauth_port_in_use')
export const codexReleaseOAuthPort = (): Promise<OAuthPortReleaseReport> => invoke('codex_release_oauth_port')
export const codexOpenExternalUrl = (url: string): Promise<void> => invoke('codex_open_external_url', { url })
export const codexImportAuthPayload = (payload: CodexAuthImportPayload): Promise<CodexAuthMutationResponse> =>
  invoke('codex_import_auth_payload', { payload })
export const codexImportAuthFromLocal = (preferredAccountName?: string | null): Promise<CodexAuthMutationResponse> =>
  invoke('codex_import_auth_from_local', { preferredAccountName: preferredAccountName ?? null })
export const codexAddAuthWithApiKey = (payload: CodexApiKeyAddPayload): Promise<CodexAuthMutationResponse> =>
  invoke('codex_add_auth_with_api_key', { payload })
export const codexListModelProviders = (): Promise<CodexModelProvidersResponse> => invoke('codex_list_model_providers')
export const codexSaveModelProvider = (payload: CodexModelProviderUpsertPayload): Promise<CodexModelProviderSaveResponse> =>
  invoke('codex_save_model_provider', { payload })
export const codexDeleteModelProvider = (providerId: string): Promise<CodexModelProviderDeleteResponse> =>
  invoke('codex_delete_model_provider', { providerId })
