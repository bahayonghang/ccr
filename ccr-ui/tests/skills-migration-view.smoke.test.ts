import { createApp, defineComponent, h, nextTick } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, describe, expect, it } from 'vitest'

const mountView = async (component: unknown) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: defineComponent({ template: '<div />' }) },
      { path: '/skills', component: defineComponent({ template: '<div />' }) },
      { path: '/configs', component: defineComponent({ template: '<div />' }) },
    ],
  })

  const app = createApp(defineComponent({
    setup() {
      return () => h(component as never)
    },
  }))

  app.use(router)
  await router.push('/skills')
  await router.isReady()
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
})

describe('SkillsMigrationView smoke', () => {
  it('renders the migration bridge and outbound destination', async () => {
    const { default: SkillsMigrationView } = await import('@/views/SkillsMigrationView.vue')
    const { el, unmount } = await mountView(SkillsMigrationView)

    try {
      expect(el.textContent).toContain('Skills 已从 CCR UI 下线')
      expect(el.textContent).toContain('skills-manage')

      const externalLink = el.querySelector<HTMLAnchorElement>('a[href="https://github.com/iamzhihuix/skills-manage"]')
      expect(externalLink).not.toBeNull()
      expect(externalLink?.target).toBe('_blank')
    } finally {
      unmount()
    }
  })
})
