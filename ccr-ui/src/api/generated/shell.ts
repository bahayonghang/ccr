/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@tauri-apps/api/core'
import type { DesktopShellPreferences } from '@/types/generated/shell/DesktopShellPreferences'
import type { SkillportAppStatus } from '@/types/generated/shell/SkillportAppStatus'
import type { TrayPanelManualPosition } from '@/types/generated/shell/TrayPanelManualPosition'

export const shellGetPreferences = (): Promise<DesktopShellPreferences> => invoke('shell_get_preferences')
export const shellSetPreferences = (preferences: DesktopShellPreferences): Promise<DesktopShellPreferences> => invoke('shell_set_preferences', { preferences })
export const shellShowMainWindow = (targetRoute?: string): Promise<void> => invoke('shell_show_main_window', { targetRoute })
export const shellRequestQuit = (): Promise<void> => invoke('shell_request_quit')
export const shellBeginTrayPanelDrag = (): Promise<void> => invoke('shell_begin_tray_panel_drag')
export const shellCompleteTrayPanelDrag = (position?: TrayPanelManualPosition | null): Promise<void> => invoke('shell_complete_tray_panel_drag', { position: position ?? null })
export const shellDetectSkillportApp = (): Promise<SkillportAppStatus> => invoke('shell_detect_skillport_app')
export const shellOpenSkillportApp = (): Promise<void> => invoke('shell_open_skillport_app')
export const shellDetectSkillsManageApp = (): Promise<SkillportAppStatus> => invoke('shell_detect_skills_manage_app')
export const shellOpenSkillsManageApp = (): Promise<void> => invoke('shell_open_skills_manage_app')
