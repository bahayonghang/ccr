import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

const landingViewPaths = [
  '../src/views/ClaudeCodeView.vue',
  '../src/views/CodexView.vue',
  '../src/views/GeminiCliView.vue',
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

  it('keeps the Claude Code terminal preview readable in both themes', async () => {
    const claudeViewPath = '../src/views/ClaudeCodeView.vue'
    const absolutePath = fileURLToPath(new URL(claudeViewPath, import.meta.url))
    const source = await readFile(absolutePath, 'utf8')
    const defaultTerminalRule = source.match(/^\.claude-terminal-card\s*\{[\s\S]*?^\}/m)?.[0] ?? ''

    expect(source).toMatch(/--claude-terminal-command:\s*var\(--stage-text-primary\)/)
    expect(source).toMatch(/:global\(\[data-theme='dark'\] \.claude-terminal-card\)/)
    expect(source).toMatch(/:global\(\[data-theme='dark'\] \.claude-terminal-card::before\)/)
    expect(defaultTerminalRule).not.toContain('rgb(10 12 16 / 92%)')
  })
})
