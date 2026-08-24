import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

describe('dashboard refresh identity', () => {
  it('does not re-fetch Codex/Grok home on every query object identity change', () => {
    const codex = readFileSync(path.join(root, 'src/features/codex/CodexView.tsx'), 'utf8')
    const grok = readFileSync(path.join(root, 'src/features/grok/GrokView.tsx'), 'utf8')
    expect(codex).not.toContain('void refresh(false)')
    expect(grok).not.toContain('void refresh(false)')
  })
})
