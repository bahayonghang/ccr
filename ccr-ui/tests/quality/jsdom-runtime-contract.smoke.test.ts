import { describe, expect, it } from 'vitest'

describe('jsdom 30 runtime contract', () => {
  it('uses the native PointerEvent implementation for element clicks', () => {
    const button = document.createElement('button')
    let observed: Event | null = null
    button.addEventListener('click', (event) => {
      observed = event
    })

    button.click()

    expect(typeof PointerEvent).toBe('function')
    expect(observed).toBeInstanceOf(PointerEvent)
  })

  it('keeps localhost secure-cookie semantics from tough-cookie 6', () => {
    const cookieName = 'ccr-jsdom-secure-probe'
    expect(window.location.hostname).toBe('localhost')

    try {
      document.cookie = `${cookieName}=1; Path=/; Secure`
      expect(document.cookie).toContain(`${cookieName}=1`)
    } finally {
      document.cookie = `${cookieName}=; Path=/; Secure; Max-Age=0`
    }
  })

  it('uses the jsdom 30 CSSOM while leaving layout-only APIs unpolyfilled', () => {
    const style = document.createElement('style')
    const probe = document.createElement('div')
    style.textContent = '.jsdom-css-probe { display: block; font-size: 0.75rem; }'
    probe.className = 'jsdom-css-probe'
    document.head.appendChild(style)
    document.body.appendChild(probe)

    try {
      const computed = window.getComputedStyle(probe)
      expect(computed.display).toBe('block')
      expect(computed.fontSize).toBe('12px')
      expect(typeof globalThis.ResizeObserver).toBe('undefined')
      expect(typeof Element.prototype.scrollIntoView).toBe('undefined')
    } finally {
      probe.remove()
      style.remove()
    }
  })
})
