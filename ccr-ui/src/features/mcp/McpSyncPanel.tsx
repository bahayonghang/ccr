import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import { listSourceMcpServers, syncAllMcpServers, syncMcpServer } from '@/api'
import type { McpServerInfo } from '@/types/api'
import type { SyncResult } from '@/types/sync'
import { logger } from '@/utils/logger'
import { SIcon, cn } from '@/ui'
import { useMcpT } from './locale'
import { mcpNotify } from './notify'

const PLATFORMS = [
  { id: 'codex', name: 'Codex', icon: 'Terminal' },
  { id: 'gemini', name: 'Antigravity CLI', icon: 'Sparkles' },
]

interface McpSyncPanelProps {
  onSynced: () => void
}

const ServerCard = memo(function ServerCard({
  server,
  sourcePlatformName,
  results,
  syncing,
  onSync,
  t,
}: {
  server: McpServerInfo
  sourcePlatformName: string
  results?: SyncResult[]
  syncing: boolean
  onSync: (name: string) => void
  t: (key: string) => string
}) {
  const handleSync = useCallback(() => {
    onSync(server.name)
  }, [onSync, server.name])
  return (
    <div className="rounded-2xl border border-border-default/50 bg-bg-elevated p-4">
      <div className="flex items-center justify-between">
        <div className="min-w-0 flex-1">
          <div className="mb-1 flex items-center gap-2">
            <h4 className="truncate text-sm font-bold text-text-primary">{server.name}</h4>
            <span className="rounded-full bg-accent-primary/10 px-2 py-0.5 text-[0.625rem] font-medium text-accent-primary">{sourcePlatformName}</span>
          </div>
          <div className="flex items-center gap-1.5 overflow-hidden rounded-lg bg-bg-surface px-2 py-1 font-mono text-xs text-text-muted">
            <SIcon name="Terminal" size="w-3 h-3" />
            <span className="truncate">{server.command} {server.args.join(' ')}</span>
          </div>
        </div>
        <button type="button" className="ml-4 inline-flex min-h-11 items-center gap-1.5 rounded-xl bg-accent-success/10 px-3 py-2 text-xs font-medium text-accent-success" disabled={syncing} onClick={handleSync}>
          <SIcon name="RefreshCw" size="w-3.5 h-3.5" className={syncing ? 'animate-spin' : ''} />
          {t('mcp.sync.sync')}
        </button>
      </div>
      {results ? (
        <div className="mt-3 flex flex-wrap gap-2 border-t border-border-default/30 pt-3">
          {results.map((result) => (
            <span key={result.platform} className={cn('inline-flex items-center gap-1 rounded-lg px-2 py-1 text-[0.625rem] font-medium', result.success ? 'bg-accent-success/10 text-accent-success' : 'bg-accent-danger/10 text-accent-danger')}>
              <SIcon name={result.success ? 'Check' : 'X'} size="w-3 h-3" />
              {result.platform}
            </span>
          ))}
        </div>
      ) : null}
    </div>
  )
})

