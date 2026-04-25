import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'

const accessibilityMocks = vi.hoisted(() => ({
  focusFirstElement: vi.fn(),
  saveFocus: vi.fn(),
  restoreFocus: vi.fn(),
}))

vi.mock('@/composables/useAccessibility', () => ({
  useFocusTrap: () => ({ focusFirstElement: accessibilityMocks.focusFirstElement }),
  useEscapeKey: vi.fn(),
  useUniqueId: (prefix: string) => `${prefix}-test`,
  focusUtils: {
    createFocusStore: () => ({
      save: accessibilityMocks.saveFocus,
      restore: accessibilityMocks.restoreFocus,
    }),
  },
}))

const flushPromises = async () => {
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

const dispatchPointer = (
  element: Element,
  type: 'pointerdown' | 'pointerup' | 'pointercancel',
  options: { x?: number; y?: number; pointerId?: number } = {},
) => {
  const event = new Event(type, { bubbles: true, cancelable: true })
  Object.defineProperties(event, {
    clientX: { value: options.x ?? 0 },
    clientY: { value: options.y ?? 0 },
    pointerId: { value: options.pointerId ?? 1 },
  })
  element.dispatchEvent(event)
}

const mountModal = async () => {
  const { default: BaseModal } = await import('@/components/common/BaseModal.vue')
  const el = document.createElement('div')
  const updates: boolean[] = []
  const close = vi.fn()
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(BaseModal, {
        modelValue: true,
        title: 'Profile',
        'onUpdate:modelValue': (value: boolean) => {
          updates.push(value)
        },
        onClose: close,
      }, {
        default: () => h('input', { value: 'bwen' }),
      })
    },
  }))

  app.mount(el)
  await flushPromises()

  return {
    updates,
    close,
    root: () => document.body.querySelector('.base-modal-root') as HTMLElement,
    dialog: () => document.body.querySelector('[role="dialog"]') as HTMLElement,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  document.body.innerHTML = ''
  document.body.style.overflow = ''
  vi.clearAllMocks()
})

describe('BaseModal smoke', () => {
  it('closes on a direct backdrop pointer click', async () => {
    const modal = await mountModal()

    try {
      dispatchPointer(modal.root(), 'pointerdown', { x: 20, y: 20 })
      dispatchPointer(modal.root(), 'pointerup', { x: 22, y: 22 })

      expect(modal.updates).toEqual([false])
      expect(modal.close).toHaveBeenCalledTimes(1)
    } finally {
      modal.unmount()
    }
  })

  it('keeps the modal open when dragging from content to backdrop', async () => {
    const modal = await mountModal()

    try {
      dispatchPointer(modal.dialog(), 'pointerdown', { x: 120, y: 120 })
      dispatchPointer(modal.root(), 'pointerup', { x: 10, y: 10 })

      expect(modal.updates).toEqual([])
      expect(modal.close).not.toHaveBeenCalled()
    } finally {
      modal.unmount()
    }
  })

  it('keeps the modal open when a backdrop press becomes a drag', async () => {
    const modal = await mountModal()

    try {
      dispatchPointer(modal.root(), 'pointerdown', { x: 10, y: 10 })
      dispatchPointer(modal.root(), 'pointerup', { x: 30, y: 10 })

      expect(modal.updates).toEqual([])
      expect(modal.close).not.toHaveBeenCalled()
    } finally {
      modal.unmount()
    }
  })
})
