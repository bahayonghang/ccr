import { invoke } from '@tauri-apps/api/core'
import { isTauriRuntime } from '@/utils/tauriRuntime'

const SKIP_EXIT_CONFIRM_KEY = 'ccr_skip_exit_confirm'

export const isTauriEnvironment = (): boolean => {
  return isTauriRuntime()
}

export const getEnvironmentName = (): 'tauri' | 'web' => {
  return isTauriEnvironment() ? 'tauri' : 'web'
}

export const getTauriVersion = async (): Promise<string | null> => {
  if (!isTauriEnvironment()) {
    return null
  }

  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    return await getVersion()
  } catch {
    return null
  }
}

export const getSkipExitConfirm = async (): Promise<boolean> => {
  try {
    return await invoke('get_skip_exit_confirm')
  } catch {
    return localStorage.getItem(SKIP_EXIT_CONFIRM_KEY) === '1'
  }
}

export const setSkipExitConfirm = async (skip: boolean): Promise<void> => {
  try {
    await invoke('set_skip_exit_confirm', { skip })
    return
  } catch {
    localStorage.setItem(SKIP_EXIT_CONFIRM_KEY, skip ? '1' : '0')
  }
}

export const TauriAPI = {
  getTauriVersion,
}

export const TauriRuntimeApi = TauriAPI
