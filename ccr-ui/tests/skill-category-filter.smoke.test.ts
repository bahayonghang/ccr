import { createApp, h, nextTick } from 'vue'
import { describe, expect, it } from 'vitest'
import SkillCategoryFilter from '@/components/skills/SkillCategoryFilter.vue'
import type { CategorySummary } from '@/types/skillVersioning'
import { createI18nStub } from './helpers/i18n-stub'

function mount(component: () => ReturnType<typeof h>): { container: HTMLElement, app: ReturnType<typeof createApp> } {
  const container = document.createElement('div')
  const app = createApp({ render: component })
  app.use(createI18nStub())
  app.mount(container)
  return { container, app }
}

const sampleCats: CategorySummary[] = [
  {
    id: 'code-dev',
    nameEn: 'Code Development',
    nameZh: '代码开发',
    icon: '💻',
    count: 7,
    skillIds: ['a', 'b', 'c', 'd', 'e', 'f', 'g'],
  },
  {
    id: 'image-gen',
    nameEn: 'Image Generation',
    nameZh: '图片生成',
    icon: '🎨',
    count: 3,
    skillIds: ['h', 'i', 'j'],
  },
]

describe('SkillCategoryFilter', () => {
  it('renders chips for every category', async () => {
    const { container, app } = mount(() =>
      h(SkillCategoryFilter, { categories: sampleCats, selectedId: null }),
    )
    await nextTick()
    const text = container.textContent ?? ''
    expect(text).toContain('Code Development')
    expect(text).toContain('Image Generation')
    expect(text).toContain('7')
    expect(text).toContain('3')
    app.unmount()
  })

  it('shows empty state when categories list is empty', async () => {
    const { container, app } = mount(() =>
      h(SkillCategoryFilter, { categories: [], selectedId: null }),
    )
    await nextTick()
    expect(container.textContent).toContain('No categories yet')
    app.unmount()
  })

  it('renders Chinese names when locale is zh', async () => {
    const { container, app } = mount(() =>
      h(SkillCategoryFilter, { categories: sampleCats, selectedId: null, locale: 'zh' }),
    )
    await nextTick()
    const text = container.textContent ?? ''
    expect(text).toContain('代码开发')
    expect(text).toContain('图片生成')
    app.unmount()
  })
})
