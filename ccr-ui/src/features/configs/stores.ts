import { create } from 'zustand'

// configs 视图状态 store（08-22-state-logic-port 批次 4）。
// 原 Pinia `stores/configs.ts` 的数据部分已迁 Query（本目录 queries.ts）；此处承载
// 父任务 design.md §5 的 `configs` 缓存路由视图态：选中态 + 搜索词 + 表单草稿
// （草稿键为配置 id，切回路由时未提交表单可恢复——外壳门 AC4 的六项状态之一）。

interface ConfigsViewState {
  currentConfig: string | null
  searchQuery: string
  /** 表单草稿：配置 id → 草稿内容（JSON 字符串或任意可序列化值，形态归消费视图）。 */
  formDrafts: Record<string, unknown>
  setCurrentConfig: (configName: string | null) => void
  setSearchQuery: (query: string) => void
  setFormDraft: (configId: string, draft: unknown) => void
  clearFormDraft: (configId: string) => void
}

export const useConfigsViewStore = create<ConfigsViewState>()((set) => ({
  currentConfig: null,
  searchQuery: '',
  formDrafts: {},

  setCurrentConfig: (currentConfig) => set({ currentConfig }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setFormDraft: (configId, draft) =>
    set((state) => ({ formDrafts: { ...state.formDrafts, [configId]: draft } })),
  clearFormDraft: (configId) =>
    set((state) => {
      if (!(configId in state.formDrafts)) return state
      const formDrafts = { ...state.formDrafts }
      delete formDrafts[configId]
      return { ...state, formDrafts }
    }),
}))
