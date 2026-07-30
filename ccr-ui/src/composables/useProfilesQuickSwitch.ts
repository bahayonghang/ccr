// Profiles 快速切换状态：钉选数组（数字编号唯一来源，≤8）+ 最近使用列表（只展示不编号）。
// 状态是 UI 偏好而非配置事实，按平台键持久化到 localStorage，不同步到后端。
// 编号语义（P0）：数字编号 = 钉选数组顺序（1..n，n≤8），与搜索/筛选/排序/启停/Apply 完全解耦；
// recordUse 只重排最近列表，绝不影响钉选编号。
import { computed, ref, watch, type ComputedRef, type Ref } from 'vue'
import { getClientPlatform } from '@/utils/windowChrome'

/** 钉选上限：第 9 次钉选拒绝并提示，不挤掉既有钉选 */
export const PROFILES_PIN_CAP = 8

/** 最近列表持久化上限（展示时再与钉选合并截断） */
const RECENT_CAP = 16

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
  pinned: Ref<string[]>
  /** 最近使用列表（recordUse 时间倒序，含已钉选项） */
  recent: Ref<string[]>
  /** 最近列表中未钉选的部分：只展示，永不编号 */
  recentNotPinned: ComputedRef<string[]>
  /** ⌘/Ctrl+数字键的稳定目标数组（= 钉选数组，最多 8 个） */
  stableTargets: ComputedRef<string[]>
  /** 平台修饰键展示文案：macos → '⌘'，其余 → 'Ctrl' */
  modifier: ComputedRef<'Ctrl' | '⌘'>
  isPinned: (name: string) => boolean
  /** 是否还能继续钉选（未达上限） */
  canPin: ComputedRef<boolean>
  /** 钉选；已达上限或已钉选时返回 false（达上限会触发 onPinLimit） */
  pin: (name: string) => boolean
  unpin: (name: string) => void
  togglePin: (name: string) => void
  /** Apply 成功后记录最近使用；只影响 recent，不影响钉选编号 */
  recordUse: (name: string) => void
  /** 重命名跟随：视图在 rename 成功后调用，钉选/最近中的旧名替换为新名 */
  renamePinned: (oldName: string, newName: string) => void
}

const readNames = (key: string): string[] => {
  try {
    const raw = localStorage.getItem(key)
    if (!raw) return []
    const parsed: unknown = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed.filter((item): item is string => typeof item === 'string' && item.length > 0)
  } catch {
    return []
  }
}

const writeNames = (key: string, names: string[]) => {
  try {
    localStorage.setItem(key, JSON.stringify(names))
  } catch {
    // localStorage 不可用（隐私模式等）时降级为纯内存状态
  }
}

export function useProfilesQuickSwitch(options: UseProfilesQuickSwitchOptions): ProfilesQuickSwitch {
  const pinnedKey = `ccr:profiles:pinned:${options.platform}`
  const recentKey = `ccr:profiles:recent:${options.platform}`

  const pinned: Ref<string[]> = ref(readNames(pinnedKey).slice(0, PROFILES_PIN_CAP))
  const recent: Ref<string[]> = ref(readNames(recentKey).slice(0, RECENT_CAP))

  const persistPinned = () => writeNames(pinnedKey, pinned.value)
  const persistRecent = () => writeNames(recentKey, recent.value)

  // stale 清理：列表加载/刷新后过滤已不存在的名称并回写；禁用不清理（仅视图置灰）
  const cleanupStale = (profileNames: string[] | null) => {
    if (profileNames === null) return
    const valid = new Set(profileNames)
    const nextPinned = pinned.value.filter(name => valid.has(name))
    if (nextPinned.length !== pinned.value.length) {
      pinned.value = nextPinned
      persistPinned()
    }
    const nextRecent = recent.value.filter(name => valid.has(name))
    if (nextRecent.length !== recent.value.length) {
      recent.value = nextRecent
      persistRecent()
    }
  }

  watch(
    () => options.getProfileNames(),
    cleanupStale,
    { immediate: true, flush: 'sync' },
  )

  const recentNotPinned = computed(() =>
    recent.value.filter(name => !pinned.value.includes(name)),
  )

  const stableTargets = computed(() => pinned.value.slice(0, PROFILES_PIN_CAP))

  const modifier = computed<'Ctrl' | '⌘'>(() =>
    getClientPlatform() === 'macos' ? '⌘' : 'Ctrl',
  )

  const isPinned = (name: string) => pinned.value.includes(name)

  const canPin = computed(() => pinned.value.length < PROFILES_PIN_CAP)

  const pin = (name: string): boolean => {
    if (!name || isPinned(name)) return false
    if (!canPin.value) {
      options.onPinLimit?.()
      return false
    }
    pinned.value = [...pinned.value, name]
    persistPinned()
    return true
  }

  const unpin = (name: string) => {
    if (!isPinned(name)) return
    pinned.value = pinned.value.filter(item => item !== name)
    persistPinned()
  }

  const togglePin = (name: string) => {
    if (isPinned(name)) unpin(name)
    else pin(name)
  }

  const recordUse = (name: string) => {
    if (!name) return
    const next = [name, ...recent.value.filter(item => item !== name)]
    recent.value = next.slice(0, RECENT_CAP)
    persistRecent()
  }

  const renamePinned = (oldName: string, newName: string) => {
    if (!oldName || !newName || oldName === newName) return
    if (pinned.value.includes(oldName)) {
      pinned.value = pinned.value.map(item => (item === oldName ? newName : item))
      persistPinned()
    }
    if (recent.value.includes(oldName)) {
      recent.value = recent.value.map(item => (item === oldName ? newName : item))
      persistRecent()
    }
  }

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
