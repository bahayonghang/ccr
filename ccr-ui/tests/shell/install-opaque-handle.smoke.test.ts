import { readFile } from 'node:fs/promises'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

import { llmusageInstallExecute } from '@/api/domains/install'

describe('llmusage install opaque handle boundary', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    invokeMock.mockResolvedValue('22222222-2222-4222-8222-222222222222')
  })

  it('sends only the backend-issued plan id when executing', async () => {
    const planId = '11111111-1111-4111-8111-111111111111'

    await llmusageInstallExecute(planId)

    expect(invokeMock).toHaveBeenCalledWith('llmusage_install_execute', { planId })
  })

  it('keeps executable fields out of the generated plan view', async () => {
    const source = await readFile('src/types/generated/install/InstallPlanView.ts', 'utf8')

    expect(source).not.toMatch(/\bcommand\s*:/)
    expect(source).not.toMatch(/\bargs\s*:/)
    expect(source).not.toMatch(/\benvs\s*:/)
  })
})
