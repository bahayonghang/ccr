import { isTauriRuntime } from '@/utils/tauriRuntime'

type ClientPlatform = 'macos' | 'windows' | 'linux' | 'unknown'

type NavigatorWithUserAgentData = Navigator & {
  userAgentData?: {
    platform?: string
  }
}

const normalizePlatform = (value?: string | null): string => value?.trim().toLowerCase() ?? ''

export const getClientPlatform = (): ClientPlatform => {
  if (typeof navigator === 'undefined') {
    return 'unknown'
  }

  const platform = normalizePlatform((navigator as NavigatorWithUserAgentData).userAgentData?.platform)
    || normalizePlatform(navigator.platform)
    || normalizePlatform(navigator.userAgent)

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

export const shouldUseCustomTitlebar = (): boolean => {
  return !(isTauriRuntime() && getClientPlatform() === 'macos')
}
