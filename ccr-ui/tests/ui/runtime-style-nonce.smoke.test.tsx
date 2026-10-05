import { cleanup, render } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { ConfirmModal } from '@/ui/confirm-modal'
import { initializeRuntimeStyleNonce, readPageCspNonce } from '@/utils/cspNonce'

describe('runtime style nonce', () => {
  let originalNonce: PropertyDescriptor | undefined
  const bootstrapElements: HTMLElement[] = []

  beforeEach(() => {
    originalNonce = Object.getOwnPropertyDescriptor(globalThis, '__webpack_nonce__')
    Reflect.deleteProperty(globalThis, '__webpack_nonce__')
  })

  afterEach(() => {
    cleanup()
    bootstrapElements.splice(0).forEach((element) => element.remove())
    if (originalNonce) Object.defineProperty(globalThis, '__webpack_nonce__', originalNonce)
    else Reflect.deleteProperty(globalThis, '__webpack_nonce__')
  })

  function bootstrapNonce(tag: 'style' | 'script', nonce: string) {
    const element = document.createElement(tag)
    element.nonce = nonce
    document.head.appendChild(element)
    bootstrapElements.push(element)
  }

  function renderConfirmation() {
    return render(<ConfirmModal isOpen title="Synthetic confirmation" message="Synthetic settings only" />)
  }

  function scrollLockStyle() {
    return [...document.querySelectorAll('style')]
      .find((style) => style.textContent?.includes('body[data-scroll-locked]'))
  }

  it('applies the page style nonce to the real confirmation scroll lock and releases it on unmount', () => {
    bootstrapNonce('style', 'synthetic-style-nonce')
    bootstrapNonce('script', 'synthetic-script-nonce')
    initializeRuntimeStyleNonce()
    const view = renderConfirmation()
    expect(scrollLockStyle()?.nonce).toBe('synthetic-style-nonce')
    expect(scrollLockStyle()?.sheet).toBeTruthy()
    expect(document.body.hasAttribute('data-scroll-locked')).toBe(true)
    expect(getComputedStyle(document.body).overflow).toBe('hidden')
    view.unmount()
    expect(scrollLockStyle()).toBeUndefined()
    expect(document.body.hasAttribute('data-scroll-locked')).toBe(false)
  })

  it('uses the script nonce when the page has no nonced style', () => {
    bootstrapNonce('script', 'synthetic-script-nonce')
    initializeRuntimeStyleNonce()
    renderConfirmation()
    expect(readPageCspNonce()).toBe('synthetic-script-nonce')
    expect(scrollLockStyle()?.nonce).toBe('synthetic-script-nonce')
  })

  it('keeps ordinary web scroll locking without introducing a nonce', () => {
    initializeRuntimeStyleNonce()
    renderConfirmation()
    expect(Object.getOwnPropertyDescriptor(globalThis, '__webpack_nonce__')).toBeUndefined()
    expect(scrollLockStyle()?.nonce).toBe('')
    expect(getComputedStyle(document.body).overflow).toBe('hidden')
  })
})
