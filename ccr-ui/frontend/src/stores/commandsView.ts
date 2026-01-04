import { defineStore } from 'pinia'

type SortKey = 'name' | 'usage' | 'modified'
type SortDir = 'asc' | 'desc'
type ViewMode = 'flat' | 'tree'

interface CommandsViewState {
  sortKey: SortKey
  sortDir: SortDir
  viewMode: ViewMode
  showDeprecated: boolean
  expandedFolders: string[]
}

export const useCommandsViewStore = defineStore('commandsView', {
  state: (): CommandsViewState => ({
    sortKey: 'name',
    sortDir: 'asc',
    viewMode: 'tree',
    showDeprecated: true,
    expandedFolders: [],
  }),

  actions: {
    setSortKey(key: SortKey) {
      this.sortKey = key
      this.persist()
    },

    toggleSortDir() {
      this.sortDir = this.sortDir === 'asc' ? 'desc' : 'asc'
      this.persist()
    },

    setViewMode(mode: ViewMode) {
      this.viewMode = mode
      this.persist()
    },

    toggleShowDeprecated() {
      this.showDeprecated = !this.showDeprecated
      this.persist()
    },

    toggleFolder(folder: string) {
      const index = this.expandedFolders.indexOf(folder)
      if (index > -1) {
        this.expandedFolders.splice(index, 1)
      } else {
        this.expandedFolders.push(folder)
      }
      this.persist()
    },

    persist() {
      localStorage.setItem('ccr-commands-view', JSON.stringify(this.$state))
    },

    restore() {
      const saved = localStorage.getItem('ccr-commands-view')
      if (saved) {
        try {
          const state = JSON.parse(saved)
          this.$patch(state)
        } catch (e) {
          console.error('Failed to restore commands view state:', e)
        }
      }
    },
  },
})
