import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  CLAUDE_SECRET_KEYS,
  CODEX_SECRET_KEYS,
  GROK_SECRET_KEYS,
  stripCredentials,
} from '@/configs/profileCredentials'
import { claudeProfileEditorAdapter } from '@/features/claude/profiles/claudeProfileEditorAdapter'
import {
  codexProfileEditorAdapter,
  type CodexEditorForm,
} from '@/features/codex/profiles/codexProfileEditorAdapter'
import { grokProfileEditorAdapter } from '@/features/grok/profiles/grokProfileEditorAdapter'
import type { CodexProfileAuthMode } from '@/types'
import { createEmptyGrokForm, fillGrokForm } from '@/utils/grokProfileEditor'
import {
  claudeProfileFixtures,
  codexProfileFixtures,
  grokProfileFixtures,
} from './fixtures/profiles'

const api = vi.hoisted(() => ({
  addClaudeProfile: vi.fn<(payload: object) => Promise<object>>(),
  updateClaudeProfile: vi.fn<(name: string, payload: object) => Promise<object>>(),
  addCodexProfile: vi.fn<(payload: object) => Promise<object>>(),
  updateCodexProfile: vi.fn<(name: string, payload: object) => Promise<object>>(),
  addGrokProfile: vi.fn<(payload: object) => Promise<{ status: 'created'; profile: { name: string } }>>(),
  updateGrokProfile: vi.fn<(
    name: string,
    payload: object,
  ) => Promise<{ status: 'updated'; profile: { name: string } }>>(),
}))

vi.mock('@/api', () => ({
  claudeApi: {
    addClaudeProfile: api.addClaudeProfile,
    updateClaudeProfile: api.updateClaudeProfile,
  },
  codexApi: {
    addCodexProfile: api.addCodexProfile,
    updateCodexProfile: api.updateCodexProfile,
  },
  grokApi: {
    addGrokProfile: api.addGrokProfile,
    updateGrokProfile: api.updateGrokProfile,
  },
}))

const createCtx = {
  isEditing: false,
  originalName: null,
  existingNames: ['taken'],
  hasExistingBaseUrl: false,
}

const editCtx = {
  isEditing: true,
  originalName: 'kept',
  existingNames: ['kept', 'taken'],
  hasExistingBaseUrl: true,
}

const submitCtx = {
  isEditing: false,
  originalName: null,
  apply: false,
  dirtyFields: new Set<string>(),
}

const asPayload = (value: object): Record<string, unknown> => value as Record<string, unknown>

const issueFields = (issues: readonly { field?: string }[]) =>
  issues.map((issue) => issue.field).filter((field): field is string => Boolean(field))

const codexForm = (overrides: Partial<CodexEditorForm>): CodexEditorForm => ({
  ...codexProfileEditorAdapter.createEmpty(),
  name: 'codex-form',
  model: 'gpt-5.6-sol',
  ...overrides,
})

