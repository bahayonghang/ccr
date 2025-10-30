import { defineStore } from 'pinia'
import { listCommands, executeCommand } from '@/api/client'
import type { CommandInfo, CommandRequest, CommandResponse } from '@/types'

interface CommandsState {
  list: CommandInfo[]
  running: boolean
  currentCommand: string | null
  lastOutput: CommandResponse | null
  error: string | null
}

export const useCommandsStore = defineStore('commands', {
  state: (): CommandsState => ({
    list: [],
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
     * 按类别分组的命令
     */
    commandsByCategory: (state) => {
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
     */
    async loadList() {
      try {
        this.list = await listCommands()
      } catch (err: any) {
        this.error = err.message || '加载命令列表失败'
        throw err
      }
    },

    /**
     * 执行命令
     * @param payload 命令请求参数
     */
    async run(payload: CommandRequest): Promise<CommandResponse> {
      this.running = true
      this.currentCommand = payload.name
      this.error = null

      try {
        const result = await executeCommand(payload)
        this.lastOutput = result
        return result
      } catch (err: any) {
        this.error = err.message || '命令执行失败'
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
