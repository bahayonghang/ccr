import { createApp, nextTick } from 'vue'
import { describe, expect, it, beforeEach, vi } from 'vitest'
import PricingView from '@/views/PricingView.vue'
import { createI18nStub } from './helpers/i18n-stub'

const apiMocks = vi.hoisted(() => ({
  getPricingList: vi.fn(),
  setPricing: vi.fn(),
  removePricing: vi.fn(),
  resetPricing: vi.fn(),
}))

vi.mock('@/api', () => ({
  getPricingList: apiMocks.getPricingList,
  setPricing: apiMocks.setPricing,
  removePricing: apiMocks.removePricing,
  resetPricing: apiMocks.resetPricing,
}))

const pricingResponse = {
  items: [
    {
      model: 'claude-sonnet-4-5',
      pricing: {
        model: 'claude-sonnet-4-5',
        input_price: 3,
        output_price: 15,
        cache_read_price: 0.3,
        cache_write_price: 3.75,
      },
    },
  ],
  total: 1,
}

const flushPromises = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await nextTick()
}

const mountPricingView = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(PricingView)
  app.use(createI18nStub())
  app.mount(el)
  await flushPromises()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

const setInputValue = async (input: HTMLInputElement, value: string) => {
  input.value = value
  input.dispatchEvent(new Event('input'))
  await nextTick()
}

describe('PricingView smoke', () => {
  beforeEach(() => {
    apiMocks.getPricingList.mockResolvedValue(pricingResponse)
    apiMocks.setPricing.mockResolvedValue({})
    apiMocks.removePricing.mockResolvedValue({ removed: true })
    apiMocks.resetPricing.mockResolvedValue({})
  })

  it('labels pricing as legacy CCR source of truth with /MTok units', async () => {
    const mounted = await mountPricingView()

    try {
      const text = mounted.el.textContent ?? ''

      expect(text).toContain('Legacy CCR Pricing')
      expect(text).toContain('Legacy CCR pricing')
      expect(text).toContain('~/.claude/pricing.toml')
      expect(text).toContain('Changes do not recalculate llmusage dashboard costs')
      expect(text).toContain('USD / MTok')
      expect(text).not.toContain('/1K tokens')
      expect(text).not.toContain('/ 1K tokens')
      expect(text).not.toContain('Recalculate now')
    } finally {
      mounted.unmount()
    }
  })

  it('saves pricing through the existing API and reloads the list', async () => {
    const mounted = await mountPricingView()

    try {
      const addButton = [...mounted.el.querySelectorAll('button')].find((button) =>
        button.textContent?.includes('Add pricing row'),
      ) as HTMLButtonElement
      addButton.click()
      await nextTick()

      const inputs = [...mounted.el.querySelectorAll('form input')] as HTMLInputElement[]
      await setInputValue(inputs[0], 'new-model')
      await setInputValue(inputs[1], '1.25')
      await setInputValue(inputs[2], '2.5')
      await setInputValue(inputs[3], '0.2')
      await setInputValue(inputs[4], '0.4')

      const form = mounted.el.querySelector('form') as HTMLFormElement
      form.dispatchEvent(new Event('submit', { cancelable: true }))
      await flushPromises()

      expect(apiMocks.setPricing).toHaveBeenCalledWith({
        model: 'new-model',
        input_price: 1.25,
        output_price: 2.5,
        cache_read_price: 0.2,
        cache_write_price: 0.4,
      })
      expect(apiMocks.getPricingList).toHaveBeenCalledTimes(2)
      expect(mounted.el.textContent).toContain('Pricing row created for new-model.')
    } finally {
      mounted.unmount()
    }
  })

  it('uses in-page confirmation for remove and reset actions', async () => {
    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => undefined)
    const confirmSpy = vi.spyOn(window, 'confirm').mockImplementation(() => true)
    const mounted = await mountPricingView()

    try {
      const removeButton = [...mounted.el.querySelectorAll('button')].find((button) =>
        button.textContent?.includes('Remove row'),
      ) as HTMLButtonElement
      removeButton.click()
      await nextTick()

      expect(confirmSpy).not.toHaveBeenCalled()
      expect(alertSpy).not.toHaveBeenCalled()
      expect(mounted.el.textContent).toContain('Remove claude-sonnet-4-5?')

      const deleteConfirm = mounted.el.querySelector('.pricing-confirm .pricing-button--danger') as HTMLButtonElement
      deleteConfirm.click()
      await flushPromises()
      expect(apiMocks.removePricing).toHaveBeenCalledWith('claude-sonnet-4-5')

      const resetButton = [...mounted.el.querySelectorAll('button')].find((button) =>
        button.textContent?.includes('Reset CCR defaults'),
      ) as HTMLButtonElement
      resetButton.click()
      await nextTick()

      expect(confirmSpy).not.toHaveBeenCalled()
      expect(mounted.el.textContent).toContain('Reset the legacy CCR pricing table?')

      const resetConfirm = mounted.el.querySelector('.pricing-confirm .pricing-button--danger') as HTMLButtonElement
      resetConfirm.click()
      await flushPromises()
      expect(apiMocks.resetPricing).toHaveBeenCalled()
      expect(alertSpy).not.toHaveBeenCalled()
    } finally {
      mounted.unmount()
      alertSpy.mockRestore()
      confirmSpy.mockRestore()
    }
  })
})