describe('profile editor adapters', () => {
  beforeEach(() => {
    api.addClaudeProfile.mockReset()
    api.updateClaudeProfile.mockReset()
    api.addCodexProfile.mockReset()
    api.updateCodexProfile.mockReset()
    api.addGrokProfile.mockReset()
    api.updateGrokProfile.mockReset()
    api.addClaudeProfile.mockResolvedValue({})
    api.updateClaudeProfile.mockResolvedValue({})
    api.addCodexProfile.mockResolvedValue({})
    api.updateCodexProfile.mockResolvedValue({})
    api.addGrokProfile.mockResolvedValue({ status: 'created', profile: { name: 'created' } })
    api.updateGrokProfile.mockResolvedValue({ status: 'updated', profile: { name: 'updated' } })
  })

  it('clears secrets in fromRecord for sanitized fixtures', () => {
    const claude = claudeProfileEditorAdapter.fromRecord(
      stripCredentials(claudeProfileFixtures[0], CLAUDE_SECRET_KEYS),
    )
    const codex = codexProfileEditorAdapter.fromRecord(
      stripCredentials(codexProfileFixtures[0], CODEX_SECRET_KEYS),
    )
    const grok = grokProfileEditorAdapter.fromRecord(
      stripCredentials(grokProfileFixtures[1], GROK_SECRET_KEYS),
    )
    expect(claude.auth_token).toBe('')
    expect(codex.auth_token).toBe('')
    expect(grok.apiKey).toBe('')
    expect(grok.envKey).toBe('')
    expect(grok.baseUrl).toBe('')
  })

  it('validates Claude empty name, duplicate name, api_key base URL, and create secret', () => {
    const emptyName = { ...claudeProfileEditorAdapter.createEmpty(), name: '' }
    const duplicate = { ...claudeProfileEditorAdapter.createEmpty(), name: 'taken', base_url: 'https://x', auth_token: 'tok' }
    const missingUrl = {
      ...claudeProfileEditorAdapter.createEmpty(),
      name: 'n',
      auth_mode: 'api_key' as const,
      base_url: '',
      auth_token: 'tok',
    }
    const missingSecret = {
      ...claudeProfileEditorAdapter.createEmpty(),
      name: 'n',
      auth_mode: 'api_key' as const,
      base_url: 'https://x',
      auth_token: '',
    }
    expect(issueFields(claudeProfileEditorAdapter.validate(emptyName, createCtx))).toContain('name')
    expect(issueFields(claudeProfileEditorAdapter.validate(duplicate, createCtx))).toContain('name')
    expect(issueFields(claudeProfileEditorAdapter.validate(missingUrl, createCtx))).toContain('base_url')
    expect(issueFields(claudeProfileEditorAdapter.validate(missingSecret, createCtx))).toContain(
      'auth_token',
    )
    expect(
      issueFields(
        claudeProfileEditorAdapter.validate(
          { ...missingSecret, name: 'kept' },
          { ...editCtx, originalName: 'kept' },
        ),
      ),
    ).not.toContain('auth_token')
  })

  it('validates the Codex auth-mode required-field matrix', () => {
    const cases: Array<{
      mode: CodexProfileAuthMode
      good: Partial<CodexEditorForm>
      bad: Partial<CodexEditorForm>
      badField: string
    }> = [
      {
        mode: 'openai_chatgpt',
        good: { auth_mode: 'openai_chatgpt', model: 'gpt-5.6-sol' },
        bad: { auth_mode: 'openai_chatgpt', model: '' },
        badField: 'model',
      },
      {
        mode: 'openai_api_key',
        good: { auth_mode: 'openai_api_key', auth_token: 'sk', model: 'gpt-5.6-sol' },
        bad: { auth_mode: 'openai_api_key', auth_token: '', model: 'gpt-5.6-sol' },
        badField: 'auth_token',
      },
      {
        mode: 'provider_env_key',
        good: {
          auth_mode: 'provider_env_key',
          base_url: 'https://relay.example',
          auth_token: 'sk',
          env_key: 'MISTRAL_API_KEY',
          model: 'gpt-5.6-sol',
        },
        bad: {
          auth_mode: 'provider_env_key',
          base_url: 'https://relay.example',
          auth_token: 'sk',
          env_key: '',
          model: 'gpt-5.6-sol',
        },
        badField: 'env_key',
      },
      {
        mode: 'provider_bearer_token',
        good: {
          auth_mode: 'provider_bearer_token',
          base_url: 'https://relay.example',
          auth_token: 'sk',
          model: 'gpt-5.6-sol',
        },
        bad: {
          auth_mode: 'provider_bearer_token',
          base_url: '',
          auth_token: 'sk',
          model: 'gpt-5.6-sol',
        },
        badField: 'base_url',
      },
      {
        mode: 'no_auth',
        good: { auth_mode: 'no_auth', base_url: 'https://local', model: 'gpt-5.6-sol' },
        bad: { auth_mode: 'no_auth', base_url: '', model: 'gpt-5.6-sol' },
        badField: 'base_url',
      },
    ]

    for (const item of cases) {
      expect(
        codexProfileEditorAdapter.validate(codexForm(item.good), createCtx),
        `${item.mode} good`,
      ).toEqual([])
      expect(
        issueFields(codexProfileEditorAdapter.validate(codexForm(item.bad), createCtx)),
        `${item.mode} bad`,
      ).toContain(item.badField)
    }
  })

  it('serializes Codex env_key and bearer derived fields only in their modes', async () => {
    await codexProfileEditorAdapter.submit(
      codexForm({
        auth_mode: 'provider_env_key',
        base_url: 'https://relay.example',
        auth_token: 'sk',
        env_key: 'MISTRAL_API_KEY',
      }),
      submitCtx,
    )
    const envPayload = asPayload(api.addCodexProfile.mock.calls[0][0])
    expect(envPayload.env_key).toBe('MISTRAL_API_KEY')
    expect(envPayload.preferred_auth_method).toBeNull()
    expect(envPayload.forced_login_method).toBeNull()

    await codexProfileEditorAdapter.submit(
      codexForm({
        auth_mode: 'provider_bearer_token',
        base_url: 'https://relay.example',
        auth_token: 'sk',
        env_key: 'SHOULD_NOT',
      }),
      submitCtx,
    )
    const bearerPayload = asPayload(api.addCodexProfile.mock.calls[1][0])
    expect(bearerPayload.env_key).toBeNull()
    expect(bearerPayload.preferred_auth_method).toBe('apikey')
    expect(bearerPayload.forced_login_method).toBe('api')

    await codexProfileEditorAdapter.submit(
      codexForm({
        auth_mode: 'openai_api_key',
        auth_token: 'sk',
        env_key: 'SHOULD_NOT',
      }),
      submitCtx,
    )
    const openAiPayload = asPayload(api.addCodexProfile.mock.calls[2][0])
    expect(openAiPayload.env_key).toBeNull()
    expect(openAiPayload.preferred_auth_method).toBeNull()
  })

  it('serializes Grok credential actions mutually exclusively', async () => {
    const base = { ...createEmptyGrokForm(), name: 'g', model: 'grok-4.6', baseUrl: 'https://x' }
    await grokProfileEditorAdapter.submit(
      { ...base, credentialAction: 'replace_api_key', apiKey: 'sk', envKey: 'ENV' },
      submitCtx,
    )
    const apiKeyReq = asPayload(api.addGrokProfile.mock.calls[0][0])
    expect(apiKeyReq.api_key).toBe('sk')
    expect(apiKeyReq).not.toHaveProperty('env_key')

    await grokProfileEditorAdapter.submit(
      { ...base, credentialAction: 'replace_env_key', apiKey: 'sk', envKey: 'GROK_KEY' },
      submitCtx,
    )
    const envReq = asPayload(api.addGrokProfile.mock.calls[1][0])
    expect(envReq.env_key).toBe('GROK_KEY')
    expect(envReq).not.toHaveProperty('api_key')

    await grokProfileEditorAdapter.submit(
      { ...base, credentialAction: 'preserve', apiKey: 'sk', envKey: 'ENV' },
      submitCtx,
    )
    const preserveReq = asPayload(api.addGrokProfile.mock.calls[2][0])
    expect(preserveReq).not.toHaveProperty('api_key')
    expect(preserveReq).not.toHaveProperty('env_key')

    await grokProfileEditorAdapter.submit(
      { ...base, credentialAction: 'clear', apiKey: 'sk', envKey: 'ENV' },
      submitCtx,
    )
    const clearReq = asPayload(api.addGrokProfile.mock.calls[3][0])
    expect(clearReq.credential_action).toBe('clear')
    expect(clearReq).not.toHaveProperty('api_key')
    expect(clearReq).not.toHaveProperty('env_key')
  })

  it('patches only reasoning effort and never serializes base_url_display', async () => {
    const form = fillGrokForm(grokProfileFixtures[1])
    form.reasoningEffort = 'high'
    await grokProfileEditorAdapter.submit(form, {
      isEditing: true,
      originalName: form.name,
      apply: false,
      dirtyFields: new Set(['reasoningEffort']),
    })
    const patch = asPayload(api.updateGrokProfile.mock.calls[0][1])
    expect(patch).toEqual({ reasoning_effort: 'high' })
    expect(JSON.stringify(patch)).not.toContain('base_url_display')
    expect(JSON.stringify(api.addGrokProfile.mock.calls)).not.toContain('base_url_display')
  })

  it('omits blank secrets on edit submit', async () => {
    await claudeProfileEditorAdapter.submit(
      {
        ...claudeProfileEditorAdapter.fromRecord(
          stripCredentials(claudeProfileFixtures[0], CLAUDE_SECRET_KEYS),
        ),
        auth_token: '',
      },
      { isEditing: true, originalName: 'claude-current', apply: false, dirtyFields: new Set() },
    )
    const claudePayload = asPayload(api.updateClaudeProfile.mock.calls[0][1])
    expect(claudePayload).not.toHaveProperty('auth_token')

    await codexProfileEditorAdapter.submit(
      {
        ...codexProfileEditorAdapter.fromRecord(
          stripCredentials(codexProfileFixtures[0], CODEX_SECRET_KEYS),
        ),
        auth_token: '',
      },
      { isEditing: true, originalName: 'codex-current', apply: false, dirtyFields: new Set() },
    )
    const codexPayload = asPayload(api.updateCodexProfile.mock.calls[0][1])
    expect(codexPayload).not.toHaveProperty('auth_token')

    const grokForm = fillGrokForm(grokProfileFixtures[1])
    await grokProfileEditorAdapter.submit(grokForm, {
      isEditing: true,
      originalName: grokForm.name,
      apply: false,
      dirtyFields: new Set(['description']),
    })
    const grokPatch = asPayload(api.updateGrokProfile.mock.calls[0][1])
    expect(grokPatch).not.toHaveProperty('api_key')
    expect(grokPatch).not.toHaveProperty('env_key')
  })

  it('hides Grok official connection fields and keeps profile_kind read-only', () => {
    const official = grokProfileEditorAdapter.fromRecord(grokProfileFixtures[0])
    const fields = grokProfileEditorAdapter.sections.flatMap((section) => section.fields)
    const hidden = fields
      .filter((field) => field.visible && !field.visible(official))
      .map((field) => field.key)
    expect(hidden).toEqual(
      expect.arrayContaining(['provider', 'baseUrl', 'credentialAction', 'apiKey', 'envKey']),
    )
    expect(fields.find((field) => field.key === 'profileKind')?.readOnly).toBe(true)
  })
})
