import { create } from 'zustand'
import { logger } from '@/utils/logger'

// commandsView store（08-22-state-logic-port 批次 4；原 Pinia `stores/commandsView.ts`
// 的 Options-API 语义等价迁移）。localStorage 持久化键 `ccr-commands-view` 不变，
// 持久化形状为完整 state 的 JSON（与原 `persist()` 一致），不走 zustand/persist
// （保持键与形状逐字节兼容，理由同 shellPreferences 的偏差记录）。

export type SortKey = 'name' | 'usage' | 'modified'
export type SortDir = 'asc' | 'desc'
export type ViewMode = 'flat' | 'tree'

export interface CommandsViewState {
  sortKey: SortKey
  sortDir: SortDir
  viewMode: ViewMode
  showDeprecated: boolean
  expandedFolders: string[]
  setSortKey: (key: SortKey) => void
  toggleSortDir: () => void
  setViewMode: (mode: ViewMode) => void
  toggleShowDeprecated: () => void
  toggleFolder: (folder: string) => void
  restore: () => void
}

const STORAGE_KEY = 'ccr-commands-view'

const persist = (state: CommandsViewState): void => {
  const { sortKey, sortDir, viewMode, showDeprecated, expandedFolders } = state
  localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify({ sortKey, sortDir, viewMode, showDeprecated, expandedFolders }),
  )
}

export type CommandStreamChannel = 'stdout' | 'stderr' | 'system'

export interface CommandStreamLine {
  channel: CommandStreamChannel
  text: string
  seq: number
  jobId: string
}

/** 账本上限：与 CommandsView.vue `MAX_LEDGER_LINES` 对齐，超出丢弃最旧行。 */
export const COMMANDS_STREAM_CAP = 2000

interface CommandsStreamState {
  activeClient: string
  linesByClient: Record<string, CommandStreamLine[]>
  setActiveClient: (client: string) => void
  appendStreamLines: (input: { client?: string; lines: CommandStreamLine[] }) => void
  clearStream: (client: string) => void
}

const truncateLines = (lines: CommandStreamLine[]): CommandStreamLine[] => {
  if (lines.length <= COMMANDS_STREAM_CAP) return lines
  return lines.slice(-COMMANDS_STREAM_CAP)
}

export const useCommandsStreamStore = create<CommandsStreamState>()((set, get) => ({
  activeClient: 'ccr',
  linesByClient: {},

  setActiveClient: (activeClient) => set({ activeClient }),

  appendStreamLines: (input) => {
    const client = input.client ?? get().activeClient
    if (input.lines.length === 0) return
    set((state) => {
      const current = state.linesByClient[client] ?? []
      return {
        linesByClient: {
          ...state.linesByClient,
          [client]: truncateLines([...current, ...input.lines]),
        },
      }
    })
  },

  clearStream: (client) => {
    set((state) => {
      if (!(client in state.linesByClient)) return state
      const linesByClient = { ...state.linesByClient }
      delete linesByClient[client]
      return { ...state, linesByClient }
    })
  },
}))

export const useCommandsViewStore = create<CommandsViewState>()((set, get) => ({
  sortKey: 'name',
  sortDir: 'asc',
  viewMode: 'tree',
  showDeprecated: true,
  expandedFolders: [],

  setSortKey: (key) => {
    set({ sortKey: key })
    persist(get())
  },

  toggleSortDir: () => {
    set((state) => ({ sortDir: state.sortDir === 'asc' ? 'desc' : 'asc' }))
    persist(get())
  },

  setViewMode: (mode) => {
    set({ viewMode: mode })
    persist(get())
  },

  toggleShowDeprecated: () => {
    set((state) => ({ showDeprecated: !state.showDeprecated }))
    persist(get())
  },

  toggleFolder: (folder) => {
    set((state) => ({
      expandedFolders: state.expandedFolders.includes(folder)
        ? state.expandedFolders.filter((item) => item !== folder)
        : [...state.expandedFolders, folder],
    }))
    persist(get())
  },

  restore: () => {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (!saved) return

    try {
      const state: unknown = JSON.parse(saved)
      // 本地持久化的旧视图状态，形状未知；按 Partial 收敛后合并。
      const partial = state as Partial<CommandsViewState>
      set({
        ...(partial.sortKey !== undefined && { sortKey: partial.sortKey }),
        ...(partial.sortDir !== undefined && { sortDir: partial.sortDir }),
        ...(partial.viewMode !== undefined && { viewMode: partial.viewMode }),
        ...(partial.showDeprecated !== undefined && { showDeprecated: partial.showDeprecated }),
        ...(Array.isArray(partial.expandedFolders) && { expandedFolders: partial.expandedFolders }),
      })
    } catch (error) {
      logger.error('Failed to restore commands view state:', error)
    }
  },
}))
