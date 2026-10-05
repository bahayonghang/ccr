import { create } from 'zustand'

// grok 缓存路由的选中态（父任务 design.md §5）。数据走 Query，选中 profile 名入本 store。

interface GrokViewState {
  selectedProfileName: string | null
  setSelectedProfileName: (name: string | null) => void
}

export const useGrokViewStore = create<GrokViewState>()((set) => ({
  selectedProfileName: null,
  setSelectedProfileName: (selectedProfileName) => set({ selectedProfileName }),
}))
