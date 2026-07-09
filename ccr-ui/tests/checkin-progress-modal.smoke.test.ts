import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { CheckinFlowPhase, CheckinLogEntry } from '@/types/checkin'
import { createI18nStub } from './helpers/i18n-stub'

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, required: true },
      title: { type: String, default: '' },
    },
    setup(props, { slots }) {
      return () => props.modelValue
        ? h('div', { 'data-title': props.title }, [
            h('h2', props.title),
            slots.default?.(),
          ])
        : null
    },
  }),
}))

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: props.size })
    },
  }),
}))

import CheckinProgressModal from '@/components/CheckinProgressModal.vue'

const mountModal = async (
  phase: CheckinFlowPhase,
  logs: CheckinLogEntry[],
  recoveryMessage?: string,
  recoveryProviderName?: string
) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () =>
        h(CheckinProgressModal, {
          isOpen: true,
          total: 5,
          current: 5,
          currentAccountName: '',
          logs,
          phase,
          recoveryMessage,
          recoveryProviderName,
        })
    },
  }))

  app.use(createI18nStub('zh-CN'))
  app.mount(el)
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  document.body.innerHTML = ''
})

describe('CheckinProgressModal smoke', () => {
  it('renders recovery state without close action', async () => {
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

    const { el, unmount } = await mountModal(
      'recovering',
      logs,
      '已获取 AnyRouter 的 WAF Cookie，正在重试 1 个账号',
      'AnyRouter'
    )

    try {
      expect(el.textContent).toContain('正在自动处理 WAF')
      expect(el.textContent).toContain('已获取 AnyRouter 的 WAF Cookie，正在重试 1 个账号')
      expect(el.textContent).toContain('当前提供商：AnyRouter')
      expect(el.textContent).toContain('自动补救后成功')
      expect(el.textContent).not.toContain('确定')
    } finally {
      unmount()
    }
  })

  it('renders finished state with final close action', async () => {
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

    const { el, unmount } = await mountModal('finished', logs)

    try {
      expect(el.textContent).toContain('签到完成')
      expect(el.textContent).toContain('全部任务执行完毕')
      expect(el.textContent).toContain('自动补救失败')
      expect(el.textContent).toContain('自动获取 WAF Cookie 失败：缺少 WAF Cookie: acw_sc__v2')
      expect(el.textContent).toContain('确认')
    } finally {
      unmount()
    }
  })

  it('renders manual-WAF terminal state when waf_blocked failures remain', async () => {
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

    const { el, unmount } = await mountModal('finished', logs)

    try {
      // 仍有 WAF 未恢复时，终态应区别于绿色"全部完成"，引导用户手动处理
      expect(el.textContent).toContain('需手动处理 WAF')
      expect(el.textContent).toContain('仍被 WAF 拦截')
      expect(el.textContent).not.toContain('全部任务执行完毕')
      expect(el.textContent).toContain('确认')
    } finally {
      unmount()
    }
  })
})
