import { addCollection } from '@iconify/vue'
import { solarIconSubset } from '@/config/solarIconSubset'

let hasRegisteredAppIcons = false

// 在应用挂载前注册本地图标子集，避免首屏回源到 Iconify API。
export const registerAppIcons = () => {
  if (hasRegisteredAppIcons) return

  addCollection(solarIconSubset)
  hasRegisteredAppIcons = true
}