export function McpSyncPanel({ onSynced }: McpSyncPanelProps) {
  const t = useMcpT()
  const [loading, setLoading] = useState(false)
  const [syncing, setSyncing] = useState(false)
  const [syncingServer, setSyncingServer] = useState<string | null>(null)
  const [sourceServers, setSourceServers] = useState<McpServerInfo[]>([])
  const [selectedPlatforms, setSelectedPlatforms] = useState<string[]>(['codex', 'gemini'])
  const [syncResults, setSyncResults] = useState<Record<string, SyncResult[]>>({})
  const sourcePlatformName = 'Claude'
  const sourceServersLabel = useMemo(() => `${t('mcp.sync.sourceServers')} (${sourcePlatformName})`, [t])

  const loadSourceServers = useCallback(async () => {
    try {
      setLoading(true)
      const next = await listSourceMcpServers<McpServerInfo[]>()
      setSourceServers(Array.isArray(next) ? next : [])
    } catch (err) {
      logger.error('Failed to load source MCP servers:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void loadSourceServers()
  }, [loadSourceServers])

  const togglePlatform = useCallback((id: string) => {
    setSelectedPlatforms((current) => (current.includes(id) ? current.filter((item) => item !== id) : [...current, id]))
  }, [])

  const handleSyncServer = useCallback(async (serverName: string) => {
    if (selectedPlatforms.length === 0) {
      mcpNotify.warning(t('mcp.sync.selectPlatformFirst'))
      return
    }
    try {
      setSyncingServer(serverName)
      const response = await syncMcpServer<{ results: SyncResult[] }>(serverName, selectedPlatforms)
      setSyncResults((current) => ({ ...current, [serverName]: response.results }))
      onSynced()
    } catch (err) {
      logger.error('Failed to sync server:', err)
      mcpNotify.error(`${t('mcp.sync.syncFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setSyncingServer(null)
    }
  }, [onSynced, selectedPlatforms, t])

  const handleSyncAll = useCallback(async () => {
    if (selectedPlatforms.length === 0) {
      mcpNotify.warning(t('mcp.sync.selectPlatformFirst'))
      return
    }
    try {
      setSyncing(true)
      const response = await syncAllMcpServers<{ servers: Record<string, SyncResult[]> }>(selectedPlatforms)
      setSyncResults((current) => ({ ...current, ...response.servers }))
      onSynced()
      mcpNotify.success(t('mcp.sync.syncAllSuccess'))
    } catch (err) {
      logger.error('Failed to sync all servers:', err)
      mcpNotify.error(`${t('mcp.sync.syncFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`)
    } finally {
      setSyncing(false)
    }
  }, [onSynced, selectedPlatforms, t])

  const onRefresh = useCallback(() => {
    void loadSourceServers()
  }, [loadSourceServers])
  const onSyncAll = useCallback(() => {
    void handleSyncAll()
  }, [handleSyncAll])

  return (
    <div className="rounded-3xl border border-border-default/25 bg-bg-elevated p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h2 className="text-lg font-bold text-text-primary">{t('mcp.sync.title')}</h2>
          <p className="text-xs text-text-muted">{t('mcp.sync.subtitle')}</p>
        </div>
        <div className="flex items-center gap-2">
          <button type="button" className="inline-flex min-h-11 items-center gap-1.5 rounded-lg bg-bg-surface px-3 py-1.5 text-xs text-text-secondary" disabled={loading} onClick={onRefresh}>
            <SIcon name="RefreshCw" size="w-3.5 h-3.5" className={loading ? 'animate-spin' : ''} />
            {t('common.refresh')}
          </button>
          <button type="button" className="inline-flex min-h-11 items-center gap-2 rounded-xl bg-accent-success px-4 py-2 text-sm font-bold text-[color:var(--color-success-contrast)] disabled:opacity-60" disabled={syncing || sourceServers.length === 0} onClick={onSyncAll}>
            <SIcon name={syncing ? 'Loader2' : 'Zap'} size="w-4 h-4" className={syncing ? 'animate-spin' : ''} />
            {t('mcp.sync.syncAll')}
          </button>
        </div>
      </div>
      <div className="mb-6">
        <label className="mb-3 block text-xs font-bold uppercase tracking-wider text-text-secondary">{t('mcp.sync.targetPlatforms')}</label>
        <div className="flex flex-wrap gap-2">
          {PLATFORMS.map((platform) => (
            <PlatformChip key={platform.id} id={platform.id} name={platform.name} icon={platform.icon} active={selectedPlatforms.includes(platform.id)} onToggle={togglePlatform} />
          ))}
        </div>
      </div>
      <div className="mb-3 flex items-center justify-between">
        <label className="text-xs font-bold uppercase tracking-wider text-text-secondary">{sourceServersLabel}</label>
        <span className="text-xs text-text-muted">{sourceServers.length} {t('mcp.sync.servers')}</span>
      </div>
      {loading ? <div className="flex justify-center py-8"><div className="loading-spinner h-8 w-8 border-accent-success/30 border-t-accent-success" /></div> : null}
      {!loading && sourceServers.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-border-default bg-bg-elevated py-8 text-center">
          <SIcon name="Server" size="w-10 h-10" className="mx-auto mb-2 text-text-muted opacity-50" />
          <p className="text-sm text-text-muted">{t('mcp.sync.noServers')}</p>
          <p className="mt-1 text-xs text-text-muted">{t('mcp.sync.noServersHint')}</p>
        </div>
      ) : null}
      {!loading && sourceServers.length > 0 ? (
        <div className="space-y-3">
          {sourceServers.map((server) => (
            <ServerCard key={server.name} server={server} sourcePlatformName={sourcePlatformName} results={syncResults[server.name]} syncing={syncingServer === server.name} onSync={handleSyncServer} t={t} />
          ))}
        </div>
      ) : null}
    </div>
  )
}

const PlatformChip = memo(function PlatformChip({
  id,
  name,
  icon,
  active,
  onToggle,
}: {
  id: string
  name: string
  icon: string
  active: boolean
  onToggle: (id: string) => void
}) {
  const handleClick = useCallback(() => {
    onToggle(id)
  }, [id, onToggle])
  return (
    <button type="button" className={cn('inline-flex min-h-11 items-center gap-2 rounded-xl border px-3 py-2 text-xs font-medium', active ? 'border-accent-success/30 bg-accent-success/20 text-accent-success' : 'border-transparent bg-bg-surface text-text-muted')} onClick={handleClick}>
      <SIcon name={icon} size="w-3.5 h-3.5" />
      <span>{name}</span>
    </button>
  )
})
