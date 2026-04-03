// CCR Control Composable - 状态管理和命令执行

import { ref, computed, onMounted } from 'vue'
import {
  getFavorites,
  addFavorite,
  removeFavorite,
  getHistory,
  addHistory,
  clearHistory as clearHistoryApi,
  CCR_MODULES,
  type FavoriteCommand,
  type CommandHistory,
  type CcrModule,
  type CcrCommand,
  type AddFavoriteRequest,
} from '@/api/ccr-control'
import { getVersion, checkUpdate, executeCommand as executeCcrCommand } from '@/api'
import type { VersionInfo, UpdateCheckResponse } from '@/types'
import { logger } from '@/utils/logger'

export function useCcrControl() {
  // ===== 状态 =====

  // 版本信息
  const versionInfo = ref<VersionInfo | null>(null)
  const updateInfo = ref<UpdateCheckResponse | null>(null)
  const loadingVersion = ref(false)

  // 模块和命令
  const modules = ref<CcrModule[]>(CCR_MODULES)
  const selectedModuleId = ref<string>('config')
  const selectedCommand = ref<CcrCommand | null>(null)

  // 收藏和历史
  const favorites = ref<FavoriteCommand[]>([])
  const history = ref<CommandHistory[]>([])
  const loadingFavorites = ref(false)
  const loadingHistory = ref(false)

  // 命令执行
  const isExecuting = ref(false)
  const outputLines = ref<string[]>([])
  const lastExitCode = ref<number | null>(null)
  const abortController = ref<AbortController | null>(null)

  const MAX_OUTPUT_LINES = 2000
  const TRUNCATION_NOTICE = `… output truncated (keeping last ${MAX_OUTPUT_LINES} lines)`

  const appendOutputLines = (lines: string[]) => {
    if (lines.length === 0) return

    const base = outputLines.value[0] === TRUNCATION_NOTICE
      ? outputLines.value.slice(1)
      : outputLines.value
    const merged = [...base, ...lines]

    if (merged.length <= MAX_OUTPUT_LINES) {
      outputLines.value = merged
      return
    }

    outputLines.value = [
      TRUNCATION_NOTICE,
      ...merged.slice(-(MAX_OUTPUT_LINES - 1)),
    ]
  }

  // 命令参数
  const commandArgs = ref<Record<string, string>>({})
  const commandFlags = ref<Record<string, unknown>>({})

  // ===== 计算属性 =====

  const selectedModule = computed(() => modules.value.find((m) => m.id === selectedModuleId.value))

  const currentCommands = computed(() => selectedModule.value?.commands || [])

  // ===== 版本管理 =====

  const loadVersionInfo = async () => {
    loadingVersion.value = true
    try {
      versionInfo.value = await getVersion()
    } catch (err) {
      logger.error('Failed to load version info', err)
    } finally {
      loadingVersion.value = false
    }
  }

  const checkForUpdate = async () => {
    loadingVersion.value = true
    try {
      updateInfo.value = await checkUpdate()
    } catch (err) {
      logger.error('Failed to check update', err)
    } finally {
      loadingVersion.value = false
    }
  }

  // ===== 收藏管理 =====

  const loadFavorites = async () => {
    loadingFavorites.value = true
    try {
      favorites.value = await getFavorites()
    } catch (err) {
      logger.error('Failed to load favorites', err)
    } finally {
      loadingFavorites.value = false
    }
  }

  const addToFavorites = async (command: CcrCommand, args: string[] = []) => {
    const req: AddFavoriteRequest = {
      command: command.command,
      args,
      display_name: command.name,
      module: selectedModuleId.value,
    }
    try {
      const fav = await addFavorite(req)
      favorites.value.push(fav)
      return true
    } catch (err) {
      logger.error('Failed to add favorite', err)
      return false
    }
  }

  const removeFromFavorites = async (id: string) => {
    try {
      await removeFavorite(id)
      favorites.value = favorites.value.filter((f) => f.id !== id)
      return true
    } catch (err) {
      logger.error('Failed to remove favorite', err)
      return false
    }
  }

  const isFavorite = (command: string) => {
    return favorites.value.some((f) => f.command === command)
  }

  // ===== 历史管理 =====

  const loadHistory = async () => {
    loadingHistory.value = true
    try {
      history.value = await getHistory(50)
    } catch (err) {
      logger.error('Failed to load history', err)
    } finally {
      loadingHistory.value = false
    }
  }

  const clearHistory = async () => {
    try {
      await clearHistoryApi()
      history.value = []
      return true
    } catch (err) {
      logger.error('Failed to clear history', err)
      return false
    }
  }

  // ===== 命令执行 =====

  const buildCommandArgs = (command: CcrCommand): string[] => {
    const args: string[] = []

    // 添加位置参数
    if (command.args) {
      for (const arg of command.args) {
        const value = commandArgs.value[arg.name]
        if (value) {
          args.push(value)
        } else if (arg.required) {
          throw new Error(`缺少必填参数: ${arg.name}`)
        }
      }
    }

    // 添加标志参数
    if (command.flags) {
      for (const flag of command.flags) {
        const value = commandFlags.value[flag.name]
        if (value !== undefined && value !== false && value !== '') {
          if (flag.type === 'boolean' && value === true) {
            args.push(flag.flag)
          } else if (flag.type !== 'boolean') {
            args.push(flag.flag, String(value))
          }
        }
      }
    }

    return args
  }

  const executeCommand = async (command: CcrCommand) => {
    if (isExecuting.value) return

    isExecuting.value = true
    outputLines.value = []
    lastExitCode.value = null

    // 创建新的 AbortController
    abortController.value = new AbortController()

    const startTime = Date.now()

    try {
      // 构建命令参数
      const args = buildCommandArgs(command)

      // 构建完整命令
      const fullCommand = command.command
      const fullArgs = fullCommand.includes(' ')
        ? [...fullCommand.split(' ').slice(1), ...args]
        : args
      const mainCommand = fullCommand.split(' ')[0]

      appendOutputLines([
        `$ ccr ${fullCommand}${args.length ? ' ' + args.join(' ') : ''}`,
        '',
      ])

      logger.debug('[CCR] Executing command', { command: mainCommand, args: fullArgs })

      // 使用 Tauri 命令执行 API（非流式）
      const response = await executeCcrCommand({
        command: mainCommand,
        args: fullArgs,
      }) as {
        success?: boolean
        stdout?: string
        stderr?: string
        exit_code?: number
      }

      const stdoutLines = String(response?.stdout ?? '')
        .split('\n')
        .map(line => line.trimEnd())
        .filter(line => line.length > 0)
      const stderrLines = String(response?.stderr ?? '')
        .split('\n')
        .map(line => line.trimEnd())
        .filter(line => line.length > 0)

      if (stdoutLines.length > 0) {
        appendOutputLines(stdoutLines)
      }
      if (stderrLines.length > 0) {
        appendOutputLines(stderrLines.map(line => `[stderr] ${line}`))
      }

      lastExitCode.value = response?.exit_code ?? (response?.success ? 0 : 1)

      const duration = Date.now() - startTime
      const success = lastExitCode.value === 0

      appendOutputLines([
        '',
        success
          ? `✅ 命令执行成功 (${duration}ms)`
          : `❌ 命令执行失败，退出码: ${lastExitCode.value}`,
      ])

      // 记录历史
      await addHistory({
        command: fullCommand,
        args,
        success,
        duration_ms: duration,
      })

      // 刷新历史列表
      await loadHistory()
    } catch (err: unknown) {
      const duration = Date.now() - startTime

      // 检查是否是用户取消
      if (err instanceof Error && err.name === 'AbortError') {
        appendOutputLines(['', '⏹️ 命令已取消'])
        lastExitCode.value = 1
      } else {
        const errorMessage = err instanceof Error ? err.message : '未知错误'
        appendOutputLines([`❌ 执行错误: ${errorMessage}`])
        lastExitCode.value = 1

        // 记录失败历史
        await addHistory({
          command: command.command,
          args: [],
          success: false,
          duration_ms: duration,
        })

        await loadHistory()
      }
    } finally {
      isExecuting.value = false
      abortController.value = null
    }
  }

  const cancelCommand = () => {
    if (abortController.value) {
      abortController.value.abort()
      appendOutputLines(['⏹️ 正在取消命令...'])
    }
  }

  const executeFromFavorite = async (favorite: FavoriteCommand) => {
    // 找到对应的命令定义
    const module = modules.value.find((m) => m.id === favorite.module)
    const command = module?.commands.find((c) => c.command === favorite.command)

    if (command) {
      // 设置参数
      if (command.args && favorite.args.length > 0) {
        command.args.forEach((arg, index) => {
          if (favorite.args[index]) {
            commandArgs.value[arg.name] = favorite.args[index]
          }
        })
      }

      await executeCommand(command)
    }
  }

  const executeFromHistory = async (historyItem: CommandHistory) => {
    // 找到对应的命令定义
    for (const module of modules.value) {
      const command = module.commands.find((c) => c.command === historyItem.command)
      if (command) {
        // 设置参数
        if (command.args && historyItem.args.length > 0) {
          command.args.forEach((arg, index) => {
            if (historyItem.args[index]) {
              commandArgs.value[arg.name] = historyItem.args[index]
            }
          })
        }

        await executeCommand(command)
        return
      }
    }
  }

  // ===== 命令选择 =====

  const selectModule = (moduleId: string) => {
    selectedModuleId.value = moduleId
    selectedCommand.value = null
    commandArgs.value = {}
    commandFlags.value = {}
  }

  const selectCommand = (command: CcrCommand) => {
    selectedCommand.value = command
    commandArgs.value = {}
    commandFlags.value = {}

    // 设置默认值
    if (command.flags) {
      for (const flag of command.flags) {
        if (flag.default !== undefined) {
          commandFlags.value[flag.name] = flag.default
        }
      }
    }
  }

  const clearOutput = () => {
    outputLines.value = []
    lastExitCode.value = null
  }

  // ===== 生命周期 =====

  onMounted(async () => {
    await Promise.all([loadVersionInfo(), loadFavorites(), loadHistory()])
  })

  return {
    // 版本
    versionInfo,
    updateInfo,
    loadingVersion,
    loadVersionInfo,
    checkForUpdate,

    // 模块和命令
    modules,
    selectedModuleId,
    selectedModule,
    currentCommands,
    selectedCommand,
    selectModule,
    selectCommand,

    // 收藏
    favorites,
    loadingFavorites,
    loadFavorites,
    addToFavorites,
    removeFromFavorites,
    isFavorite,

    // 历史
    history,
    loadingHistory,
    loadHistory,
    clearHistory,

    // 命令执行
    isExecuting,
    outputLines,
    lastExitCode,
    commandArgs,
    commandFlags,
    buildCommandArgs,
    executeCommand,
    cancelCommand,
    executeFromFavorite,
    executeFromHistory,
    clearOutput,
  }
}
