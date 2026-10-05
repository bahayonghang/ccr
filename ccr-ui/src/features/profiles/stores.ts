import { create } from 'zustand'

// Profiles 快速切换跨页共享 store（08-22-state-logic-port 批次 5c；原 Vue composable
// `useProfilesQuickSwitch.ts` 的状态承载迁移，React hook 薄封装仍在原文件）。
//
// 状态是 UI 偏好而非配置事实：钉选数组（数字编号唯一来源，≤8）+ 最近使用列表
// （只展示不编号），按平台键持久化到 localStorage，不同步到后端。
// 编号语义（P0）：数字编号 = 钉选数组顺序（1..n，n≤8），与搜索/筛选/排序/启停/Apply
// 完全解耦；recordUse 只重排最近列表，绝不影响钉选编号。
//
// 持久化偏差记录（沿用批次 4 shellPreferences 先例）：不用 zustand/persist 中间件。
// 原实现的持久化按平台分散为 ccr:profiles:pinned:<platform> /
// ccr:profiles:recent:<platform> 两族 key，本 store 逐 key 手动读写，键布局字节不变。
// 初值在模块加载时扫描两族前缀全量水合（等价原实现各实例挂载时的惰性读取，
// 且对任意平台名开放，不写死 claude/codex/grok 清单）。

/** 钉选上限：第 9 次钉选拒绝并提示，不挤掉既有钉选 */
export const PROFILES_PIN_CAP = 8

/** 最近列表持久化上限（展示时再与钉选合并截断） */
const RECENT_CAP = 16

const PINNED_KEY_PREFIX = 'ccr:profiles:pinned:'
const RECENT_KEY_PREFIX = 'ccr:profiles:recent:'
const VIEW_KEY_PREFIX = 'ccr:profiles:view:'

export type ProfilesSurfaceViewMode = 'card' | 'table'

const pinnedKeyOf = (platform: string) => `${PINNED_KEY_PREFIX}${platform}`
const recentKeyOf = (platform: string) => `${RECENT_KEY_PREFIX}${platform}`

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

/** 模块加载时的逐平台水合：扫描 localStorage 中两族前缀的全部键。 */
const readPersistedByPlatform = (): {
  pinnedByPlatform: Record<string, string[]>
  recentByPlatform: Record<string, string[]>
} => {
  const pinnedByPlatform: Record<string, string[]> = {}
  const recentByPlatform: Record<string, string[]> = {}
  if (typeof window === 'undefined') return { pinnedByPlatform, recentByPlatform }

  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i)
    if (!key) continue
    if (key.startsWith(PINNED_KEY_PREFIX)) {
      pinnedByPlatform[key.slice(PINNED_KEY_PREFIX.length)] = readNames(key).slice(0, PROFILES_PIN_CAP)
    } else if (key.startsWith(RECENT_KEY_PREFIX)) {
      recentByPlatform[key.slice(RECENT_KEY_PREFIX.length)] = readNames(key).slice(0, RECENT_CAP)
    }
  }
  return { pinnedByPlatform, recentByPlatform }
}

const persisted = readPersistedByPlatform()

interface ProfilesQuickSwitchState {
  /** 平台 → 钉选数组（顺序 = 用户钉选操作顺序），数字编号的唯一来源 */
  pinnedByPlatform: Record<string, string[]>
  /** 平台 → 最近使用列表（recordUse 时间倒序，含已钉选项） */
  recentByPlatform: Record<string, string[]>
  /** 钉选；已达上限或已钉选时返回 false（达上限会触发 onPinLimit） */
  pin: (platform: string, name: string, onPinLimit?: () => void) => boolean
  unpin: (platform: string, name: string) => void
  /** Apply 成功后记录最近使用；只影响 recent，不影响钉选编号 */
  recordUse: (platform: string, name: string) => void
  /** 重命名跟随：视图在 rename 成功后调用，钉选/最近中的旧名替换为新名 */
  renamePinned: (platform: string, oldName: string, newName: string) => void
  /**
   * stale 清理：列表加载/刷新后过滤已不存在的名称并回写；
   * profileNames 为 null（列表未就绪）时跳过，禁用不清理（仅视图置灰）
   */
  cleanupStale: (platform: string, profileNames: string[] | null) => void
}

const readViewMode = (raw: string | null): ProfilesSurfaceViewMode | null => {
  if (raw === 'card' || raw === 'table') return raw
  return null
}

const writeViewMode = (platform: string, mode: ProfilesSurfaceViewMode) => {
  try {
    localStorage.setItem(`${VIEW_KEY_PREFIX}${platform}`, mode)
  } catch {
    // storage 不可用时降级为纯内存
  }
}

