import { isTauriRuntime } from '@/utils/tauriRuntime'

type ClientPlatform = 'macos' | 'windows' | 'linux' | 'unknown'
export type WindowChromeMode = 'native' | 'custom'

type NavigatorWithUserAgentData = Navigator & {
  userAgentData?: {
    platform?: string
  }
}

const normalizePlatform = (value?: string | null): string => value?.trim().toLowerCase() ?? ''

export const resolveClientPlatform = (platformHint?: string | null): ClientPlatform => {
  const platform = normalizePlatform(platformHint)

  if (platform.includes('mac')) {
    return 'macos'
  }

  if (platform.includes('win')) {
    return 'windows'
  }

  if (platform.includes('linux') || platform.includes('x11')) {
    return 'linux'
  }

  return 'unknown'
}

export const getClientPlatform = (): ClientPlatform => {
  if (typeof navigator === 'undefined') {
    return 'unknown'
  }

  const platform = (navigator as NavigatorWithUserAgentData).userAgentData?.platform
    || navigator.platform
    || navigator.userAgent

  return resolveClientPlatform(platform)
}

export const resolveWindowChromeMode = (
  isTauri: boolean,
  platform: ClientPlatform,
): WindowChromeMode => {
  if (isTauri && platform === 'macos') {
    return 'native'
  }

  return 'custom'
}

export const getWindowChromeMode = (): WindowChromeMode => {
  return resolveWindowChromeMode(isTauriRuntime(), getClientPlatform())
}

export const shouldUseNativeWindowChrome = (): boolean => {
  return getWindowChromeMode() === 'native'
}

export const shouldUseCustomTitlebar = (): boolean => {
  return getWindowChromeMode() === 'custom'
}

export const getWindowChromeTopInset = (): number => {
  return shouldUseCustomTitlebar() ? 36 : 0
}
