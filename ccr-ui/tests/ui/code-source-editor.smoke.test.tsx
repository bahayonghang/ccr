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

  it('mounts with a readable runtime style carrying the page CSP nonce', () => {
    const style = document.createElement('style')
    style.setAttribute('nonce', 'editor-test-nonce')
    document.head.appendChild(style)
    const view = render(
      <CodeSourceEditor value='{"ok":true}' language="json" onChange={() => undefined} onSave={() => undefined} />,
    )
    expect(view.container.querySelector('.code-source-editor')).toBeTruthy()
    const editorStyle = [...document.querySelectorAll('style')].find((candidate) => candidate.textContent?.includes('.cm-scroller'))
    expect(editorStyle?.nonce).toBe('editor-test-nonce')
    expect(editorStyle?.sheet).toBeTruthy()
    view.unmount()
    style.remove()
  })
})
