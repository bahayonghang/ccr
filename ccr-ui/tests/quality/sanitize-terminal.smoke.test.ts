import { describe, expect, it } from 'vitest'
import { sanitizeTerminal } from '@/utils/sanitize'

describe('terminal sanitizer smoke', () => {
  it('keeps ansi-safe span markup while stripping unsafe attributes and tags', () => {
    const dirty = '<span class="ansi-red" onclick="alert(1)">hello</span><img src=x onerror="alert(2)"><script>alert(3)</script><br>'
    const sanitized = sanitizeTerminal(dirty)

    expect(sanitized).toContain('<span class="ansi-red">hello</span>')
    expect(sanitized).toContain('<br>')
    expect(sanitized).not.toContain('onclick=')
    expect(sanitized).not.toContain('<img')
    expect(sanitized).not.toContain('<script')
  })
})
