import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

const landingViewPaths = [
  '../src/features/claude/ClaudeCodeView.tsx',
  '../src/features/codex/CodexView.tsx',
  '../src/features/gemini/GeminiCliView.tsx',
]

const forbiddenStageUtilities = /\btext-white(?:\/|\b)|\bbg-white\/|\bborder-white\//
const requiredStageContract = /stage-page|stage-text-|stage-surface-|stage-border-/

describe('stage landing theme contract', () => {
  it.each(landingViewPaths)('keeps %s on the shared stage theme contract', async (relativePath) => {
    const absolutePath = fileURLToPath(new URL(relativePath, import.meta.url))
    const source = await readFile(absolutePath, 'utf8')

    expect(source).toMatch(requiredStageContract)
    expect(source).not.toMatch(forbiddenStageUtilities)
  })

  it('keeps the Claude Code console readable with semantic tokens', async () => {
    const claudeViewPath = '../src/features/claude/ClaudeCodeView.tsx'
    const absolutePath = fileURLToPath(new URL(claudeViewPath, import.meta.url))
    const source = await readFile(absolutePath, 'utf8')

    expect(source).toMatch(/className="[^"]*claude-console/)
    expect(source).toMatch(/text-text-primary/)
    expect(source).not.toContain('claude-terminal-card')
    expect(source).not.toContain('rgb(10 12 16 / 92%)')
  })
})
