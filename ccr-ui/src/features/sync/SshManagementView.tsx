import { useCallback, useEffect, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import {
  refreshEnvironments,
  sshAddHost,
  sshConnect,
  sshConfirmHostFingerprint,
  sshDetectCli,
  sshDisconnect,
  sshGetConnectionState,
  sshListHosts,
  sshProbeHostFingerprint,
  sshReadConfig,
  sshReconnect,
  sshTestConnection,
  sshWriteConfig,
  type SshConnectResult,
  type SshConnectionState,
  type SshFingerprintProbeResult,
  type SshHostConfig,
} from '@/api'
import { PageHeader, PageShell, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/ui'
import { useSyncTt } from './locale'
import { SshHostRow } from './SshHostRow'

interface HostForm {
  host: string
  port: number
  user: string
  name: string
  id: string
  identity_file: string
  connectPassword: string
  platform: string
  configPath: string
  configContent: string
}

const emptyForm = (): HostForm => ({
  host: '',
  port: 22,
  user: '',
  name: '',
  id: '',
  identity_file: '',
  connectPassword: '',
  platform: 'claude',
  configPath: 'settings.json',
  configContent: '',
})

function buildEnvId(host: SshHostConfig): string {
  return `ssh:${host.id?.trim() || host.host}`
}

export function SshManagementView() {
  const tt = useSyncTt()
  const form = useForm<HostForm>({ defaultValues: emptyForm() })
  const values = form.watch()
  const [hosts, setHosts] = useState<SshHostConfig[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [activeEnvId, setActiveEnvId] = useState('')
  const [activeConnectionState, setActiveConnectionState] = useState<SshConnectionState | null>(null)
  const [pendingFingerprint, setPendingFingerprint] = useState<SshFingerprintProbeResult | null>(null)
  const [cliStatusText, setCliStatusText] = useState('')
  const [testResults, setTestResults] = useState<Record<string, SshConnectResult>>({})
  const [testingHosts, setTestingHosts] = useState<Set<string>>(new Set())
  const selectedHost = useMemo(() => hosts.find((host) => buildEnvId(host) === activeEnvId), [activeEnvId, hosts])
  const selectedHostLabel = selectedHost ? `${selectedHost.user || 'user'}@${selectedHost.host}` : tt('未连接', 'Not connected')

  const loadHosts = useCallback(async () => {
    setLoading(true)
    setError('')
    try {
      const next = await sshListHosts()
      setHosts(Array.isArray(next) ? next : [])
    } catch (e: unknown) {
      setError(e?.toString?.() || tt('加载 SSH 主机失败', 'Failed to load SSH hosts'))
    } finally {
      setLoading(false)
    }
  }, [tt])

  useEffect(() => {
    void loadHosts()
  }, [loadHosts])

  const addHost = useCallback(async () => {
    setError('')
    try {
      if (!values.host.trim()) throw new Error(tt('主机地址不能为空', 'Host is required'))
      await sshAddHost({
        id: values.id.trim() || undefined,
        name: values.name.trim() || undefined,
        host: values.host.trim(),
        port: Number(values.port) || 22,
        user: values.user.trim() || undefined,
        identity_file: values.identity_file.trim() || undefined,
      })
      await refreshEnvironments()
      await loadHosts()
      form.reset({ ...emptyForm(), platform: values.platform, configPath: values.configPath })
    } catch (e: unknown) {
      setError(e?.toString?.() || tt('新增 SSH 主机失败', 'Failed to add SSH host'))
    }
  }, [form, loadHosts, tt, values])

  const connectHost = useCallback(async (host: SshHostConfig) => {
    setError('')
    setPendingFingerprint(null)
    const envId = buildEnvId(host)
    try {
      const probe = await sshProbeHostFingerprint(envId)
      if (probe.status === 'mismatch' || probe.status === 'new') {
        setPendingFingerprint(probe)
        setActiveEnvId(envId)
        return
      }
      setActiveConnectionState(await sshConnect(envId, form.getValues('connectPassword') || undefined))
      setActiveEnvId(envId)
      setCliStatusText('')
    } catch (e: unknown) {
      setError(e?.toString?.() || tt('连接 SSH 主机失败', 'Failed to connect to SSH host'))
    }
  }, [form, tt])

  const confirmFingerprintAndConnect = useCallback(async () => {
    if (!pendingFingerprint || !activeEnvId) return
    try {
      await sshConfirmHostFingerprint(pendingFingerprint.challenge_id)
      setActiveConnectionState(await sshConnect(activeEnvId, form.getValues('connectPassword') || undefined))
      setPendingFingerprint(null)
    } catch (e: unknown) {
      setError(e?.toString?.() || tt('确认指纹失败', 'Failed to confirm fingerprint'))
    }
  }, [activeEnvId, form, pendingFingerprint, tt])

  const testConnect = useCallback(async (host: SshHostConfig) => {
    const envId = buildEnvId(host)
    setTestingHosts((current) => new Set([...current, envId]))
    try {
      const result = await sshTestConnection(envId)
      setTestResults((current) => ({ ...current, [envId]: result }))
    } catch (e: unknown) {
      setTestResults((current) => ({
        ...current,
        [envId]: { success: false, latency_ms: 0, error_code: null, error: e?.toString?.() || tt('测试失败', 'Test failed') },
      }))
    } finally {
      setTestingHosts((current) => {
        const next = new Set(current)
        next.delete(envId)
        return next
      })
    }
  }, [tt])

  const handlePlatform = useCallback((value: string) => {
    form.setValue('platform', value)
  }, [form])
  const handleAdd = useCallback(() => {
    void addHost()
  }, [addHost])
  const handleRefresh = useCallback(() => {
    void loadHosts()
  }, [loadHosts])
  const handleReconnect = useCallback(async () => {
    if (!activeEnvId) return
    setActiveConnectionState(await sshReconnect(activeEnvId, form.getValues('connectPassword') || undefined))
  }, [activeEnvId, form])
  const handleDisconnect = useCallback(async () => {
    setActiveConnectionState(await sshDisconnect())
    setActiveEnvId('')
    setPendingFingerprint(null)
  }, [])
  const handleRead = useCallback(async () => {
    if (!activeEnvId) return
    form.setValue('configContent', await sshReadConfig(activeEnvId, form.getValues('platform'), form.getValues('configPath')))
  }, [activeEnvId, form])
  const handleWrite = useCallback(async () => {
    if (!activeEnvId) return
    await sshWriteConfig(activeEnvId, form.getValues('platform'), form.getValues('configPath'), form.getValues('configContent'), true)
  }, [activeEnvId, form])
  const handleDetect = useCallback(async () => {
    if (!activeEnvId) return
    setCliStatusText(JSON.stringify(await sshDetectCli(activeEnvId), null, 2))
  }, [activeEnvId])
  const handleState = useCallback(async () => {
    if (!activeEnvId) return
    const state = await sshGetConnectionState(activeEnvId)
    if (!Array.isArray(state)) setActiveConnectionState(state)
  }, [activeEnvId])
  const onState = useCallback(() => {
    void handleState()
  }, [handleState])
  const onReconnect = useCallback(() => {
    void handleReconnect()
  }, [handleReconnect])
  const onDisconnect = useCallback(() => {
    void handleDisconnect()
  }, [handleDisconnect])
  const onRead = useCallback(() => {
    void handleRead()
  }, [handleRead])
  const onWrite = useCallback(() => {
    void handleWrite()
  }, [handleWrite])
  const onDetect = useCallback(() => {
    void handleDetect()
  }, [handleDetect])
  const onConfirmFingerprint = useCallback(() => {
    void confirmFingerprintAndConnect()
  }, [confirmFingerprintAndConnect])

  return (
    <PageShell
      className="min-w-0"
      header={
        <PageHeader
          title={tt('SSH 远程管理', 'SSH Remote Management')}
          description={tt('添加主机并连接后执行配置读写和 CLI 检测', 'Add a host, connect, then run config read/write and CLI checks.')}
          actions={<button type="button" className="rounded-lg border border-border-default/15 px-3 py-2" disabled={loading} onClick={handleRefresh}>{tt('刷新主机', 'Refresh hosts')}</button>}
        />
      }
    >
      {error ? <div className="mb-4 rounded-lg border border-accent-danger/30 bg-accent-danger/10 p-3 text-sm text-accent-danger">{error}</div> : null}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <section className="space-y-3 rounded-xl border border-border-default/15 bg-bg-surface p-4">
          <h2 className="text-base font-semibold">{tt('新增 SSH 主机', 'Add SSH host')}</h2>
          <div className="grid grid-cols-2 gap-3">
            <input className="rounded-md border border-border-default/15 px-3 py-2" placeholder={tt('名称（可选）', 'Name (optional)')} {...form.register('name')} />
            <input className="rounded-md border border-border-default/15 px-3 py-2" placeholder="ID" {...form.register('id')} />
            <input className="col-span-2 rounded-md border border-border-default/15 px-3 py-2" placeholder={tt('主机地址', 'Host')} {...form.register('host')} />
            <input type="number" className="rounded-md border border-border-default/15 px-3 py-2" placeholder={tt('端口', 'Port')} {...form.register('port', { valueAsNumber: true })} />
            <input className="rounded-md border border-border-default/15 px-3 py-2" placeholder={tt('用户名', 'User')} {...form.register('user')} />
            <input type="password" className="col-span-2 rounded-md border border-border-default/15 px-3 py-2" placeholder={tt('密码（仅内存，可选）', 'Password (memory only, optional)')} {...form.register('connectPassword')} />
            <input className="col-span-2 rounded-md border border-border-default/15 px-3 py-2" placeholder={tt('私钥路径（可选）', 'Identity file (optional)')} {...form.register('identity_file')} />
          </div>
          <button type="button" className="rounded-lg bg-accent-primary/10 px-3 py-2 text-accent-primary" onClick={handleAdd}>{tt('添加主机', 'Add host')}</button>
        </section>
        <section className="space-y-3 rounded-xl border border-border-default/15 bg-bg-surface p-4">
          <h2 className="text-base font-semibold">{tt('主机列表', 'Host list')}</h2>
          {hosts.length === 0 ? <div className="text-sm text-text-muted">{tt('暂无 SSH 主机', 'No SSH hosts yet')}</div> : null}
          {hosts.map((host) => {
            const envId = buildEnvId(host)
            return <SshHostRow key={envId} host={host} envId={envId} testing={testingHosts.has(envId)} testResult={testResults[envId]} tt={tt} onTest={testConnect} onConnect={connectHost} />
          })}
        </section>
      </div>
      <section className="mt-6 space-y-3 rounded-xl border border-border-default/15 bg-bg-surface p-4">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-base font-semibold">{tt('已连接主机', 'Connected host')}</h2>
          <div className="flex gap-2">
            <button type="button" className="rounded border border-border-default/15 px-2 py-1 text-xs" disabled={!activeEnvId} onClick={onState}>{tt('刷新状态', 'Refresh state')}</button>
            <button type="button" className="rounded border border-border-default/15 px-2 py-1 text-xs" disabled={!activeEnvId} onClick={onReconnect}>{tt('重连', 'Reconnect')}</button>
            <button type="button" className="rounded border border-border-default/15 px-2 py-1 text-xs" disabled={!activeEnvId} onClick={onDisconnect}>{tt('断开', 'Disconnect')}</button>
          </div>
        </div>
        <div className="text-sm text-text-muted">{selectedHostLabel}</div>
        {pendingFingerprint ? (
          <div className="space-y-2 rounded-lg border border-accent-warning/30 bg-accent-warning/10 p-3 text-sm text-accent-warning">
            <div>{tt('检测到主机指纹需要确认：', 'Host fingerprint confirmation required:')}</div>
            <div className="font-mono text-xs">{pendingFingerprint.key_type} {pendingFingerprint.fingerprint}</div>
            <button type="button" className="rounded border border-accent-warning/60 px-2 py-1 text-xs" onClick={onConfirmFingerprint}>{tt('确认并连接', 'Confirm and connect')}</button>
          </div>
        ) : null}
        <div className="flex flex-wrap items-center gap-2">
          <Select value={values.platform} onValueChange={handlePlatform}>
            <SelectTrigger className="w-36"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="claude">claude</SelectItem>
              <SelectItem value="codex">codex</SelectItem>
              <SelectItem value="gemini">gemini</SelectItem>
              <SelectItem value="opencode">opencode</SelectItem>
            </SelectContent>
          </Select>
          <input className="min-w-[14rem] rounded-md border border-border-default/15 px-2 py-1 text-sm" {...form.register('configPath')} />
          <button type="button" className="rounded border px-2 py-1 text-xs" disabled={!activeEnvId} onClick={onRead}>{tt('读取配置', 'Read config')}</button>
          <button type="button" className="rounded border px-2 py-1 text-xs" disabled={!activeEnvId} onClick={onWrite}>{tt('写入配置', 'Write config')}</button>
          <button type="button" className="rounded border px-2 py-1 text-xs" disabled={!activeEnvId} onClick={onDetect}>{tt('检测 CLI', 'Detect CLI')}</button>
        </div>
        <textarea className="min-h-[14rem] w-full rounded-md border border-border-default/15 p-3 font-mono text-xs" {...form.register('configContent')} />
        {cliStatusText ? <pre className="overflow-x-auto rounded-md border border-border-default/15 p-3 text-xs">{cliStatusText}</pre> : null}
        {activeConnectionState ? <div className="text-xs text-text-muted">{`${tt('连接状态', 'Connection state')}: ${activeConnectionState.connected ? tt('已连接', 'Connected') : tt('未连接', 'Not connected')}`}</div> : null}
      </section>
    </PageShell>
  )
}
