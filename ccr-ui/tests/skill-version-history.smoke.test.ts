import { createApp, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import SkillVersionHistoryPanel from '@/components/skills/SkillVersionHistoryPanel.vue'
import type { VersionMeta } from '@/types/skillVersioning'
import { createI18nStub } from './helpers/i18n-stub'

function mount(component: () => ReturnType<typeof h>): { container: HTMLElement, app: ReturnType<typeof createApp> } {
  const container = document.createElement('div')
  const app = createApp({ render: component })
  app.use(createI18nStub())
  app.mount(container)
  return { container, app }
}

const sampleHistory: VersionMeta[] = [
  {
    id: 'abc12345def6',
    skillPath: '/tmp/skill',
    skillName: 'my-skill',
    timestamp: '2026-04-17T10:00:00Z',
    message: 'first snapshot',
    source: 'manual',
    contentHash: 'hash1',
  },
  {
    id: 'fff78901bbb2',
    skillPath: '/tmp/skill',
    skillName: 'my-skill',
    timestamp: '2026-04-17T11:00:00Z',
    message: 'second snapshot',
    source: 'auto',
    contentHash: 'hash2',
  },
]

vi.mock('@/api', () => ({
  skillsVersionList: vi.fn(async () => sampleHistory),
  skillsVersionGet: vi.fn(),
  skillsVersionSnapshot: vi.fn(),
  skillsVersionDiff: vi.fn(),
  skillsVersionRollback: vi.fn(),
}))

afterEach(() => {
  vi.clearAllMocks()
})

describe('SkillVersionHistoryPanel', () => {
  it('renders empty state when installPath is null', async () => {
    const { container, app } = mount(() =>
      h(SkillVersionHistoryPanel, { installPath: null, skillName: 'test' }),
    )
    await nextTick()
    expect(container.textContent).toContain('Version History')
    app.unmount()
  })

  it('renders snapshot rows after history loads', async () => {
    const { container, app } = mount(() =>
      h(SkillVersionHistoryPanel, { installPath: '/tmp/skill', skillName: 'my-skill' }),
    )
    await nextTick()
    await new Promise((r) => setTimeout(r, 0))
    await nextTick()

    const text = container.textContent ?? ''
    expect(text).toContain('first snapshot')
    expect(text).toContain('second snapshot')
    expect(text).toContain('abc12345')
    app.unmount()
  })

  it('exposes compact controls per row', async () => {
    const { container, app } = mount(() =>
      h(SkillVersionHistoryPanel, { installPath: '/tmp/skill', skillName: 'my-skill' }),
    )
    await nextTick()
    await new Promise((r) => setTimeout(r, 0))
    await nextTick()

    const text = container.textContent ?? ''
    expect(text).toMatch(/Rollback/)
    expect(text).toMatch(/Base/)
    app.unmount()
  })
})
