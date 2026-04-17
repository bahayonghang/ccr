import { createApp, h, nextTick } from 'vue'
import { describe, expect, it } from 'vitest'
import SkillMergeSuggestionsPanel from '@/components/skills/SkillMergeSuggestionsPanel.vue'
import type { MergeSuggestion } from '@/types/skillVersioning'
import { createI18nStub } from './helpers/i18n-stub'

function mount(component: () => ReturnType<typeof h>): { container: HTMLElement, app: ReturnType<typeof createApp> } {
  const container = document.createElement('div')
  const app = createApp({ render: component })
  app.use(createI18nStub())
  app.mount(container)
  return { container, app }
}

const samples: MergeSuggestion[] = [
  {
    categoryId: 'code-dev',
    categoryName: '代码开发',
    reason: '同属代码开发 (85%)',
    skills: [
      { id: 'a', name: 'tdd-runner' },
      { id: 'b', name: 'test-runner' },
    ],
    similarity: 0.85,
  },
  {
    categoryId: 'image-gen',
    categoryName: '图片生成',
    reason: '同属图片生成 (42%)',
    skills: [
      { id: 'c', name: 'cover-maker' },
      { id: 'd', name: 'image-cover' },
    ],
    similarity: 0.42,
  },
]

describe('SkillMergeSuggestionsPanel', () => {
  it('renders suggestion pairs', async () => {
    const { container, app } = mount(() =>
      h(SkillMergeSuggestionsPanel, { suggestions: samples }),
    )
    await nextTick()
    const text = container.textContent ?? ''
    expect(text).toContain('tdd-runner')
    expect(text).toContain('test-runner')
    expect(text).toContain('cover-maker')
    expect(text).toContain('image-cover')
    expect(text).toContain('85%')
    expect(text).toContain('42%')
    app.unmount()
  })

  it('shows empty state when no suggestions', async () => {
    const { container, app } = mount(() =>
      h(SkillMergeSuggestionsPanel, { suggestions: [] }),
    )
    await nextTick()
    expect(container.textContent).toContain('No redundant skills detected')
    app.unmount()
  })

  it('renders the count badge matching list length', async () => {
    const { container, app } = mount(() =>
      h(SkillMergeSuggestionsPanel, { suggestions: samples }),
    )
    await nextTick()
    const countBadge = container.querySelector('.merge-panel__count')
    expect(countBadge?.textContent?.trim()).toBe('2')
    app.unmount()
  })
})
