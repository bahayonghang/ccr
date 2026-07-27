import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { executeCommand, listCommands } from '@/api'
import { useCachedFetch } from '@/composables/useCachedFetch'
import { getErrorMessage } from '@/types'
import type { CommandInfo, CommandRequest, CommandResponse } from '@/types'

export const useCommandsStore = defineStore('commands', () => {
  const commandsCache = useCachedFetch<CommandInfo[]>({
    ttlMs: 2 * 60 * 1000,
    initialValue: [],
    isEmpty: (value) => value.length === 0,
  })

  const running = ref(false)
  const currentCommand = ref<string | null>(null)
  const lastOutput = ref<CommandResponse | null>(null)
  const error = ref<string | null>(null)

  const hasCommands = computed(() => commandsCache.data.value.length > 0)
  const commandsByCategory = computed<Record<string, CommandInfo[]>>(() => {
    return commandsCache.data.value.reduce((groups, cmd) => {
      const category = cmd.category || 'Other'
      if (!groups[category]) {
        groups[category] = []
      }
      groups[category].push(cmd)
      return groups
    }, {} as Record<string, CommandInfo[]>)
  })

  async function loadList(force = false) {
    try {
      error.value = null
      return await commandsCache.fetch(() => listCommands(), force)
    } catch (err: unknown) {
      error.value = getErrorMessage(err, '加载命令列表失败')
      throw err
    }
  }

  function clearCache() {
    commandsCache.data.value = []
    commandsCache.invalidate()
    error.value = null
  }

  async function run(payload: CommandRequest): Promise<CommandResponse> {
    running.value = true
    currentCommand.value = payload.command
    error.value = null

    try {
      const result = await executeCommand(payload)
      lastOutput.value = result
      return result
    } catch (err: unknown) {
      error.value = getErrorMessage(err, '命令执行失败')
      throw err
    } finally {
      running.value = false
      currentCommand.value = null
    }
  }

  function clearOutput() {
    lastOutput.value = null
    error.value = null
  }

  return {
    list: commandsCache.data,
    lastFetchedAt: commandsCache.lastFetchedAt,
    running,
    currentCommand,
    lastOutput,
    error,
    hasCommands,
    isCacheValid: commandsCache.isCacheValid,
    commandsByCategory,
    loadList,
    clearCache,
    run,
    clearOutput,
  }
})
