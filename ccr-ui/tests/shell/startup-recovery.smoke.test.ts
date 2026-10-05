import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { logger } from '@/utils/logger'
import { installStartupErrorHandlers, renderFatalStartup } from '@/utils/startupRecovery'

describe('startup recovery smoke', () => {
  afterEach(() => {
    document.body.innerHTML = ''
    vi.restoreAllMocks()
  })

  it('renders a fatal startup placeholder into #app', () => {
    document.body.innerHTML = '<div id="app"></div>'

    renderFatalStartup('Router initialization: boom')

    const mountNode = document.querySelector('#app')
    expect(mountNode?.textContent).toContain('CCR Desktop failed to finish startup')
    expect(mountNode?.textContent).toContain('Router initialization: boom')
  })

  it('keeps window errors on the startup failure path before uninstall', () => {
    document.body.innerHTML = '<div id="app">shell</div>'
    const errorSpy = vi.spyOn(logger, 'error')
    const uninstall = installStartupErrorHandlers()

    window.dispatchEvent(new ErrorEvent('error', {
      message: 'boom',
      cancelable: true,
    }))

    expect(document.querySelector('#app')?.textContent).toContain('CCR Desktop failed to finish startup')
    expect(errorSpy).toHaveBeenCalledWith('[startup] Unhandled window error failed', expect.anything())
    uninstall()
  })

  it('does not replace #app after startup handlers uninstall', () => {
    document.body.innerHTML = '<div id="app">shell</div>'
    const errorSpy = vi.spyOn(logger, 'error')
    const uninstall = installStartupErrorHandlers()
    uninstall()

    window.dispatchEvent(new ErrorEvent('error', {
      message: 'boom',
      cancelable: true,
    }))

    expect(document.querySelector('#app')?.textContent).toBe('shell')
    expect(errorSpy.mock.calls.some(([message]) => (
      typeof message === 'string' && message.includes('[startup] Unhandled window error')
    ))).toBe(false)
  })

  it('uninstalls startup handlers from the first shell mount effect', () => {
    const source = readFileSync(resolve(process.cwd(), 'src/main.tsx'), 'utf8')
    expect(source).toContain('const uninstallStartupErrorHandlers = installStartupErrorHandlers()')
    expect(source).toContain('onMounted={uninstallStartupErrorHandlers}')
    expect(source).toContain('onMounted()')
    expect(source).toContain('[onMounted]')
    expect(source).not.toMatch(/^\s*installStartupErrorHandlers\(\)\s*$/m)
  })
})
