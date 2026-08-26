import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { CodeSourceEditor } from '@/features/editor/CodeSourceEditor'
import { readPageCspNonce } from '@/features/editor/cspNonce'

describe('CodeSourceEditor', () => {
  it('reads the page CSP nonce from a style tag', () => {
    const style = document.createElement('style')
    style.setAttribute('nonce', 'test-nonce')
    document.head.appendChild(style)
    expect(readPageCspNonce()).toBe('test-nonce')
    style.remove()
  })

  it('mounts without throwing', () => {
    const view = render(
      <CodeSourceEditor value='{"ok":true}' language="json" onChange={() => undefined} onSave={() => undefined} />,
    )
    expect(view.container.querySelector('.code-source-editor')).toBeTruthy()
    view.unmount()
  })
})
