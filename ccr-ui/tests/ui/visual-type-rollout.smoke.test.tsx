import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { useForm } from 'react-hook-form'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { AgentEditModal, type AgentEditForm } from '@/features/platform/agents/AgentEditModal'
import { McpPresetsPanel } from '@/features/mcp/McpPresetsPanel'

vi.mock('@/api', () => ({
  listMcpPresets: vi.fn(async () => [
    {
      id: 'preset-1',
      name: 'Test Preset',
      description: 'A test preset',
      tags: ['test'],
      requires_api_key: false,
      command: 'npx',
      args: ['-y', 'tool'],
    },
  ]),
  installMcpPreset: vi.fn(),
}))

const root = join(import.meta.dirname, '..', '..')
const PRIMITIVES_CSS_PATH = join(root, 'src/ui/primitives.css')
const TOKENS_CSS_PATH = join(root, 'src/styles/tokens.css')

const UI_CLASSES_PATHS = [
  join(root, 'src/features/codex/ui-classes.ts'),
  join(root, 'src/features/opencode/ui-classes.ts'),
  join(root, 'src/features/grok/ui-classes.ts'),
]

const injectStyle = (css: string): HTMLStyleElement => {
  const style = document.createElement('style')
  style.textContent = css
  document.head.appendChild(style)
  return style
}

beforeAll(() => {
  injectStyle(readFileSync(TOKENS_CSS_PATH, 'utf8'))
  injectStyle(readFileSync(PRIMITIVES_CSS_PATH, 'utf8'))
})

describe('visual type rollout (08-26-visual-type-rollout)', () => {
  it.each(UI_CLASSES_PATHS)('ui-classes.ts no longer exports button class helpers (%s)', (filePath) => {
    const source = readFileSync(filePath, 'utf8')
    expect(source).not.toMatch(/export const primaryBtnClass/)
    expect(source).not.toMatch(/export const ghostBtnClass/)
    expect(source).not.toMatch(/export const secondaryBtnClass/)
    expect(source).not.toMatch(/export const dangerBtnClass/)
  })

  it('BaseCommands and BasePlugins no longer use bg-accent-primary px-4 py-2', () => {
    for (const rel of ['src/features/platform/commands/BaseCommands.tsx', 'src/features/platform/plugins/BasePlugins.tsx']) {
      const source = readFileSync(join(root, rel), 'utf8')
      expect(source).not.toContain('bg-accent-primary px-4 py-2')
    }
  })

  it('AgentEditModal maps save to primary, Add tool to secondary, cancel to ghost', () => {
    function Harness() {
      const form = useForm<AgentEditForm>({
        defaultValues: { model: 'gpt-4', systemPrompt: '', toolDraft: '', toolsText: '' },
      })
      return (
        <AgentEditModal
          open
          name="test-agent"
          saving={false}
          t={(key) => key}
          form={form}
          onClose={() => undefined}
          onSave={() => undefined}
        />
      )
    }

    render(<Harness />)

    expect(screen.getByRole('button', { name: 'common.save' }).className).toContain('ui-btn--primary')
    expect(screen.getByRole('button', { name: 'agents.addTool' }).className).toContain('ui-btn--secondary')
    expect(screen.getByRole('button', { name: 'common.cancel' }).className).toContain('ui-btn--ghost')
  })

  it('McpPresetsPanel confirm install is primary and cancel is ghost', async () => {
    render(<McpPresetsPanel onInstalled={() => undefined} />)

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Test Preset/i })).toBeTruthy()
    })
    fireEvent.click(screen.getByRole('button', { name: /Test Preset/i }))

    expect(screen.getByRole('button', { name: 'common.cancel' }).className).toContain('ui-btn--ghost')
    expect(screen.getByRole('button', { name: 'mcp.presets.confirmInstall' }).className).toContain('ui-btn--primary')
  })
})
