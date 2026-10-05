import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { CodexSettingsView } from '@/features/codex/CodexSettingsView'
import { setLocale } from '@/i18n'

const bridge = vi.hoisted(() => ({ invoke: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({ invoke: bridge.invoke }))
vi.mock('@/utils/tauriRuntime', async (original) => ({ ...await original<object>(), isTauriRuntime: () => true }))

beforeEach(async () => {
  bridge.invoke.mockReset().mockImplementation(async (command: string) => {
    if (command === 'get_current_environment') return { id: 'local', env_type: 'local', name: 'Local', available: true }
    if (command === 'codex_get_settings') return { model: 'fixture-model', model_reasoning_effort: 'future-effort' }
    throw new Error('Unexpected fixture command: ' + command)
  })
  await setLocale('zh-CN')
})

describe('Settings visible labels with the real translator', () => {
  it('translates Codex save and known effort options in both locales while preserving an unknown value', async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })
    const { container } = render(<QueryClientProvider client={client}><CodexSettingsView /></QueryClientProvider>)
    await screen.findByDisplayValue('fixture-model')

    for (const [locale, fieldLabel, expected] of [
      ['zh-CN', '推理深度', ['保存', '最低', '低', '中', '高']],
      ['en-US', 'Reasoning Effort', ['Save', 'Minimal', 'Low', 'Medium', 'High']],
    ] as const) {
      await act(async () => { await setLocale(locale) })
      await waitFor(() => {
        const effort = screen.getByLabelText(fieldLabel) as HTMLSelectElement
        expect(effort.value).toBe('future-effort')
        expect(effort.querySelector('option[value="future-effort"]')?.textContent).toBe('future-effort')
        const save = container.querySelector('button[form="platform-settings-form"]')
        const labels = [save?.textContent, ...['minimal', 'low', 'medium', 'high'].map((value) => (
          Array.from(effort.options).find((option) => option.value === value)?.textContent
        ))]
        expect(labels).toEqual(expected)
        expect(labels.join(' ')).not.toMatch(/⟦|codex\.settings\.save/)
      })
    }
    expect(bridge.invoke.mock.calls.filter(([command]) => command === 'codex_get_settings')).toHaveLength(1)
  })
})
