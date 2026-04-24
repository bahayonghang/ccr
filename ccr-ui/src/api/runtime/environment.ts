import { invoke } from '@tauri-apps/api/core'
import { isTauriRuntime } from '@/utils/tauriRuntime'

const SKIP_EXIT_CONFIRM_KEY = 'ccr_skip_exit_confirm'

export interface DesktopShellPreferences {
  confirm_before_exit: boolean
  close_to_tray: boolean
  open_panel_on_tray_click: boolean
  tray_panel: TrayPanelPlacementState
}

export interface TrayPanelManualPosition {
  x: number
  y: number
}

export interface TrayPanelPlacementState {
  placement_mode: 'anchored' | 'manual'
  manual_position: TrayPanelManualPosition | null
}

export type SkillsManageAppPlatform = 'windows' | 'macos' | 'other'
export type SkillsManageAppSource = 'bundle_id' | 'registry' | 'known_path' | 'unsupported' | 'not_found'

export interface SkillsManageAppStatus {
  supported: boolean
  installed: boolean
  platform: SkillsManageAppPlatform
  source: SkillsManageAppSource
}

export interface EnvironmentInfo {
  id: string
  name: string
  env_type: string
  is_active: boolean
  description: string
}

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

export const listEnvironments = async (): Promise<EnvironmentInfo[]> => {
  return invoke('list_environments')
}

export const getCurrentEnvironment = async (): Promise<EnvironmentInfo | null> => {
  return invoke('get_current_environment')
}

export const switchEnvironment = async (envId: string): Promise<void> => {
  await invoke('switch_environment', { envId })
}

export const refreshEnvironments = async (): Promise<EnvironmentInfo[]> => {
  return invoke('refresh_environments')
}

export const shellGetPreferences = async (): Promise<DesktopShellPreferences> => {
  return invoke('shell_get_preferences')
}

export const shellSetPreferences = async (
  preferences: DesktopShellPreferences
): Promise<DesktopShellPreferences> => {
  return invoke('shell_set_preferences', { preferences })
}

export const shellShowMainWindow = async (targetRoute?: string): Promise<void> => {
  await invoke('shell_show_main_window', { targetRoute })
}

export const shellRequestQuit = async (): Promise<void> => {
  await invoke('shell_request_quit')
}

export const shellBeginTrayPanelDrag = async (): Promise<void> => {
  await invoke('shell_begin_tray_panel_drag')
}

export const shellCompleteTrayPanelDrag = async (
  position?: TrayPanelManualPosition | null
): Promise<void> => {
  await invoke('shell_complete_tray_panel_drag', { position: position ?? null })
}

export const detectSkillsManageApp = async (): Promise<SkillsManageAppStatus> => {
  return invoke('shell_detect_skills_manage_app')
}

export const openSkillsManageApp = async (): Promise<void> => {
  await invoke('shell_open_skills_manage_app')
}

export const getSkipExitConfirm = async (): Promise<boolean> => {
  try {
    const preferences = await shellGetPreferences()
    return !preferences.confirm_before_exit
  } catch {
    return localStorage.getItem(SKIP_EXIT_CONFIRM_KEY) === '1'
  }
}

export const setSkipExitConfirm = async (skip: boolean): Promise<void> => {
  try {
    const preferences = await shellGetPreferences()
    await shellSetPreferences({
      ...preferences,
      confirm_before_exit: !skip,
    })
    return
  } catch {
    localStorage.setItem(SKIP_EXIT_CONFIRM_KEY, skip ? '1' : '0')
  }
}

export const TauriAPI = {
  getTauriVersion,
}

export const TauriRuntimeApi = TauriAPI
