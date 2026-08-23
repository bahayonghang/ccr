// Profiles 页快捷键：⌘K 命令面板 / ⌘1-9 切换启用 profile / 聚焦搜索 / Esc 关闭面板。
// Claude/Codex/Grok 三页共用，取代各自重复的 window keydown 实现。
//
// 08-22-state-logic-port 批次 5c：Vue → React。onMounted/onBeforeUnmount →
// useEffect + cleanup；分支逻辑逐行保留。
//
// 签名变化（消费方均为待迁移 .vue 视图）：
// - paletteOpen：Ref<boolean> → 普通 boolean；
// - 新增 setPaletteOpen（原回调内对 paletteOpen.value 的直接写入改为经 setter，
//   ⌘K 用函数式更新切换，Esc 置 false）。

import { useEffect } from 'react'
import type { Dispatch, SetStateAction } from 'react'

export interface UseProfilesHotkeysOptions {
  /** 命令面板开关状态 */
  paletteOpen: boolean
  /** 命令面板开关状态写入 */
  setPaletteOpen: Dispatch<SetStateAction<boolean>>
  /** 聚焦搜索框（工具栏 focusSearch） */
  focusSearch: () => void
  /** ⌘1-9 的稳定编号目标（钉选数组，来自 useProfilesQuickSwitch） */
  getStableTargets: () => string[]
  /** ⌘1-9 命中时触发 */
  onApply: (name: string) => void
}

const isEditableTarget = (el: EventTarget | null): boolean => {
  if (!(el instanceof HTMLElement)) return false
  const tag = el.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable
}

export function useProfilesHotkeys(options: UseProfilesHotkeysOptions) {
  const { paletteOpen, setPaletteOpen, focusSearch, getStableTargets, onApply } = options

  useEffect(() => {
    const onWindowKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setPaletteOpen((open) => !open)
        return
      }
      if ((event.metaKey || event.ctrlKey) && /^[1-9]$/.test(event.key)) {
        const idx = Number.parseInt(event.key, 10) - 1
        const name = getStableTargets()[idx]
        if (name) {
          event.preventDefault()
          onApply(name)
        }
        return
      }
      if (event.key === '/' && !isEditableTarget(event.target)) {
        event.preventDefault()
        focusSearch()
        return
      }
      if (event.key === 'Escape' && paletteOpen) {
        setPaletteOpen(false)
      }
    }

    window.addEventListener('keydown', onWindowKeyDown)
    return () => window.removeEventListener('keydown', onWindowKeyDown)
  }, [paletteOpen, setPaletteOpen, focusSearch, getStableTargets, onApply])
}
