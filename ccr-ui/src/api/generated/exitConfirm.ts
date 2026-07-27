/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'

export const getSkipExitConfirm = (): Promise<boolean> => invoke('get_skip_exit_confirm')
export const setSkipExitConfirm = (skip: boolean): Promise<void> => invoke('set_skip_exit_confirm', { skip })
