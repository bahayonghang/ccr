import { beforeEach, describe, expect, it, vi } from 'vitest'

const grokClientMocks = vi.hoisted(() => ({
  getGrokConfigRaw: vi.fn(),
  saveGrokConfigRaw: vi.fn(),
  listGrokConfigLayers: vi.fn(),
  getGrokSettings: vi.fn(),
  updateGrokSettings: vi.fn(),
}))

vi.mock('@/api/generated/grok', () => grokClientMocks)

import {
  getGrokConfigRaw,
  getGrokSettings,
  listGrokConfigLayers,
  saveGrokConfigRaw,
  updateGrokSettings,
} from '@/api/domains/grok'

describe('Grok settings domain API', () => {
  beforeEach(() => {
    for (const mock of Object.values(grokClientMocks)) mock.mockReset()
  })

  it('normalizes unsupported raw and layer responses for ConfigSourcePanel', async () => {
    grokClientMocks.getGrokConfigRaw.mockResolvedValue({
      status: 'unsupported_environment',
      env_type: 'wsl',
    })
    grokClientMocks.listGrokConfigLayers.mockResolvedValue({
      status: 'unsupported_environment',
      env_type: 'ssh',
    })

    await expect(getGrokConfigRaw()).resolves.toEqual({
      status: 'unsupported_environment',
      envType: 'wsl',
    })
    await expect(listGrokConfigLayers()).resolves.toEqual({
      status: 'unsupported_environment',
      envType: 'ssh',
    })
  })

  it('preserves structured invalid markers and read tokens', async () => {
    grokClientMocks.getGrokConfigRaw.mockResolvedValue({
      status: 'ok',
      content: '[ui]\ntheme = "dark"\n',
      token: 'v1',
      path: 'C:/Users/test/.grok/config.toml',
      exists: true,
    })
    grokClientMocks.saveGrokConfigRaw.mockResolvedValue({
      status: 'invalid',
      kind: 'syntax',
      message: 'expected a value',
      line: 2,
      column: 9,
    })

    await expect(getGrokConfigRaw()).resolves.toMatchObject({ status: 'ok', token: 'v1' })
    await expect(saveGrokConfigRaw('invalid', 'v1')).resolves.toEqual({
      status: 'invalid',
      kind: 'syntax',
      message: 'expected a value',
      line: 2,
      column: 9,
    })
    expect(grokClientMocks.saveGrokConfigRaw).toHaveBeenCalledWith('invalid', 'v1')
  })

  it('passes typed patches through the generated client and rejects unknown statuses', async () => {
    const patch = { set: { 'ui.theme': 'dark' }, unset: [] }
    grokClientMocks.getGrokSettings.mockResolvedValue({
      status: 'unsupported_environment',
      env_type: 'wsl',
    })
    grokClientMocks.updateGrokSettings.mockResolvedValue({ status: 'saved' })

    await expect(getGrokSettings()).resolves.toMatchObject({ status: 'unsupported_environment' })
    await expect(updateGrokSettings(patch)).resolves.toEqual({ status: 'saved' })
    expect(grokClientMocks.updateGrokSettings).toHaveBeenCalledWith(patch)

    grokClientMocks.updateGrokSettings.mockResolvedValue({ status: 'unexpected' })
    await expect(updateGrokSettings(patch)).rejects.toThrow('Grok settings update response is invalid')
  })
})
