import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { cancelCcrCommandJob, listConfigs, startCcrCommandJob } from '@/api'
import { useCommands } from './queries'
import {
  addFavorite as addFavoriteItem,
  addRecentItem,
  clearRecentItems,
  getFavorites,
  getRecentItems,
  removeFavorite as removeFavoriteItem,
} from '@/api/domains/uiState'
import type { CommandJobDelta, CommandJobSnapshot, ConfigItem } from '@/types'
import type { CommandHistoryDto as CommandHistoryItem } from '@/types/generated/ui_state/CommandHistoryDto'
import type { FavoriteCommandDto as FavoriteCommand } from '@/types/generated/ui_state/FavoriteCommandDto'
import { normalizeCliClient, type CliClient } from '@/types/router'
import { createAnsiRenderer } from '@/utils/ansiRenderer'
import { copyText } from '@/utils/clipboard'
import { logger } from '@/utils/logger'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import {
  CLI_CLIENTS,
  fallbackCommandRegistry,
  MAX_LEDGER_LINES,
  normalizeCommand,
  resolvedCommandName,
  splitArgs,
  type CommandCollection,
  type CommandUiInfo,
  type LedgerChannel,
} from './commands-model'
import { useCommandsT } from './locale'

export function useCommandsPage() {
  const t = useCommandsT()
  const params = useParams()
  const navigate = useNavigate()
  const runtimeUnavailable = !isTauriRuntime()
  const [selectedClient, setSelectedClient] = useState<CliClient>(() => normalizeCliClient(params.client) ?? 'ccr')
  const [commands, setCommands] = useState<CommandUiInfo[]>([])
  const [selectedCommand, setSelectedCommand] = useState('')
  const [args, setArgs] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [activeCategory, setActiveCategory] = useState('all')
  const [activeCollection, setActiveCollection] = useState<CommandCollection>('catalog')
  const [dangerAccepted, setDangerAccepted] = useState(false)
  const [currentSnapshot, setCurrentSnapshot] = useState<CommandJobSnapshot | null>(null)
  const [configs, setConfigs] = useState<ConfigItem[]>([])
  const [favorites, setFavorites] = useState<FavoriteCommand[]>([])
  const [historyItems, setHistoryItems] = useState<CommandHistoryItem[]>([])
  const lastDeltaSeq = useRef(-1)
  const preserveArgs = useRef(false)
  const recordedJobIds = useRef(new Set<string>())
  const ansiRenderer = useRef(createAnsiRenderer())

  const commandsQuery = useCommands(selectedClient)

  const applyCommandList = useCallback((client: CliClient, list = fallbackCommandRegistry[client]) => {
    const next = list.map((command) => normalizeCommand(command, client, t))
    setCommands(next)
    setSelectedCommand((current) => (next.some((item) => item.name === current) ? current : next[0]?.name ?? ''))
  }, [t])

  useEffect(() => {
    const client = normalizeCliClient(params.client)
    if (client && client !== selectedClient) setSelectedClient(client)
  }, [params.client, selectedClient])

  useEffect(() => {
    if (runtimeUnavailable || selectedClient !== 'ccr' || !commandsQuery.data) {
      applyCommandList(selectedClient)
      return
    }
    applyCommandList('ccr', commandsQuery.data.length > 0 ? commandsQuery.data : fallbackCommandRegistry.ccr)
  }, [applyCommandList, commandsQuery.data, runtimeUnavailable, selectedClient])

  useEffect(() => {
    const current = normalizeCliClient(params.client) || 'ccr'
    if (current !== selectedClient) void navigate(`/commands/${selectedClient}`, { replace: true })
  }, [navigate, params.client, selectedClient])

  useEffect(() => {
    if (runtimeUnavailable) {
      setConfigs([{ name: 'default' } as ConfigItem, { name: 'workspace' } as ConfigItem])
      setFavorites([])
      setHistoryItems([])
      return
    }
    void listConfigs().then((response) => {
      setConfigs(Array.isArray(response) ? response : response.configs)
    }).catch((error) => logger.error('Failed to load configs:', error))
    void Promise.all([getFavorites(), getRecentItems(20)]).then(([favoriteData, historyData]) => {
      setFavorites(favoriteData)
      setHistoryItems(historyData)
    }).catch((error) => logger.error('Failed to load command favorites/history:', error))
  }, [runtimeUnavailable])

  useEffect(() => {
    if (runtimeUnavailable) return
    const unlisteners: UnlistenFn[] = []
    const handleDelta = (event: Event<CommandJobDelta>) => {
      setCurrentSnapshot((snapshot) => {
        const delta = event.payload
        if (!snapshot || delta.job_id !== snapshot.job_id || delta.seq <= lastDeltaSeq.current) return snapshot
        lastDeltaSeq.current = delta.seq
        const field = delta.channel === 'stdout' ? 'stdout_lines' : delta.channel === 'stderr' ? 'stderr_lines' : 'system_lines'
        const lines = [...snapshot[field], ...delta.lines]
        return { ...snapshot, [field]: lines.slice(-500), status: delta.status ?? snapshot.status, truncated: Boolean(snapshot.truncated) || delta.dropped_count > 0, dropped_lines: (snapshot.dropped_lines ?? 0) + delta.dropped_count }
      })
    }
    const handleSnapshot = (event: Event<CommandJobSnapshot>) => {
      const payload = event.payload
      setCurrentSnapshot((snapshot) => {
        if (!snapshot || payload.job_id === snapshot.job_id) {
          lastDeltaSeq.current = -1
          return payload
        }
        return snapshot
      })
      if (recordedJobIds.current.has(payload.job_id)) return
      if (!['success', 'failed', 'cancelled', 'cleanup_failed'].includes(payload.status)) return
      recordedJobIds.current.add(payload.job_id)
      void addRecentItem(payload.command, payload.args, payload.status === 'success', payload.duration_ms ?? 0)
        .then(() => getRecentItems(20))
        .then(setHistoryItems)
        .catch((error) => logger.error('Failed to persist command history:', error))
    }
    void listen<CommandJobDelta>('commands:job-progress', handleDelta).then((fn) => unlisteners.push(fn))
    void listen<CommandJobSnapshot>('commands:job-finished', handleSnapshot).then((fn) => unlisteners.push(fn))
    void listen<CommandJobSnapshot>('commands:job-cancelled', handleSnapshot).then((fn) => unlisteners.push(fn))
    return () => {
      unlisteners.forEach((fn) => fn())
    }
  }, [runtimeUnavailable])

  const selectedCommandInfo = commands.find((command) => command.name === selectedCommand)
  const isRunning = currentSnapshot?.status === 'queued' || currentSnapshot?.status === 'running'
  const canRun = !runtimeUnavailable && selectedClient === 'ccr'
  const canEditArgs = canRun && Boolean(selectedCommandInfo?.executable) && !isRunning
  const canExecuteSelected = Boolean(canEditArgs && selectedCommandInfo && !(selectedCommandInfo.dangerous && !dangerAccepted) && !(selectedCommandInfo.requiresArgs && args.trim().length === 0))

  const filteredCommands = useMemo(() => {
    const query = searchQuery.trim().toLowerCase()
    return commands.filter((command) => {
      const matchesCategory = activeCategory === 'all' || command.category === activeCategory
      const matchesQuery = !query || command.name.toLowerCase().includes(query) || command.description.toLowerCase().includes(query)
      return matchesCategory && matchesQuery
    })
  }, [activeCategory, commands, searchQuery])

  const ledgerLines = useMemo(() => {
    if (!currentSnapshot) return []
    const build = (channel: LedgerChannel, lines: string[]) => lines.map((text, index) => ({ channel, text, index, safeHtml: ansiRenderer.current.renderLine(text) }))
    const all = [...build('system', currentSnapshot.system_lines), ...build('stdout', currentSnapshot.stdout_lines), ...build('stderr', currentSnapshot.stderr_lines)]
    return all.length > MAX_LEDGER_LINES ? all.slice(-MAX_LEDGER_LINES) : all
  }, [currentSnapshot])

  const handleExecute = useCallback(async () => {
    if (!canExecuteSelected || !selectedCommandInfo) return
    try {
      const response = await startCcrCommandJob({ command: selectedCommandInfo.name, args: splitArgs(args), confirmationToken: selectedCommandInfo.dangerous && dangerAccepted ? `desktop-confirm:${selectedCommandInfo.name}` : undefined })
      setCurrentSnapshot(response.snapshot)
      lastDeltaSeq.current = -1
    } catch (error) {
      const message = error instanceof Error ? error.message : t('commands.unknownError')
      setCurrentSnapshot({ job_id: 'local-error', command: selectedCommandInfo.name, args: splitArgs(args), status: 'failed', started_at: new Date().toISOString(), finished_at: new Date().toISOString(), duration_ms: 0, exit_code: -1, stdout_lines: [], stderr_lines: [], system_lines: [message], truncated: false, dropped_lines: 0, error: message })
    }
  }, [args, canExecuteSelected, dangerAccepted, selectedCommandInfo, t])

  const handleCancel = useCallback(async () => {
    if (!currentSnapshot) return
    try {
      setCurrentSnapshot(await cancelCcrCommandJob(currentSnapshot.job_id))
    } catch (error) {
      logger.error('Failed to cancel command job:', error)
    }
  }, [currentSnapshot])

  const handleCopyOutput = useCallback(async () => {
    const text = ledgerLines.map((line) => `[${line.channel}] ${line.text}`).join('\n')
    await copyText(text)
  }, [ledgerLines])

  const handleClearOutput = useCallback(() => {
    ansiRenderer.current.clear()
    setCurrentSnapshot(null)
    lastDeltaSeq.current = -1
  }, [])

  const loadPersistedCommand = useCallback((command: string, persistedArgs: string[]) => {
    const nextCommand = resolvedCommandName(command)
    if (!commands.some((item) => item.name === nextCommand)) return
    preserveArgs.current = true
    setSelectedCommand(nextCommand)
    setArgs(persistedArgs.join(' '))
    setActiveCollection('catalog')
    setDangerAccepted(false)
  }, [commands])

  const handleToggleFavorite = useCallback(async () => {
    if (!selectedCommandInfo) return
    const selectedArgs = splitArgs(args)
    const existing = favorites.find((item) => item.command === selectedCommand && JSON.stringify(item.args) === JSON.stringify(selectedArgs))
    try {
      if (existing) {
        await removeFavoriteItem(existing.id)
        setFavorites((current) => current.filter((item) => item.id !== existing.id))
        return
      }
      const favorite = await addFavoriteItem(selectedCommandInfo.name, selectedArgs, selectedCommandInfo.title || selectedCommandInfo.name, 'commands')
      setFavorites((current) => [favorite, ...current])
    } catch (error) {
      logger.error('Failed to toggle favorite:', error)
    }
  }, [args, favorites, selectedCommand, selectedCommandInfo])

  const handleClearHistory = useCallback(async () => {
    try {
      await clearRecentItems()
      setHistoryItems([])
    } catch (error) {
      logger.error('Failed to clear recent history:', error)
    }
  }, [])

  return {
    t,
    runtimeUnavailable,
    runtimeCopy: getRuntimeUnavailableCopy('commands'),
    selectedClient,
    setSelectedClient,
    commands,
    selectedCommand,
    setSelectedCommand,
    args,
    setArgs,
    searchQuery,
    setSearchQuery,
    activeCategory,
    setActiveCategory,
    activeCollection,
    setActiveCollection,
    dangerAccepted,
    setDangerAccepted,
    currentSnapshot,
    configs,
    favorites,
    historyItems,
    selectedCommandInfo,
    isRunning,
    canRun,
    canEditArgs,
    canExecuteSelected,
    filteredCommands,
    ledgerLines,
    ledgerTruncated: (currentSnapshot ? currentSnapshot.stdout_lines.length + currentSnapshot.stderr_lines.length + currentSnapshot.system_lines.length : 0) > MAX_LEDGER_LINES,
    handleExecute,
    handleCancel,
    handleCopyOutput,
    handleClearOutput,
    loadPersistedCommand,
    handleToggleFavorite,
    handleClearHistory,
    CLI_CLIENTS,
    MAX_LEDGER_LINES,
  }
}
