import { createApp, defineComponent, nextTick, ref } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { SkillRecord } from '@/types/skills'

const unifiedSkillsMock = {
  filteredSkills: ref([{
    id: 'sg_skill_alpha',
    name: 'Skill Alpha',
    description: 'Alpha',
    category: 'ops',
    tags: ['sync'],
    origin: 'unknown',
    installCount: 1,
    sourceRef: undefined,
    sourceLabel: undefined,
    editableInstallations: ['ins_skill_alpha'],
    lifecycle: { targetCount: 1, healthyTargetCount: 1, hasErrors: false },
    targets: [{ id: 'ins_skill_alpha', platformId: 'codex', platformName: 'Codex', targetPath: '/tmp/skill-alpha', syncMode: 'copy', status: 'ok', isPrimary: true }],
    installations: [{ id: 'ins_skill_alpha', platformId: 'codex', platformName: 'Codex', installPath: '/tmp/skill-alpha', installMode: 'copy', isPrimary: true }],
  }] as SkillRecord[]),
  selectedSkill: ref({
    id: 'sg_skill_alpha',
    name: 'Skill Alpha',
    description: 'Alpha',
    category: 'ops',
    tags: ['sync'],
    origin: 'unknown',
    installCount: 1,
    sourceRef: undefined,
    sourceLabel: undefined,
    editableInstallations: ['ins_skill_alpha'],
    lifecycle: { targetCount: 1, healthyTargetCount: 1, hasErrors: false },
    targets: [{ id: 'ins_skill_alpha', platformId: 'codex', platformName: 'Codex', targetPath: '/tmp/skill-alpha', syncMode: 'copy', status: 'ok', isPrimary: true }],
    installations: [{ id: 'ins_skill_alpha', platformId: 'codex', platformName: 'Codex', installPath: '/tmp/skill-alpha', installMode: 'copy', isPrimary: true }],
  }),
  selectedInstallation: ref({ id: 'ins_skill_alpha', platformId: 'codex', platformName: 'Codex', installPath: '/tmp/skill-alpha', installMode: 'copy', isPrimary: true }),
  mutationLoading: ref(false),
  selectSkill: vi.fn(),
  ensureDetail: vi.fn(),
  ensureContent: vi.fn(async () => ({
    raw: '---\nname: skill-alpha\ndescription: Alpha\n---\n\n# Skill Alpha\n\nA tracked note.',
    content: '---\nname: skill-alpha\ndescription: Alpha\n---\n\n# Skill Alpha\n\nA tracked note.',
    skillId: 'sg_skill_alpha',
    installationId: 'ins_skill_alpha',
    name: 'Skill Alpha',
    tags: [],
    skillDir: '/tmp/skill-alpha',
  })),
  ensureFiles: vi.fn(async () => [
    { path: 'SKILL.md', size: 10, isDir: false },
    { path: 'notes.md', size: 5, isDir: false },
  ]),
  ensureFileContent: vi.fn(async (_skillId: string, path: string) => ({ skillId: 'sg_skill_alpha', installationId: 'ins_skill_alpha', path, content: `content:${path}` })),
  saveContent: vi.fn(),
  syncSkill: vi.fn(),
  removeInstallation: vi.fn(),
  removeSkillRecord: vi.fn(),
}

vi.mock('@/composables/useUnifiedSkills', () => ({
  useUnifiedSkills: () => unifiedSkillsMock,
}))
vi.mock('@tanstack/vue-virtual', () => ({
  useVirtualizer: vi.fn(() => ref({
    getTotalSize: () => 100,
    getVirtualItems: () => [{ index: 0, start: 0 }],
    measureElement: vi.fn(),
  })),
}))
vi.mock('@/components/ui/SIcon.vue', () => ({ default: defineComponent({ template: '<span />' }) }))
vi.mock('@/stores/ui', () => ({ useUIStore: () => ({ showSuccess: vi.fn(), showError: vi.fn() }) }))

const mountPanel = async () => {
  const { default: InventoryPanel } = await import('@/components/skills/InventoryPanel.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)
  const app = createApp(InventoryPanel, { selectedPlatforms: ['codex'] })
  app.mount(el)
  await nextTick()
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
  vi.clearAllMocks()
})

describe('InventoryPanel file tree smoke', () => {
  it('switches file preview when clicking a file row', async () => {
    const { el, unmount } = await mountPanel()
    try {
      const filesButton = [...el.querySelectorAll('button')].find((button) => button.textContent?.includes('Files')) as HTMLButtonElement | undefined
      filesButton?.click()
      await nextTick()
      const buttons = [...el.querySelectorAll('[data-testid="content-file-row"]')] as HTMLButtonElement[]
      expect(buttons.length).toBeGreaterThan(1)
      buttons[1]?.click()
      await nextTick()
      expect(unifiedSkillsMock.ensureFileContent).toHaveBeenCalledWith('sg_skill_alpha', 'notes.md', 'ins_skill_alpha', true)
    } finally {
      unmount()
    }
  })

  it('renders markdown without frontmatter and uses legacy source copy for unknown origin', async () => {
    const { el, unmount } = await mountPanel()
    try {
      await nextTick()
      await nextTick()
      expect(el.textContent).toContain('Legacy install')
      expect(el.textContent).toContain('Untracked source')
      expect(el.textContent).toContain('Skill Alpha')
      expect(el.textContent).not.toContain('name: skill-alpha')
    } finally {
      unmount()
    }
  })
})