const hydrateViewKey = (
  key: string | null,
  viewByPlatform: Record<string, ProfilesSurfaceViewMode>,
) => {
  if (!key?.startsWith(VIEW_KEY_PREFIX)) return
  const mode = readViewMode(localStorage.getItem(key))
  if (!mode) return
  viewByPlatform[key.slice(VIEW_KEY_PREFIX.length)] = mode
}

const readPersistedViews = (): Record<string, ProfilesSurfaceViewMode> => {
  const viewByPlatform: Record<string, ProfilesSurfaceViewMode> = {}
  if (typeof window === 'undefined') return viewByPlatform
  try {
    for (let i = 0; i < localStorage.length; i += 1) {
      hydrateViewKey(localStorage.key(i), viewByPlatform)
    }
  } catch {
    return viewByPlatform
  }
  return viewByPlatform
}

interface ProfilesViewState {
  viewByPlatform: Record<string, ProfilesSurfaceViewMode>
  setView: (platform: string, mode: ProfilesSurfaceViewMode) => void
}

export const useProfilesViewStore = create<ProfilesViewState>()((set) => ({
  viewByPlatform: readPersistedViews(),
  setView: (platform, mode) => {
    set((state) => ({
      viewByPlatform: { ...state.viewByPlatform, [platform]: mode },
    }))
    writeViewMode(platform, mode)
  },
}))

export const useProfilesQuickSwitchStore = create<ProfilesQuickSwitchState>()((set, get) => ({
  pinnedByPlatform: persisted.pinnedByPlatform,
  recentByPlatform: persisted.recentByPlatform,

  pin: (platform, name, onPinLimit) => {
    if (!name) return false
    const current = get().pinnedByPlatform[platform] ?? []
    if (current.includes(name)) return false
    if (current.length >= PROFILES_PIN_CAP) {
      onPinLimit?.()
      return false
    }
    const next = [...current, name]
    set((s) => ({ pinnedByPlatform: { ...s.pinnedByPlatform, [platform]: next } }))
    writeNames(pinnedKeyOf(platform), next)
    return true
  },

  unpin: (platform, name) => {
    const current = get().pinnedByPlatform[platform] ?? []
    if (!current.includes(name)) return
    const next = current.filter((item) => item !== name)
    set((s) => ({ pinnedByPlatform: { ...s.pinnedByPlatform, [platform]: next } }))
    writeNames(pinnedKeyOf(platform), next)
  },

  recordUse: (platform, name) => {
    if (!name) return
    const current = get().recentByPlatform[platform] ?? []
    const next = [name, ...current.filter((item) => item !== name)].slice(0, RECENT_CAP)
    set((s) => ({ recentByPlatform: { ...s.recentByPlatform, [platform]: next } }))
    writeNames(recentKeyOf(platform), next)
  },

  renamePinned: (platform, oldName, newName) => {
    if (!oldName || !newName || oldName === newName) return

    const pinned = get().pinnedByPlatform[platform] ?? []
    if (pinned.includes(oldName)) {
      const next = pinned.map((item) => (item === oldName ? newName : item))
      set((s) => ({ pinnedByPlatform: { ...s.pinnedByPlatform, [platform]: next } }))
      writeNames(pinnedKeyOf(platform), next)
    }

    const recent = get().recentByPlatform[platform] ?? []
    if (recent.includes(oldName)) {
      const next = recent.map((item) => (item === oldName ? newName : item))
      set((s) => ({ recentByPlatform: { ...s.recentByPlatform, [platform]: next } }))
      writeNames(recentKeyOf(platform), next)
    }
  },

  cleanupStale: (platform, profileNames) => {
    if (profileNames === null) return
    const valid = new Set(profileNames)

    const pinned = get().pinnedByPlatform[platform] ?? []
    const nextPinned = pinned.filter((name) => valid.has(name))
    if (nextPinned.length !== pinned.length) {
      set((s) => ({ pinnedByPlatform: { ...s.pinnedByPlatform, [platform]: nextPinned } }))
      writeNames(pinnedKeyOf(platform), nextPinned)
    }

    const recent = get().recentByPlatform[platform] ?? []
    const nextRecent = recent.filter((name) => valid.has(name))
    if (nextRecent.length !== recent.length) {
      set((s) => ({ recentByPlatform: { ...s.recentByPlatform, [platform]: nextRecent } }))
      writeNames(recentKeyOf(platform), nextRecent)
    }
  },
}))
