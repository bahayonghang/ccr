// Profiles 快速切换 React hook（08-22-state-logic-port 批次 5c）。
// 状态承载已迁 `features/profiles/stores.ts`（Zustand 跨页共享，localStorage 按平台键
// 逐 key 手动读写）；本文件是按平台取值的薄封装，导出名不变。
//
// 签名变化（消费方均为待迁移 .vue 视图 / ProfilesQuickRail / useProfilesHotkeys）：
// - pinned/recent：Ref<string[]> → string[]（store 选择器直读，空平台回退共享空数组
//   常量保证引用稳定）；recentNotPinned/stableTargets/modifier/canPin 由 computed 改为
//   useMemo/普通值；
// - 新增 setQuery 无——本 hook 无查询词；动作函数名与语义不变。
//
// watch(:96) 映射登记（classification §2）：原
// `watch(() => options.getProfileNames(), cleanupStale, { immediate: true, flush: 'sync' })`
// → useEffect([getProfileNames, storeCleanupStale])。immediate: true 由 effect 首次执行
// 覆盖；flush: 'sync' 无等价物，退化为渲染后 effect 时序；cleanupStale 幂等（列表无变化
// 不写 state），消费方每渲染重建 getter 引起的重跑无副作用。

import { useCallback, useEffect, useMemo } from 'react'
import { getClientPlatform } from '@/utils/windowChrome'
import { PROFILES_PIN_CAP, useProfilesQuickSwitchStore } from '@/features/profiles/stores'

export { PROFILES_PIN_CAP }

export interface UseProfilesQuickSwitchOptions {
  /** 平台键，例如 'claude' / 'codex'，用于 localStorage 键后缀 */
  platform: string
  /**
   * 当前存在的 profile 名列表（惰性求值，用于 stale 名称清理）。
   * 首次成功加载前返回 null，避免用尚未就绪的空列表清空持久化状态。
   */
  getProfileNames: () => string[] | null
  /** 钉选已达上限时的提示回调（视图接 toast），缺省静默拒绝 */
  onPinLimit?: () => void
}

export interface ProfilesQuickSwitch {
  /** 钉选数组（顺序 = 用户钉选操作顺序），数字编号的唯一来源 */
  pinned: string[]
  /** 最近使用列表（recordUse 时间倒序，含已钉选项） */
  recent: string[]
  /** 最近列表中未钉选的部分：只展示，永不编号 */
  recentNotPinned: string[]
  /** ⌘/Ctrl+数字键的稳定目标数组（= 钉选数组，最多 8 个） */
  stableTargets: string[]
  /** 平台修饰键展示文案：macos → '⌘'，其余 → 'Ctrl' */
  modifier: 'Ctrl' | '⌘'
  isPinned: (name: string) => boolean
  /** 是否还能继续钉选（未达上限） */
  canPin: boolean
  /** 钉选；已达上限或已钉选时返回 false（达上限会触发 onPinLimit） */
  pin: (name: string) => boolean
  unpin: (name: string) => void
  togglePin: (name: string) => void
  /** Apply 成功后记录最近使用；只影响 recent，不影响钉选编号 */
  recordUse: (name: string) => void
  /** 重命名跟随：视图在 rename 成功后调用，钉选/最近中的旧名替换为新名 */
  renamePinned: (oldName: string, newName: string) => void
}

const EMPTY_NAMES: string[] = []

export function useProfilesQuickSwitch(options: UseProfilesQuickSwitchOptions): ProfilesQuickSwitch {
  const { platform, getProfileNames, onPinLimit } = options

  const pinned = useProfilesQuickSwitchStore((s) => s.pinnedByPlatform[platform] ?? EMPTY_NAMES)
  const recent = useProfilesQuickSwitchStore((s) => s.recentByPlatform[platform] ?? EMPTY_NAMES)
  const storePin = useProfilesQuickSwitchStore((s) => s.pin)
  const storeUnpin = useProfilesQuickSwitchStore((s) => s.unpin)
  const storeRecordUse = useProfilesQuickSwitchStore((s) => s.recordUse)
  const storeRenamePinned = useProfilesQuickSwitchStore((s) => s.renamePinned)
  const storeCleanupStale = useProfilesQuickSwitchStore((s) => s.cleanupStale)

  // 原 watch(useProfilesQuickSwitch.ts:96)，映射见文件头登记。
  useEffect(() => {
    storeCleanupStale(platform, getProfileNames())
  }, [platform, getProfileNames, storeCleanupStale])

  // 原 computed(:102)：来源 recent、pinned
  const recentNotPinned = useMemo(
    () => recent.filter((name) => !pinned.includes(name)),
    [recent, pinned],
  )

  // 原 computed(:106)：来源 pinned
  const stableTargets = useMemo(() => pinned.slice(0, PROFILES_PIN_CAP), [pinned])

  // 平台修饰键为会话期常量（getClientPlatform 非响应式读取）
  const modifier = useMemo<'Ctrl' | '⌘'>(
    () => (getClientPlatform() === 'macos' ? '⌘' : 'Ctrl'),
    [],
  )

  const isPinned = useCallback((name: string) => pinned.includes(name), [pinned])

  // 原 computed(:114)：来源 pinned.length
  const canPin = pinned.length < PROFILES_PIN_CAP

  const pin = useCallback(
    (name: string) => storePin(platform, name, onPinLimit),
    [storePin, platform, onPinLimit],
  )

  const unpin = useCallback(
    (name: string) => storeUnpin(platform, name),
    [storeUnpin, platform],
  )

  const togglePin = useCallback(
    (name: string) => {
      if (pinned.includes(name)) storeUnpin(platform, name)
      else storePin(platform, name, onPinLimit)
    },
    [pinned, platform, onPinLimit, storePin, storeUnpin],
  )

  const recordUse = useCallback(
    (name: string) => storeRecordUse(platform, name),
    [storeRecordUse, platform],
  )

  const renamePinned = useCallback(
    (oldName: string, newName: string) => storeRenamePinned(platform, oldName, newName),
    [storeRenamePinned, platform],
  )

  return {
    pinned,
    recent,
    recentNotPinned,
    stableTargets,
    modifier,
    isPinned,
    canPin,
    pin,
    unpin,
    togglePin,
    recordUse,
    renamePinned,
  }
}
