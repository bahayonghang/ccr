import { render } from '@testing-library/react'
import { afterEach, beforeAll, describe, expect, it } from 'vitest'
import type { CheckinFlowPhase, CheckinLogEntry } from '@/types/checkin'
import { CheckinProgressModal } from '@/features/checkin/components/CheckinProgressModal'
import { setLocale } from '@/i18n'

beforeAll(async () => {
  await setLocale('zh-CN')
  class ResizeObserverStub {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  }
  if (typeof globalThis.ResizeObserver === 'undefined') {
    globalThis.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver
  }
})

const mountModal = (
  phase: CheckinFlowPhase,
  logs: CheckinLogEntry[],
  recoveryMessage?: string,
  recoveryProviderName?: string,
) =>
  render(
    <CheckinProgressModal
      isOpen
      total={5}
      current={5}
      currentAccountName=""
      logs={logs}
      phase={phase}
      recoveryMessage={recoveryMessage}
      recoveryProviderName={recoveryProviderName}
    />,
  )

afterEach(() => {
  localStorage.setItem('ccr-ui-locale', 'zh-CN')
})

describe('CheckinProgressModal smoke', () => {
  it('renders recovery state without close action', () => {
    const logs: CheckinLogEntry[] = [
      {
        accountId: 'acc-1',
        accountName: 'anyrouter_stumail',
        providerName: 'AnyRouter',
        status: 'success',
        message: '签到成功，获得 $25 额度',
        wafRecoveryAttempted: true,
        wafRecovered: true,
        timestamp: new Date('2026-03-23T09:00:05.000Z'),
      },
    ]
    const { container } = mountModal(
      'recovering',
      logs,
      '已获取 AnyRouter 的 WAF Cookie，正在重试 1 个账号',
      'AnyRouter',
    )
    expect(container.textContent + document.body.textContent).toContain('正在自动处理 WAF')
    expect(document.body.textContent).toContain('已获取 AnyRouter 的 WAF Cookie，正在重试 1 个账号')
    expect(document.body.textContent).toContain('当前提供商：AnyRouter')
    expect(document.body.textContent).toContain('自动补救后成功')
    expect(document.body.textContent).not.toContain('确定')
  })

  it('renders finished state with final close action', () => {
    const logs: CheckinLogEntry[] = [
      {
        accountId: 'acc-1',
        accountName: 'anyrouter_stumail',
        providerName: 'AnyRouter',
        status: 'failed',
        message: 'API error: 检测到 WAF 挑战页面',
        wafRecoveryAttempted: true,
        wafRecovered: false,
        wafRecoveryError: '自动获取 WAF Cookie 失败：缺少 WAF Cookie: acw_sc__v2',
        timestamp: new Date('2026-03-23T09:00:05.000Z'),
      },
    ]
    mountModal('finished', logs)
    expect(document.body.textContent).toContain('签到完成')
    expect(document.body.textContent).toContain('全部任务执行完毕')
    expect(document.body.textContent).toContain('自动补救失败')
    expect(document.body.textContent).toContain('自动获取 WAF Cookie 失败：缺少 WAF Cookie: acw_sc__v2')
    expect(document.body.textContent).toContain('确认')
  })

  it('renders manual-WAF terminal state when waf_blocked failures remain', () => {
    const logs: CheckinLogEntry[] = [
      {
        accountId: 'acc-1',
        accountName: 'anyrouter_stumail',
        providerName: 'AnyRouter',
        status: 'failed',
        message: 'API error: 检测到 WAF 挑战页面',
        errorCode: 'waf_blocked',
        wafRecoveryAttempted: true,
        wafRecovered: false,
        timestamp: new Date('2026-03-23T09:00:05.000Z'),
      },
    ]
    mountModal('finished', logs)
    expect(document.body.textContent).toContain('需手动处理 WAF')
    expect(document.body.textContent).toContain('仍被 WAF 拦截')
    expect(document.body.textContent).not.toContain('全部任务执行完毕')
    expect(document.body.textContent).toContain('确认')
  })
})
