import { describe, expect, it } from 'vitest'
import {
  addConfigFormSchema,
  emptyConfigForm,
  parseTagsInput,
  toUpdateRequest,
  valuesFromConfig,
} from '@/features/configs/lib/configForm'
import { appSettingsSchema } from '@/features/configs/lib/settingsModel'
import { maskSensitive } from '@/utils/logRedact'

describe('config form schema', () => {
  it('rejects add form without name, url, or token', () => {
    const parsed = addConfigFormSchema.safeParse(emptyConfigForm())
    expect(parsed.success).toBe(false)
  })

  it('accepts a complete add form and serializes tags', () => {
    const values = {
      ...emptyConfigForm(),
      name: 'relay',
      base_url: 'https://api.example.com',
      auth_token: 'sk-ant-secret-token',
      tagsInput: 'prod, backup',
    }
    const parsed = addConfigFormSchema.safeParse(values)
    expect(parsed.success).toBe(true)
    const request = toUpdateRequest(values, values.name)
    expect(request.tags).toEqual(['prod', 'backup'])
    expect(parseTagsInput('a, b ,')).toEqual(['a', 'b'])
  })

  it('loads config values without exposing secrets in redacted logs', () => {
    const values = valuesFromConfig({
      name: 'work',
      description: 'desc',
      base_url: 'https://api.example.com',
      auth_token: 'sk-ant-secret-token',
      tags: ['prod'],
    })
    expect(values.auth_token).toBe('sk-ant-secret-token')
    expect(maskSensitive(values.auth_token)).not.toContain('secret')
  })

  it('accepts app settings defaults', () => {
    const parsed = appSettingsSchema.safeParse({
      theme: 'system',
      flavor: 'neutral',
      locale: 'zh-CN',
      uiFont: '',
      codeFont: '',
      uiSelect: '__default__',
      codeSelect: '__default__',
      confirmBeforeExit: true,
      closeToTray: false,
      openPanelOnTrayClick: true,
      sidebarWidth: 240,
      perfTelemetryEnabled: false,
    })
    expect(parsed.success).toBe(true)
  })
})
