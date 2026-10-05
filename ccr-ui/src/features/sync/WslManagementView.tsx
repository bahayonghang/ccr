import { memo, useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import {
  clearWslCache,
  detectWslCli,
  getWslCacheStatus,
  listWslDistros,
  readWslConfig,
  refreshWslDistros,
  syncWslConfig,
  type WslCacheStatus,
  type WslCliStatus,
  type WslDistro,
} from '@/api/runtime/wsl'
import { logger } from '@/utils/logger'
import { getClientPlatform } from '@/utils/windowChrome'
import { PageHeader, PageShell, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, SIcon } from '@/ui'
import { useSyncTt } from './locale'

const PLATFORMS = ['claude', 'codex', 'gemini']

const DistroRow = memo(function DistroRow({
  distro,
  selected,
  onSelect,
  tt,
}: {
  distro: WslDistro
  selected: boolean
  onSelect: (name: string) => void
  tt: (zh: string, en: string) => string
}) {
  const handleClick = useCallback(() => {
    onSelect(distro.name)
  }, [distro.name, onSelect])
  const stateClass = distro.state.toLowerCase() === 'running' ? 'text-accent-success' : distro.state.toLowerCase() === 'stopped' ? 'text-text-muted' : 'text-accent-warning'
  return (
    <button type="button" className={`flex w-full items-center gap-3 rounded-xl border p-3 text-left ${selected ? 'border-accent-primary/30 bg-accent-primary/10 text-accent-primary' : 'border-border-default/25 bg-bg-surface text-text-primary'}`} onClick={handleClick}>
      <SIcon name="Terminal" size="w-5 h-5" />
      <div className="min-w-0 flex-1">
        <div className="truncate font-medium">{distro.name}</div>
        <div className="mt-0.5 flex items-center gap-2 text-[0.625rem]">
          <span className="opacity-60">{`WSL${distro.version === 'Wsl2' ? '2' : '1'}`}</span>
          <span className={stateClass}>{`● ${distro.state}`}</span>
        </div>
      </div>
      {distro.is_default ? <span className="rounded bg-accent-primary/20 px-1.5 py-0.5 text-[0.5625rem] font-bold uppercase text-accent-primary">{tt('默认', 'Default')}</span> : null}
    </button>
  )
})

export function WslManagementView() {
  const tt = useSyncTt()
  const isWindows = getClientPlatform() === 'windows'
  const [distros, setDistros] = useState<WslDistro[]>([])
  const [selectedDistro, setSelectedDistro] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [isSyncing, setIsSyncing] = useState(false)
  const [syncMessage, setSyncMessage] = useState('')
  const [configContent, setConfigContent] = useState('')
  const [cliStatus, setCliStatus] = useState<WslCliStatus>({})
  const [cacheStatus, setCacheStatus] = useState<WslCacheStatus | null>(null)
  const form = useForm<{ platform: string }>({ defaultValues: { platform: 'claude' } })
  const platform = form.watch('platform')

  const fetchCacheStatus = useCallback(async () => {
    try {
      setCacheStatus(await getWslCacheStatus())
    } catch (e) {
      logger.error('[WSL] Failed to get cache status:', e)
    }
  }, [])

  const loadDistroDetails = useCallback(async (name: string, nextPlatform: string) => {
    try {
      setCliStatus(await detectWslCli(name))
    } catch (e) {
      logger.error('[WSL] Failed to detect CLI:', e)
    }
    try {
      setConfigContent(await readWslConfig({ distro: name, platform: nextPlatform, path: 'settings.json' }))
    } catch (e) {
      setConfigContent(`${tt('读取失败', 'Read failed')}: ${e}`)
    }
  }, [tt])

  const fetchDistros = useCallback(async (forceRefresh = false) => {
    setIsLoading(true)
    try {
      const next = await listWslDistros(forceRefresh)
      setDistros(next)
      await fetchCacheStatus()
      if (next.length > 0 && !selectedDistro) {
        setSelectedDistro(next[0].name)
        await loadDistroDetails(next[0].name, platform)
      }
    } catch (e) {
      logger.error('[WSL] Failed to list distros:', e)
    } finally {
      setIsLoading(false)
    }
  }, [fetchCacheStatus, loadDistroDetails, platform, selectedDistro])

  useEffect(() => {
    if (isWindows) void fetchDistros()
  }, [fetchDistros, isWindows])

  const selectDistro = useCallback((name: string) => {
    setSelectedDistro(name)
    void loadDistroDetails(name, platform)
  }, [loadDistroDetails, platform])

  const handlePlatform = useCallback((value: string) => {
    form.setValue('platform', value)
    if (selectedDistro) void loadDistroDetails(selectedDistro, value)
  }, [form, loadDistroDetails, selectedDistro])

  const refresh = useCallback(async () => {
    setIsRefreshing(true)
    await fetchDistros()
    setIsRefreshing(false)
  }, [fetchDistros])
  const forceRefresh = useCallback(async () => {
    setIsRefreshing(true)
    try {
      const next = await refreshWslDistros()
      setDistros(next)
      await fetchCacheStatus()
    } catch (e) {
      logger.error('[WSL] Failed to force refresh:', e)
    } finally {
      setIsRefreshing(false)
    }
  }, [fetchCacheStatus])
  const clearCache = useCallback(async () => {
    try {
      await clearWslCache()
      await fetchCacheStatus()
    } catch (e) {
      logger.error('[WSL] Failed to clear cache:', e)
    }
  }, [fetchCacheStatus])
  const syncConfig = useCallback(async (direction: string) => {
    if (!selectedDistro) return
    setIsSyncing(true)
    setSyncMessage('')
    try {
      setSyncMessage(await syncWslConfig({ distro: selectedDistro, platform, direction }))
    } catch (e) {
      setSyncMessage(`${tt('同步失败', 'Sync failed')}: ${e}`)
    } finally {
      setIsSyncing(false)
    }
  }, [platform, selectedDistro, tt])

  const handleRefresh = useCallback(() => {
    void refresh()
  }, [refresh])
  const handleForce = useCallback(() => {
    void forceRefresh()
  }, [forceRefresh])
  const handleClear = useCallback(() => {
    void clearCache()
  }, [clearCache])
  const pushWsl = useCallback(() => {
    void syncConfig('localToWsl')
  }, [syncConfig])
  const pullWsl = useCallback(() => {
    void syncConfig('wslToLocal')
  }, [syncConfig])

  if (!isWindows) {
    return (
      <PageShell header={<PageHeader title={tt('WSL 环境管理', 'WSL Environment Management')} description={tt('仅 Windows 平台可用', 'Available on Windows only')} />}>
        <div className="rounded-xl border border-border-default/15 bg-bg-surface p-8 text-center text-text-secondary">{tt('当前系统不是 Windows，WSL 管理不可用。', 'This system is not Windows. WSL management is unavailable.')}</div>
      </PageShell>
    )
  }

  return (
    <PageShell
      className="min-w-0"
      header={
        <PageHeader
          title={tt('WSL 环境管理', 'WSL Environment Management')}
          description={tt('管理 Windows Subsystem for Linux 发行版配置', 'Manage Windows Subsystem for Linux distribution configuration')}
          actions={
            <>
              <button type="button" className="flex items-center gap-2 rounded-lg border border-border-default/25 px-4 py-2 text-sm" disabled={isRefreshing} onClick={handleRefresh}>
                <SIcon name="RefreshCw" size="w-4 h-4" className={isRefreshing ? 'animate-spin' : ''} />
                {tt('刷新', 'Refresh')}
              </button>
              <button type="button" className="flex items-center gap-2 rounded-lg border border-accent-primary/30 bg-accent-primary/10 px-4 py-2 text-sm text-accent-primary" disabled={isRefreshing} onClick={handleForce}>
                {tt('强制刷新', 'Force refresh')}
              </button>
            </>
          }
        />
      }
    >
      {cacheStatus ? (
        <div className="mb-4 flex items-center justify-between rounded-lg border border-border-default/25 bg-bg-surface px-4 py-2 text-sm">
          <span>{`${tt('缓存状态', 'Cache status')}: ${cacheStatus.has_disk_cache ? tt('已缓存', 'Cached') : tt('未缓存', 'Not cached')}`}</span>
          <button type="button" className="text-xs text-text-muted" onClick={handleClear}>{tt('清除缓存', 'Clear cache')}</button>
        </div>
      ) : null}
      {isLoading ? <div className="flex justify-center py-12"><div className="loading-spinner h-8 w-8 border-accent-primary/30 border-t-accent-primary" /></div> : null}
      {!isLoading && distros.length === 0 ? (
        <div className="rounded-xl border border-border-default/15 bg-bg-surface p-8 text-center">
          <p className="font-medium text-text-primary">{tt('未检测到 WSL 发行版', 'No WSL distributions detected')}</p>
        </div>
      ) : null}
      {!isLoading && distros.length > 0 ? (
        <div className="grid grid-cols-12 gap-6">
          <div className="col-span-4 space-y-2">
            {distros.map((distro) => (
              <DistroRow key={distro.name} distro={distro} selected={selectedDistro === distro.name} onSelect={selectDistro} tt={tt} />
            ))}
          </div>
          <div className="col-span-8 space-y-6">
            <div className="rounded-xl border border-border-default/15 bg-bg-surface p-4">
              <h3 className="mb-3 text-sm font-semibold">{tt('AI CLI 工具状态', 'AI CLI tool status')}</h3>
              <div className="grid grid-cols-3 gap-3">
                {Object.entries(cliStatus).map(([tool, installed]) => (
                  <div key={tool} className="flex items-center gap-2 text-sm">
                    <SIcon name={installed ? 'CheckCircle2' : 'XCircle'} size="w-4 h-4" className={installed ? 'text-accent-success' : 'text-text-muted'} />
                    <span>{tool}</span>
                  </div>
                ))}
              </div>
            </div>
            <div className="rounded-xl border border-border-default/15 bg-bg-surface p-4">
              <div className="mb-3 flex items-center justify-between">
                <h3 className="text-sm font-semibold">{tt('配置文件', 'Config file')}</h3>
                <Select value={platform} onValueChange={handlePlatform}>
                  <SelectTrigger className="w-40"><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {PLATFORMS.map((item) => (
                      <SelectItem key={item} value={item}>{item}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <pre className="max-h-64 overflow-auto whitespace-pre-wrap rounded-lg p-3 font-mono text-xs">{configContent || '(空)'}</pre>
            </div>
            <div className="rounded-xl border border-border-default/15 bg-bg-surface p-4">
              <h3 className="mb-3 text-sm font-semibold">{tt('配置同步', 'Config sync')}</h3>
              <div className="flex gap-3">
                <button type="button" className="rounded-lg bg-accent-primary/10 px-4 py-2 text-sm text-accent-primary" disabled={isSyncing} onClick={pushWsl}>{tt('推送到 WSL', 'Push to WSL')}</button>
                <button type="button" className="rounded-lg border border-border-default/15 px-4 py-2 text-sm" disabled={isSyncing} onClick={pullWsl}>{tt('从 WSL 拉取', 'Pull from WSL')}</button>
              </div>
              {syncMessage ? <p className="mt-2 text-xs text-text-muted">{syncMessage}</p> : null}
            </div>
          </div>
        </div>
      ) : null}
    </PageShell>
  )
}
