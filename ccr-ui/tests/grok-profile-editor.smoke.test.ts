import { describe, expect, it } from 'vitest'
import {
  buildGrokCreateRequest,
  buildGrokPatch,
  createEmptyGrokForm,
  fillGrokForm,
} from '@/utils/grokProfileEditor'
import { validateGrokEditor } from '@/features/grok/profiles/grokEditorValidation'
import { grokProfileFixtures } from './fixtures/profiles'

const t = (key: string) => key

describe('grok profile editor lock', () => {
  it('does not copy display URLs or secrets into the writable form', () => {
    const form = fillGrokForm(grokProfileFixtures[1])
    expect(form.baseUrl).toBe('')
    expect(form.apiKey).toBe('')
    expect(form.envKey).toBe('')
    expect(form.credentialAction).toBe('preserve')
    expect(JSON.stringify(form)).not.toContain('base_url_display')
  })

  it('allows a blank base URL when editing a profile that already has one', () => {
    const form = fillGrokForm(grokProfileFixtures[1])
    form.name = grokProfileFixtures[1].name
    const issues = validateGrokEditor({
      form,
      editingName: form.name,
      hasExistingBaseUrl: true,
      t,
    })
    expect(issues.some((issue) => issue.section === 'connection')).toBe(false)
  })

  it('keeps credential fields mutually exclusive on create', () => {
    const form = createEmptyGrokForm()
    form.name = 'g'
    form.baseUrl = 'https://api.example.com'
    form.credentialAction = 'replace_api_key'
    form.apiKey = 'sk'
    form.envKey = 'SHOULD_NOT'
    const request = buildGrokCreateRequest(form)
    expect(request.api_key).toBe('sk')
    expect(request).not.toHaveProperty('env_key')
  })

  it('emits a reasoning-only patch without URL or credential fields', () => {
    const form = fillGrokForm(grokProfileFixtures[1])
    form.reasoningEffort = 'high'
    const patch = buildGrokPatch(form, new Set(['reasoningEffort']))
    expect(patch).toEqual({ reasoning_effort: 'high' })
    expect(patch).not.toHaveProperty('base_url')
    expect(patch).not.toHaveProperty('api_key')
    expect(patch).not.toHaveProperty('env_key')
  })
})
