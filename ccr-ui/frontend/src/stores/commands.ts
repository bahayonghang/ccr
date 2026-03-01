import { defineStore } from 'pinia'
import { listCommands, executeCommand } from '@/api/modules/config'
import type { CommandInfo, CommandRequest, CommandResponse } from '@/types'
import { getErrorMessage } from '@/types'

interface CommandsState {
  list: CommandInfo[]
  lastFetchedAt: number
  running: boolean
  currentCommand: string | null
  lastOutput: CommandResponse | null
  error: string | null
}

export const useCommandsStore = defineStore('commands', {
  state: (): CommandsState => ({
    list: [],
    lastFetchedAt: 0,
    running: false,
    currentCommand: null,
    lastOutput: null,
    error: null
  }),

  getters: {
    /**
     * 是否有可用命令
     */
    hasCommands: (state) => state.list.length > 0,

    /**
     * 缓存是否有效（2分钟内）
     */
    isCacheValid: (state) => {
      const cacheAge = Date.now() - state.lastFetchedAt
      return cacheAge < 2 * 60 * 1000 && state.list.length > 0
    },
    
    /**
     * 按类别分组的命令
     */
    commandsByCategory: (state): Record<string, CommandInfo[]> => {
      return state.list.reduce((groups, cmd) => {
        const category = cmd.category || 'Other'
        if (!groups[category]) {
          groups[category] = []
        }
        groups[category].push(cmd)
        return groups
      }, {} as Record<string, CommandInfo[]>)
    }
  },

  actions: {
    /**
     * 加载命令列表
     * @param force 是否强制刷新（忽略缓存）
     */
    async loadList(force = false) {
      // 缓存有效且不是强制刷新，直接返回
      if (!force && this.isCacheValid) {
        return this.list
      }

      try {
        this.list = await listCommands()
        this.lastFetchedAt = Date.now()
        return this.list
      } catch (err: unknown) {
        this.error = getErrorMessage(err, '加载命令列表失败')
        throw err
      }
    },

    /**
     * 清除命令列表缓存
     */
    clearCache() {
      this.list = []
      this.lastFetchedAt = 0
      this.error = null
    },

    /**
     * 执行命令
     * @param payload 命令请求参数
     */
    async run(payload: CommandRequest): Promise<CommandResponse> {
      this.running = true
      this.currentCommand = payload.command
      this.error = null

      try {
        const result = await executeCommand(payload)
        this.lastOutput = result
        return result
      } catch (err: unknown) {
        this.error = getErrorMessage(err, '命令执行失败')
        throw err
      } finally {
        this.running = false
        this.currentCommand = null
      }
    },

    /**
     * 清除上次输出
     */
    clearOutput() {
      this.lastOutput = null
      this.error = null
    }
  }
})
