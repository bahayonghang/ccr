import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { MonitoringLogRow } from '@/features/monitoring/MonitoringLogRow'
import type { MonitoringEntry } from '@/features/monitoring/monitoring-types'
import { redactLogText } from '@/utils/logRedact'

describe('monitoring log sanitize', () => {
  it('does not execute injected script markup', () => {
    const log: MonitoringEntry = {
      id: 'row-1',
      timestamp: new Date().toISOString(),
      level: 'info',
      channel: 'runtime',
      eventType: 'log',
      source: 'test',
      message: '\u001b[31mred\u001b[0m <img src=x onerror="window.__xss=1"> <script>window.__xss=1</script>',
    }
    render(<MonitoringLogRow log={log} locale="zh-CN" />)
    expect((window as Window & { __xss?: number }).__xss).toBeUndefined()
    expect(document.querySelector('script')).toBeNull()
  })

  it('redacts plaintext credentials before display helpers run', () => {
    expect(redactLogText('apiKey=sk-ant-1234567890abcdef')).not.toContain('sk-ant-1234567890abcdef')
  })
})
