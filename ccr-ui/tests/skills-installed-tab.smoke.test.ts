import { createApp, defineComponent, nextTick, ref } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { UnifiedSkill } from '@/types/skills'

vi.mock('@tanstack/vue-virtual', () => ({
  useVirtualizer: vi.fn(() =>
    ref({
      getTotalSize: () => 552,
      getVirtualItems: () => [
        { index: 0, start: 0 },
        { index: 1, start: 184 },
        { index: 2, start: 368 },
      ],
      measureElement: vi.fn(),
    }),
  ),
}))

vi.mock('@/components/skills/SkillCard.vue', () => ({
  default: defineComponent({
    name: 'SkillCardStub',
    props: {
      skill: {
        type: Object,
        required: true,
      },
    },
    template: '<div data-testid="skill-card">{{ skill.name }}</div>',
  }),
}))

const mountInstalledTab = async (skills: UnifiedSkill[]) => {
  const { default: SkillsInstalledTab } = await import('@/components/skills/SkillsInstalledTab.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(SkillsInstalledTab, {
    skills,
    isLoading: false,
  })

  app.config.globalProperties.$t = (key: string) => key
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

describe('SkillsInstalledTab smoke', () => {
  it('renders only the virtualized slice for large skill lists', async () => {
    const skills = Array.from({ length: 1000 }, (_, index) => ({
      name: `Skill ${index}`,
      skillDir: `/tmp/skill-${index}`,
      platform: 'codex',
      platformName: 'Codex',
      tags: [],
    })) satisfies UnifiedSkill[]

    const { el, unmount } = await mountInstalledTab(skills)

    try {
      const cards = [...el.querySelectorAll('[data-testid="skill-card"]')].map((node) =>
        node.textContent?.trim(),
      )

      expect(cards).toEqual(['Skill 0', 'Skill 1', 'Skill 2'])
      expect(cards).not.toContain('Skill 999')
      expect(el.querySelectorAll('[data-testid="skills-installed-row"]')).toHaveLength(3)
    } finally {
      unmount()
    }
  })
})
