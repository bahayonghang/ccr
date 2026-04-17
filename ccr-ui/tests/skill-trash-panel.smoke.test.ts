import { createApp, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import SkillTrashPanel from '@/components/skills/SkillTrashPanel.vue'
import type { TrashEntry } from '@/types/skillVersioning'
import { createI18nStub } from './helpers/i18n-stub'

function mount(component: () => ReturnType<typeof h>): { container: HTMLElement, app: ReturnType<typeof createApp> } {
  const container = document.createElement('div')
  const app = createApp({ render: component })
  app.use(createI18nStub())
  app.mount(container)
  return { container, app }
}

const sampleEntries: TrashEntry[] = [
  {
    id: 'trash001',
    skillName: 'deleted-skill',
    originalPath: '/home/user/.claude/skills/deleted-skill',
    deletedAt: '2026-04-17T08:00:00Z',
    expiresAt: '2026-04-24T08:00:00Z',
  },
]

vi.mock('@/api', () => ({
  skillsTrashList: vi.fn(async () => sampleEntries),
  skillsTrashSoftDelete: vi.fn(),
  skillsTrashRestore: vi.fn(),
  skillsTrashPurge: vi.fn(async () => true),
}))

afterEach(() => {
  vi.clearAllMocks()
})

describe('SkillTrashPanel', () => {
  it('renders header and refresh button', async () => {
    const { container, app } = mount(() => h(SkillTrashPanel))
    await nextTick()
    expect(container.textContent).toContain('Trash')
    expect(container.textContent).toContain('auto-purge')
    app.unmount()
  })

  it('lists trash entries after mount', async () => {
    const { container, app } = mount(() => h(SkillTrashPanel))
    await nextTick()
    await new Promise((r) => setTimeout(r, 0))
    await nextTick()

    const text = container.textContent ?? ''
    expect(text).toContain('deleted-skill')
    expect(text).toContain('Restore')
    expect(text).toContain('Delete forever')
    app.unmount()
  })

  it('shows empty state when list is empty', async () => {
    const { skillsTrashList } = await import('@/api')
    ;(skillsTrashList as ReturnType<typeof vi.fn>).mockResolvedValueOnce([])

    const { container, app } = mount(() => h(SkillTrashPanel))
    await nextTick()
    await new Promise((r) => setTimeout(r, 0))
    await nextTick()

    expect(container.textContent).toContain('Trash is empty')
    app.unmount()
  })
})
