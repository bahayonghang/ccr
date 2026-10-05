import { memo, useCallback } from 'react'
import type { SshConnectResult, SshHostConfig } from '@/api'

interface SshHostRowProps {
  host: SshHostConfig
  envId: string
  testing: boolean
  testResult?: SshConnectResult
  tt: (zh: string, en: string) => string
  onTest: (host: SshHostConfig) => void
  onConnect: (host: SshHostConfig) => void
}

export const SshHostRow = memo(function SshHostRow({
  host,
  envId,
  testing,
  testResult,
  tt,
  onTest,
  onConnect,
}: SshHostRowProps) {
  const handleTest = useCallback(() => {
    onTest(host)
  }, [host, onTest])
  const handleConnect = useCallback(() => {
    onConnect(host)
  }, [host, onConnect])
  const resultText = testResult
    ? testResult.success
      ? `${tt('连通', 'Reachable')} (${testResult.latency_ms} ms)`
      : `${tt('失败', 'Failed')}: ${testResult.error ?? ''}`
    : null

  return (
    <div className="rounded-lg border border-border-default/15 p-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-sm font-medium text-text-primary">{host.name || host.host}</div>
          <div className="text-xs text-text-muted">{`${host.user || 'user'}@${host.host}:${host.port || 22}`}</div>
        </div>
        <div className="flex items-center gap-1">
          <button type="button" className="rounded border border-border-default/15 px-2 py-1 text-xs" disabled={testing} onClick={handleTest}>
            {testing ? tt('测试中…', 'Testing...') : tt('测试连接', 'Test connection')}
          </button>
          <button type="button" className="rounded border border-border-default/15 px-2 py-1 text-xs" onClick={handleConnect}>
            {tt('连接', 'Connect')}
          </button>
        </div>
      </div>
      {resultText ? <div className={`mt-2 text-xs ${testResult?.success ? 'text-accent-success' : 'text-accent-danger'}`}>{resultText}</div> : null}
      <span className="sr-only">{envId}</span>
    </div>
  )
})
