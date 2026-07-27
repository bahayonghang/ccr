/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { CommandCatalog } from '@/types/generated/command_exec/CommandCatalog'
import type { CommandExecutionResult } from '@/types/generated/command_exec/CommandExecutionResult'
import type { CommandHelpResponse } from '@/types/generated/command_exec/CommandHelpResponse'
import type { CommandJobSnapshot } from '@/types/generated/command_exec/CommandJobSnapshot'
import type { StartCommandJobResponse } from '@/types/generated/command_exec/StartCommandJobResponse'

export type ExecuteCcrCommandInput = {
  command: string
  args?: string[]
  confirmationToken?: string | null
}

export const executeCcrCommand = (input: ExecuteCcrCommandInput): Promise<CommandExecutionResult> =>
  invoke('execute_ccr_command', input)

export const listCcrCommands = (): Promise<CommandCatalog> =>
  invoke('list_ccr_commands')

export const getCcrCommandHelp = (command: string): Promise<CommandHelpResponse> =>
  invoke('get_ccr_command_help', { command })

export const startCcrCommandJob = (input: ExecuteCcrCommandInput): Promise<StartCommandJobResponse> =>
  invoke('start_ccr_command_job', input)

export const getCcrCommandJobStatus = (jobId: string): Promise<CommandJobSnapshot> =>
  invoke('get_ccr_command_job_status', { jobId })

export const cancelCcrCommandJob = (jobId: string): Promise<CommandJobSnapshot> =>
  invoke('cancel_ccr_command_job', { jobId })
