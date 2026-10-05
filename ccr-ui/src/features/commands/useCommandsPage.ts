import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { listConfigs } from '@/api'
import { useCommands } from './queries'
import {
  addFavorite as addFavoriteItem,
  getFavorites,
  removeFavorite as removeFavoriteItem,
} from '@/api/domains/uiState'
import type { ConfigItem } from '@/types'
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
import { useCommandsStreamStore } from './stores'
import { isCommandActive } from './commandJobState'

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
  const job = useCommandsStreamStore((state) => state.job)
  const currentSnapshot = job.snapshot
  const submitting = useCommandsStreamStore((state) => state.submitting)
  const cancelling = useCommandsStreamStore((state) => state.cancelling)
  const jobError = useCommandsStreamStore((state) => state.error)
  const listenerFailures = useCommandsStreamStore((state) => state.listenerFailures)
  const historyWrites = useCommandsStreamStore((state) => state.historyWrites)
  const historyItems = useCommandsStreamStore((state) => state.historyItems)
  const handleCancel = useCommandsStreamStore((state) => state.cancel)
  const handleRefreshJob = useCommandsStreamStore((state) => state.reconcile)
  const handleClearHistory = useCommandsStreamStore((state) => state.clearHistory)
  const handleRefreshHistory = useCommandsStreamStore((state) => state.loadHistory)
  const [configs, setConfigs] = useState<ConfigItem[]>([])
  const [favorites, setFavorites] = useState<FavoriteCommand[]>([])
  const preserveArgs = useRef(false)
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
      return
    }
    void listConfigs('claude').then((response) => {
      setConfigs(Array.isArray(response) ? response : response.configs)
    }).catch((error) => logger.error('Failed to load configs:', error))
    void getFavorites().then(setFavorites).catch((error) => logger.error('Failed to load command favorites:', error))
    void useCommandsStreamStore.getState().loadHistory()
    void useCommandsStreamStore.getState().reconcile()
  }, [runtimeUnavailable])

  const selectedCommandInfo = commands.find((command) => command.name === selectedCommand)
  const isRunning = isCommandActive(currentSnapshot) && !job.expired
  const canRun = !runtimeUnavailable && selectedClient === 'ccr'
  const canEditArgs = canRun && Boolean(selectedCommandInfo?.executable) && !isRunning && !submitting
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
    await useCommandsStreamStore.getState().start({
      command: selectedCommandInfo.name, args: splitArgs(args),
      confirmationToken: selectedCommandInfo.dangerous && dangerAccepted ? `desktop-confirm:${selectedCommandInfo.name}` : undefined,
    })
  }, [args, canExecuteSelected, dangerAccepted, selectedCommandInfo])

  const handleCopyOutput = useCallback(async () => {
    const text = ledgerLines.map((line) => `[${line.channel}] ${line.text}`).join('\n')
    await copyText(text)
  }, [ledgerLines])

  const handleClearOutput = useCallback(() => {
    ansiRenderer.current.clear()
    useCommandsStreamStore.getState().clearOutput()
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
    submitting,
    cancelling,
    jobError,
    liveUpdatesUnavailable: Object.keys(listenerFailures).length > 0,
    historyWrite: currentSnapshot ? historyWrites[currentSnapshot.job_id] : undefined,
    failedHistoryWrites: Object.values(historyWrites).filter((write) => write.status === 'failed'),
    jobExpired: job.expired,
    handleRefreshJob,
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
    handleRefreshHistory,
    CLI_CLIENTS,
    MAX_LEDGER_LINES,
  }
}
