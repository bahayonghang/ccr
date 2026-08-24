import { addCollection } from '@iconify/react'
import { solarShellIconSubset } from '@/config/solarShellIconSubset'

let hasRegisteredShellIcons = false
let deferredIconsPromise: Promise<void> | null = null

// 写入 @iconify/react 的本地缓存（与 SIcon 同源）。原先注册进 Vue 包的缓存，
// React Icon 仍走 API 加载，loader 回调在卸载时因闭包陈旧而不 abort。
export const registerShellIcons = () => {
  if (hasRegisteredShellIcons) return

  addCollection(solarShellIconSubset)
  hasRegisteredShellIcons = true
}

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
