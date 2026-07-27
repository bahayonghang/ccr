import { isTauriRuntime } from '@/utils/tauriRuntime'
import {
  getCurrentEnvironment as getCurrentEnvironmentTyped,
  listEnvironments as listEnvironmentsTyped,
  refreshEnvironments as refreshEnvironmentsTyped,
  switchEnvironment as switchEnvironmentTyped,
} from '../generated/environment'
import type { EnvironmentInfo } from '@/types/generated/environment/EnvironmentInfo'
import {
  shellBeginTrayPanelDrag as shellBeginTrayPanelDragTyped,
  shellCompleteTrayPanelDrag as shellCompleteTrayPanelDragTyped,
  shellDetectSkillportApp,
  shellDetectSkillsManageApp,
  shellGetPreferences as shellGetPreferencesTyped,
  shellOpenSkillportApp,
  shellOpenSkillsManageApp,
  shellRequestQuit as shellRequestQuitTyped,
  shellSetPreferences as shellSetPreferencesTyped,
  shellShowMainWindow as shellShowMainWindowTyped,
} from '../generated/shell'
import type { DesktopShellPreferences as GeneratedDesktopShellPreferences } from '@/types/generated/shell/DesktopShellPreferences'
import type { SkillportAppStatus as GeneratedSkillportAppStatus } from '@/types/generated/shell/SkillportAppStatus'
import type { TrayPanelManualPosition as GeneratedTrayPanelManualPosition } from '@/types/generated/shell/TrayPanelManualPosition'
import type { TrayPanelPlacementState as GeneratedTrayPanelPlacementState } from '@/types/generated/shell/TrayPanelPlacementState'

const SKIP_EXIT_CONFIRM_KEY = 'ccr_skip_exit_confirm'

export type DesktopShellPreferences = GeneratedDesktopShellPreferences
export type TrayPanelManualPosition = GeneratedTrayPanelManualPosition
export type TrayPanelPlacementState = GeneratedTrayPanelPlacementState
export type SkillportAppStatus = GeneratedSkillportAppStatus
export type SkillportAppPlatform = SkillportAppStatus['platform']
export type SkillportAppSource = SkillportAppStatus['source']

export type SkillsManageAppPlatform = SkillportAppPlatform
export type SkillsManageAppSource = SkillportAppSource
export type SkillsManageAppStatus = SkillportAppStatus

export type { EnvironmentInfo }

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

const requireTauriEnvironment = (command: string): void => {
  if (!isTauriEnvironment()) {
    throw new Error(`Tauri runtime is unavailable for ${command}`)
  }
}

export const listEnvironments = async (): Promise<EnvironmentInfo[]> => {
  requireTauriEnvironment('list_environments')
  return listEnvironmentsTyped()
}

export const getCurrentEnvironment = async (): Promise<EnvironmentInfo> => {
  requireTauriEnvironment('get_current_environment')
  return getCurrentEnvironmentTyped()
}

export const switchEnvironment = async (envId: string): Promise<void> => {
  requireTauriEnvironment('switch_environment')
  await switchEnvironmentTyped(envId)
}

export const refreshEnvironments = async (): Promise<EnvironmentInfo[]> => {
  requireTauriEnvironment('refresh_environments')
  return refreshEnvironmentsTyped()
}

export const shellGetPreferences = async (): Promise<DesktopShellPreferences> => {
  requireTauriEnvironment('shell_get_preferences')
  return shellGetPreferencesTyped()
}

export const shellSetPreferences = async (
  preferences: DesktopShellPreferences,
): Promise<DesktopShellPreferences> => {
  requireTauriEnvironment('shell_set_preferences')
  return shellSetPreferencesTyped(preferences)
}

export const shellShowMainWindow = async (targetRoute?: string): Promise<void> => {
  requireTauriEnvironment('shell_show_main_window')
  await shellShowMainWindowTyped(targetRoute)
}

export const shellRequestQuit = async (): Promise<void> => {
  requireTauriEnvironment('shell_request_quit')
  await shellRequestQuitTyped()
}

export const shellBeginTrayPanelDrag = async (): Promise<void> => {
  requireTauriEnvironment('shell_begin_tray_panel_drag')
  await shellBeginTrayPanelDragTyped()
}

export const shellCompleteTrayPanelDrag = async (
  position?: TrayPanelManualPosition | null,
): Promise<void> => {
  requireTauriEnvironment('shell_complete_tray_panel_drag')
  await shellCompleteTrayPanelDragTyped(position)
}

export const detectSkillportApp = async (): Promise<SkillportAppStatus> => {
  requireTauriEnvironment('shell_detect_skillport_app')
  return shellDetectSkillportApp()
}

export const openSkillportApp = async (): Promise<void> => {
  requireTauriEnvironment('shell_open_skillport_app')
  await shellOpenSkillportApp()
}

// Legacy aliases kept for migration period.
export const detectSkillsManageApp = async (): Promise<SkillsManageAppStatus> => {
  requireTauriEnvironment('shell_detect_skills_manage_app')
  return shellDetectSkillsManageApp()
}

// Legacy aliases kept for migration period.
export const openSkillsManageApp = async (): Promise<void> => {
  requireTauriEnvironment('shell_open_skills_manage_app')
  await shellOpenSkillsManageApp()
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
