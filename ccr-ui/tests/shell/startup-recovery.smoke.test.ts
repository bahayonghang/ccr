import { afterEach, describe, expect, it } from 'vitest'
import { renderFatalStartup } from '@/utils/startupRecovery'

describe('startup recovery smoke', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('renders a fatal startup placeholder into #app', () => {
    document.body.innerHTML = '<div id="app"></div>'

    renderFatalStartup('Router initialization: boom')

    const mountNode = document.querySelector('#app')
    expect(mountNode?.textContent).toContain('CCR Desktop failed to finish startup')
    expect(mountNode?.textContent).toContain('Router initialization: boom')
  })
})
