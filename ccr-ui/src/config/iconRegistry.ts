import { addCollection } from '@iconify/vue'
import { solarShellIconSubset } from '@/config/solarShellIconSubset'

let hasRegisteredShellIcons = false
let deferredIconsPromise: Promise<void> | null = null

// 首屏只注册壳层与首页必需图标，降低启动链路体积。
export const registerShellIcons = () => {
  if (hasRegisteredShellIcons) return

  addCollection(solarShellIconSubset)
  hasRegisteredShellIcons = true
}

// 首帧之后再补齐完整子集，继续保持离线图标能力。
export const registerDeferredIcons = (): Promise<void> => {
  if (deferredIconsPromise) {
    return deferredIconsPromise
  }

  deferredIconsPromise = import('@/config/solarIconSubset')
    .then(({ solarIconSubset }) => {
      addCollection(solarIconSubset)
    })
    .catch((error) => {
      deferredIconsPromise = null
      throw error
    })

  return deferredIconsPromise
}

export const registerAppIcons = async () => {
  registerShellIcons()
  await registerDeferredIcons()
}
