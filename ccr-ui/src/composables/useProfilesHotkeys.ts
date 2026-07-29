// Profiles 页快捷键：⌘K 命令面板 / ⌘1-9 切换启用 profile / 聚焦搜索 / Esc 关闭面板。
// Claude/Codex 两页共用，取代各自重复的 window keydown 实现。
import { onBeforeUnmount, onMounted, type Ref } from 'vue'

export interface UseProfilesHotkeysOptions<T extends { name: string }> {
  /** 命令面板开关状态 */
  paletteOpen: Ref<boolean>
  /** 聚焦搜索框（工具栏 focusSearch） */
  focusSearch: () => void
  /** ⌘1-9 可切换的 profile 列表（按显示顺序），惰性求值以读到最新值 */
  getApplicableProfiles: () => T[]
  /**
   * ⌘1-9 的稳定编号目标（钉选数组，来自 useProfilesQuickSwitch）。
   * 注入后数字键改用该数组（与过滤/排序解耦）；未注入保持旧行为。
   */
  getStableTargets?: () => string[]
  /** ⌘1-9 命中时触发 */
  onApply: (name: string) => void
}

const isEditableTarget = (el: EventTarget | null): boolean => {
  if (!(el instanceof HTMLElement)) return false
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable
}

export function useProfilesHotkeys<T extends { name: string }>(
  options: UseProfilesHotkeysOptions<T>
) {
  const { paletteOpen, focusSearch, getApplicableProfiles, getStableTargets, onApply } = options

  const onWindowKeyDown = (event: KeyboardEvent) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault()
      paletteOpen.value = !paletteOpen.value
      return
    }
    if ((event.metaKey || event.ctrlKey) && /^[1-9]$/.test(event.key)) {
      const idx = Number.parseInt(event.key, 10) - 1
      if (getStableTargets) {
        const name = getStableTargets()[idx]
        if (name) {
          event.preventDefault()
          onApply(name)
        }
        return
      }
      const target = getApplicableProfiles()[idx]
      if (target) {
        event.preventDefault()
        onApply(target.name)
      }
      return
    }
    if (event.key === '/' && !isEditableTarget(event.target)) {
      event.preventDefault()
      focusSearch()
      return
    }
    if (event.key === 'Escape' && paletteOpen.value) {
      paletteOpen.value = false
    }
  }

  onMounted(() => window.addEventListener('keydown', onWindowKeyDown))
  onBeforeUnmount(() => window.removeEventListener('keydown', onWindowKeyDown))
}
