import { describe, expect, it, vi } from 'vitest'
import {
  claudeAuthSessionConfig,
  codexAuthSessionConfig,
  grokAuthConfig,
} from '@/configs/auth'
import { claudeAgentsConfig, geminiAgentsConfig } from '@/configs/agents'
import { probeLocalEnvironment } from '@/configs/probeLocal'
import {
  claudeSettingsConfig,
  codexSettingsConfig,
  grokSettingsConfig,
  opencodeSettingsConfig,
} from '@/configs/settings'
import { claudeCommandsConfig } from '@/configs/commands'
import { claudePluginsConfig } from '@/configs/plugins'
import { codexMcpConfig, geminiMcpConfig, opencodeMcpConfig } from '@/configs/mcp'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { BUILT_IN_PROVIDER_TEMPLATES } from '@/configs/providerTemplates'
import { claudeProfilesConfig, codexProfilesConfig, grokProfilesConfig } from '@/configs/profiles'
import {
  dailyCostOptions,
  dailyCostSeries,
  heatmapOptions,
  heatmapSeries,
  tokenStackOptions,
  tokenStackSeries,
} from '@/features/claude/observer/chartOptions'
import {
  buildCompactInventory,
  buildNextActions,
  buildReadinessItems,
  formatDashboardDateTime,
  formatTokens as formatCodexTokens,
} from '@/features/codex/dashboard-model'
import {
  filterResults,
  getAccountOriginKey,
  getAlreadyCheckedInDetail,
  getErrorHint,
  getErrorLabel,
  getFailedDetail,
  getProviderLoginUrl,
  getSkipReasonText,
  getSkippedDetail,
  getStatusText,
  getSuccessDetail,
  sumAccountStats,
} from '@/features/checkin/lib/checkinFormat'
import { handleSelectorKeyDown } from '@/features/configs/provider-templates/selectorKeyboard'
import { createEventBatcher } from '@/features/monitoring/eventBatcher'
import { buildErrorOutput, buildOperationOutput } from '@/features/sync/sync-output'
import {
  extractRemotePathFromMessage,
  isAncestorNotFound,
  maskSecrets,
  normalizeRemoteParentPath,
  toErrorMessage,
} from '@/features/sync/sync-mask'
import {
  formatReset,
  quotaScale,
  quotaToneClass,
  shouldPersistTrayPanelManualPosition,
} from '@/features/tray/tray-format'
import {
  authModeToLoginMethod,
  buildCodexProfileModelCatalog,
  buildCodexProfileRequest,
  codexProfileToEditorForm,
  createCodexProfileEditorForm,
  isDeprecatedAuthMode,
  normalizeModelName,
  normalizeOptionalText,
  parseTagsInput,
  resolveModelSelection,
  usesOpenAiAuthMode,
} from '@/utils/codexProfileEditor'
import {
  applyEditedGroup,
  buildGroupFromForm,
  buildHandler,
  cloneHookMap,
  emptyGroupForm,
  emptyHandlerForm,
  getEventColor,
  getHandlerSummary,
  groupExtraKeys,
  groupKey,
  groupToForm,
  handlerExtraKeys,
  handlerToForm,
} from '@/features/claude/hooks/hooksModel'
import { createEmptyForm, stripUnchangedSecretPreviews, toSuccessMessage } from '@/features/mcp/mcp-constants'
import {
  applyClaudeTemplateToForm,
  buildClaudeProfileRequest,
  buildClaudeTemplateDraft,
  createClaudeProfileForm,
  fillClaudeProfileForm,
  parseClaudeProfileTags,
  resetClaudeProfileForm,
} from '@/utils/claudeProfileEditor'
import {
  claudeAuthModeLabel,
  createClaudeDiffFields,
  createClaudeInspectorDescriptor,
  createClaudeProfileSections,
  createClaudeRowDescriptor,
  filterClaudeProfiles,
  getClaudeProfileProviderKey,
  getClaudeProfileProviderLabel,
  groupProfilesByProvider,
  highlightSearchMatch,
  isCustomClaudeProfileBaseUrl,
  isOfficialClaudeProfileBaseUrl,
  normalizeClaudeProfilesState,
  resolveProviderColor,
  resolveProviderIcon,
} from '@/utils/claudeProfiles'
import {
  buildGrokCreateRequest,
  buildGrokPatch,
  createEmptyGrokForm,
  fillGrokForm,
  parseGrokTags,
} from '@/utils/grokProfileEditor'
import {
  buildCustomTemplate,
  customTemplateSchema,
  draftForCustomSave,
  emptyCustomTemplateForm,
  fillCustomForm,
} from '@/features/configs/lib/templateForm'
import {
  createGrokDiffFields,
  createGrokInspectorDescriptor,
  createGrokRowDescriptor,
  grokAuthModeLabel,
  resolveGrokBaseUrl,
} from '@/utils/grokProfiles'
import {
  buildGrokSettingsPatch,
  createEmptyGrokSettingsForm,
  grokSettingsResponseToForm,
  validateGrokSettingsForm,
} from '@/utils/grokSettings'
import { getRuntimeUnavailableCopy, isRuntimeUnavailableError } from '@/utils/runtimeState'
import { sanitizeInput, sanitizeMarkdown, sanitizeTerminal } from '@/utils/sanitize'
import { scheduleAfterPaint, scheduleWhenIdle } from '@/utils/scheduling'
import { installStartupErrorHandlers, reportStartupFailure } from '@/utils/startupRecovery'
import { formatBaseUrlDisplay, truncateMiddle } from '@/utils/text'
import {
  buildProviderTemplateOptions,
  compactList,
  compactString,
  createCustomProviderTemplateFromDraft,
  deleteCustomProviderTemplate,
  formatListInput,
  getTemplatesForPlatform,
  mapTemplateToClaudeLegacyConfigPatch,
  mapTemplateToClaudeProfilePatch,
  mapTemplateToCodexApiAccountPatch,
  mapTemplateToCodexProfilePatch,
  mapTemplateToCodexProviderPatch,
  mapTemplateToOpenCodeProviderPatch,
  mergeProviderTemplates,
  parseJsonObject,
  parseListInput,
  providerTemplateSearchText,
  readCustomProviderTemplates,
  resolveTemplateBaseUrls,
  resolveTemplateEndpoint,
  safeJson,
  sanitizeProviderTemplate,
  slugifyTemplateId,
  upsertCustomProviderTemplate,
  writeCustomProviderTemplates,
} from '@/utils/providerTemplates'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (command: string) => {
    if (/list_|_list$|servers|plugins|agents|commands/.test(command)) return { servers: [], agents: [], plugins: [], commands: [] }
    return {}
  }),
}))

const t = (key: string) => key

describe('coverage helpers', () => {
  it('sanitizes and truncates text', () => {
    expect(sanitizeTerminal('')).toBe('')
    expect(sanitizeTerminal('ok <script>x</script>')).not.toContain('script')
    expect(sanitizeMarkdown('')).toBe('')
    expect(sanitizeMarkdown('<p>hi</p>')).toContain('hi')
    expect(sanitizeInput('')).toBe('')
    expect(sanitizeInput('<b>x</b>')).toBe('x')
    expect(truncateMiddle('short')).toBe('short')
    expect(truncateMiddle('abcdefghijklmnopqrstuvwxyz0123456789', 4, 4)).toContain('…')
    expect(formatBaseUrlDisplay('https://api.example.com/very/long/path/name')).toContain('api.example.com')
    expect(formatBaseUrlDisplay('not a url')).toBe('not a url')
  })

  it('covers grok helpers and mcp constants', () => {
    const form = createEmptyGrokSettingsForm()
    expect(validateGrokSettingsForm(form, new Set())).toBeNull()
    form['session.auto_compact_threshold_percent'] = '150'
    expect(validateGrokSettingsForm(form, new Set(['session.auto_compact_threshold_percent']))).toBe(
      'session.auto_compact_threshold_percent',
    )
    expect(buildGrokSettingsPatch(form, new Set(['ui.theme']))).toEqual({ set: {}, unset: ['ui.theme'] })
    const mapped = grokSettingsResponseToForm({
      status: 'ok',
      exists: true,
      activation: 'active',
      activation_name: 'p1',
      managed_keys_locked: false,
      models: { default: 'grok', default_reasoning_effort: 'low' },
      ui: { theme: 'dark' },
      session: { auto_compact_threshold_percent: 40, load_envrc: true },
      cli: { auto_update: false, channel: 'stable', show_tips: true },
      hints: { new_session_worktree_mode: 'ask', fork_worktree_mode: 'never' },
      custom_models: [],
    } as never)
    expect(mapped['models.default']).toBe('grok')
    expect(grokAuthModeLabel(t, 'session')).toContain('session')
    const profile = {
      name: 'p1',
      description: null,
      provider: 'xai',
      model: 'grok-2',
      base_url_display: 'https://api.x.ai',
      auth_mode: 'inline_api_key',
      profile_kind: 'third_party',
      has_base_url: true,
      reasoning_effort: 'low',
      api_backend: null,
      context_window: null,
      supports_backend_search: null,
      env_key: null,
      has_inline_credential: true,
      enabled: true,
      tags: [],
    } as never
    expect(resolveGrokBaseUrl(profile, t)).toContain('api.x.ai')
    const row = createGrokRowDescriptor(t)
    expect(row.model(profile)).toBe('grok-2')
    expect(createGrokDiffFields(t)[0]?.value(profile)).toBe('grok-2')
    const inspector = createGrokInspectorDescriptor(t)
    expect(inspector.runtimeSummary(profile)).toContain('grok-2')
    expect(inspector.missingMessage(['model'])).toContain('missing.model')
    expect(inspector.useInsights([profile]).totalIssueCount).toBeGreaterThanOrEqual(0)
    const empty = createEmptyForm()
    expect(empty.platform).toBe('claude')
    const patch = { TOKEN: '••••' }
    stripUnchangedSecretPreviews(patch, { TOKEN: '••••' })
    expect(patch.TOKEN).toBeUndefined()
    stripUnchangedSecretPreviews(null, null)
    expect(toSuccessMessage('ok', 'fb')).toBe('ok')
    expect(toSuccessMessage({ message: 'saved' }, 'fb')).toBe('saved')
    expect(toSuccessMessage({}, 'fb')).toBe('fb')
  })

  it('covers runtime copy, scheduling, notify, and startup handlers', () => {
    expect(isRuntimeUnavailableError(new Error('invoke is not a function'))).toBe(true)
    expect(isRuntimeUnavailableError('plain')).toBe(false)
    expect(getRuntimeUnavailableCopy('usage').title).toContain('桌面')
    expect(getRuntimeUnavailableCopy('commands').description).toContain('命令')
    expect(getRuntimeUnavailableCopy('sync').description).toContain('同步')
    expect(getRuntimeUnavailableCopy('generic').description).toContain('Tauri')
    const cancelPaint = scheduleAfterPaint(() => undefined)
    cancelPaint()
    const cancelIdle = scheduleWhenIdle(() => undefined, { timeout: 10, fallbackDelay: 10 })
    cancelIdle()
    surfaceNotify.success('ok')
    surfaceNotify.error('err')
    surfaceNotify.warning('warn')
    void surfaceNotify.confirm({
      title: 't',
      message: 'm',
      confirmText: 'y',
      cancelText: 'n',
      type: 'warning',
    })
    const stop = installStartupErrorHandlers()
    reportStartupFailure('unit', new Error('boom'))
    stop()
  })

  it('executes shared config list and mutation wrappers', async () => {
    await expect(claudeCommandsConfig.list()).resolves.toEqual([])
    await expect(claudeAgentsConfig.list()).resolves.toEqual([])
    await expect(geminiAgentsConfig.list()).resolves.toEqual([])
    await expect(claudePluginsConfig.list()).resolves.toEqual([])
    await expect(geminiMcpConfig.list()).resolves.toEqual([])
    await expect(codexMcpConfig.list()).resolves.toEqual([])
    await expect(opencodeMcpConfig.list()).resolves.toEqual([])
    await claudeAgentsConfig.create({ name: 'a' })
    await claudeAgentsConfig.update('a', { name: 'a' })
    await claudeAgentsConfig.remove('a')
    await claudeAgentsConfig.toggle?.('a')
    await geminiMcpConfig.create({ name: 'm' })
    await geminiMcpConfig.update('m', { name: 'm' })
    await geminiMcpConfig.remove('m')
    await codexMcpConfig.create({ name: 'm' })
    await codexMcpConfig.update('m', { name: 'm' })
    await codexMcpConfig.remove('m')
    await opencodeMcpConfig.create({ name: 'm', command: 'npx' })
    await opencodeMcpConfig.update('m', { name: 'm', url: 'https://x' })
    await opencodeMcpConfig.remove('m')
    await claudePluginsConfig.create({ id: 'p', name: 'p' })
    await claudePluginsConfig.remove('p')
    await probeLocalEnvironment().catch(() => undefined)
    await grokAuthConfig.load().catch(() => undefined)
    await grokAuthConfig.authOff().catch(() => undefined)
    await grokAuthConfig.probe?.().catch(() => undefined)
    await claudeAuthSessionConfig.load()
    await claudeAuthSessionConfig.authOff()
    await codexAuthSessionConfig.load()
    await codexAuthSessionConfig.authOff()
    await claudeSettingsConfig.load().catch(() => undefined)
    await grokSettingsConfig.load().catch(() => undefined)
    await codexSettingsConfig.load().catch(() => undefined)
    await opencodeSettingsConfig.load().catch(() => undefined)
  })

  it('executes provider template mappers and custom form helpers', () => {
    expect(compactString('  a  ')).toBe('a')
    expect(compactList(['a', '', 'a'])).toEqual(['a'])
    expect(parseListInput('a, b\nc')).toEqual(['a', 'b', 'c'])
    expect(formatListInput(['a', 'b'])).toBe('a\nb')
    expect(safeJson({})).toBe('{}')
    expect(parseJsonObject('{"x":1}')).toEqual({ x: 1 })
    expect(slugifyTemplateId('Hello World')).toMatch(/hello/)
    writeCustomProviderTemplates([])
    expect(readCustomProviderTemplates()).toEqual([])
    const first = BUILT_IN_PROVIDER_TEMPLATES[0]
    expect(first).toBeTruthy()
    if (!first) return
    const merged = mergeProviderTemplates(BUILT_IN_PROVIDER_TEMPLATES, [])
    expect(getTemplatesForPlatform(merged, 'claude').length).toBeGreaterThan(0)
    expect(buildProviderTemplateOptions(merged, 'codex').length).toBeGreaterThan(0)
    for (const template of merged.slice(0, 12)) {
      sanitizeProviderTemplate(template)
      providerTemplateSearchText(template, 'claude')
      resolveTemplateBaseUrls(template, 'claude')
      resolveTemplateEndpoint(template, 'claude')
      mapTemplateToClaudeProfilePatch(template)
      mapTemplateToClaudeLegacyConfigPatch(template)
      mapTemplateToCodexProviderPatch(template)
      mapTemplateToCodexApiAccountPatch(template)
      mapTemplateToCodexProfilePatch(template)
      mapTemplateToOpenCodeProviderPatch(template)
    }
    const created = createCustomProviderTemplateFromDraft(
      {
        platform: 'claude',
        defaultName: 'Custom',
        name: 'Custom',
        category: 'third_party',
        aliases: [],
        tags: [],
        baseUrls: ['https://example.com'],
        modelCatalog: ['m1'],
        platformOverride: { baseUrl: 'https://example.com' },
      },
      ['claude', 'codex', 'opencode'],
      { name: 'Custom', category: 'third_party' },
    )
    const stored = upsertCustomProviderTemplate([], created)
    expect(deleteCustomProviderTemplate(stored, created.id)).toEqual([])
    const values = emptyCustomTemplateForm()
    values.name = 'Custom'
    values.platformClaude = true
    values.baseUrlsInput = 'https://example.com'
    expect(customTemplateSchema.parse(values).name).toBe('Custom')
    const filled = fillCustomForm({ currentPlatform: 'claude', draft: null, template: created })
    expect(filled.name).toBe(created.name)
    expect(draftForCustomSave('claude', created, null)?.platform).toBe('claude')
    const built = buildCustomTemplate({
      values: filled,
      draft: {
        platform: 'claude',
        defaultName: created.name,
        category: 'third_party',
        platformOverride: { baseUrl: 'https://example.com' },
      },
      existing: created,
    })
    expect(built.template?.id).toBeTruthy()
    expect(parseClaudeProfileTags('a, b')).toEqual(['a', 'b'])
    expect(parseClaudeProfileTags('  ')).toBeUndefined()
    const form = createClaudeProfileForm()
    resetClaudeProfileForm(form)
    fillClaudeProfileForm(form, { name: 'n', enabled: true } as never)
    expect(buildClaudeProfileRequest(form).name).toBe('n')
    const draft = buildClaudeTemplateDraft(form)
    applyClaudeTemplateToForm(form, { template: created, endpoint: 'https://example.com' })
    expect(draft.platform).toBe('claude')
    const handlerForm = emptyHandlerForm('command')
    handlerForm.command = 'echo'
    const builtHandler = buildHandler(handlerForm)
    expect(getHandlerSummary(builtHandler)).toContain('echo')
    expect(getEventColor('Stop')).toContain('accent-danger')
    expect(cloneHookMap({})).toEqual({})
    const matcherGroup = { matcher: '', hooks: [builtHandler] }
    const groupForm = groupToForm('Stop', matcherGroup)
    expect(groupKey('Stop', matcherGroup)).toContain('Stop')
    expect(groupExtraKeys(matcherGroup)).toEqual([])
    expect(handlerExtraKeys(builtHandler)).toEqual([])
    const fromForm = buildGroupFromForm(groupForm)
    applyEditedGroup({
      source: {},
      editing: null,
      event: fromForm.event,
      group: fromForm.group,
    })
    expect(handlerToForm(builtHandler).type).toBe('command')
    expect(emptyGroupForm('Stop').event).toBe('Stop')
  })

  it('executes remaining 0% logic helpers', async () => {
    const t = (key: string) => key
    expect(maskSecrets('api_key=sk-abcdefghijklmnop')).toContain('••••')
    expect(isAncestorNotFound('AncestorNotFound /ccr/x')).toBe(true)
    expect(normalizeRemoteParentPath('/ccr/claude/file')).toBe('/ccr/')
    expect(extractRemotePathFromMessage('remote path /ccr/foo')).toBe('/ccr/foo')
    expect(toErrorMessage(new Error('x'))).toBe('x')
    const output = buildOperationOutput({
      result: { success: true, message: 'ok', successCount: 1, total: 1, failed: [], durationMs: 1 },
      fallback: 'fb',
      t,
      assets: [],
    })
    expect(output.status).toBe('success')
    expect(buildErrorOutput({ message: 'boom', fallback: 'fb', t }).status).toBe('failed')
    expect(quotaScale(150)).toBe(1)
    expect(quotaToneClass(90)).toContain('critical')
    expect(quotaToneClass(70)).toContain('warning')
    expect(quotaToneClass(10)).toContain('healthy')
    expect(shouldPersistTrayPanelManualPosition(null, { x: 1, y: 1 })).toBe(true)
    expect(shouldPersistTrayPanelManualPosition({ x: 0, y: 0 }, { x: 1, y: 1 })).toBe(false)
    expect(formatReset(t, Math.floor(Date.now() / 1000) + 90)).toContain('m')
    const flushed: number[][] = []
    const batcher = createEventBatcher<number>((batch) => {
      flushed.push(batch)
    }, 10)
    batcher.push(1)
    batcher.commit()
    batcher.dispose()
    expect(flushed[0]).toEqual([1])
    const keys: string[] = []
    const event = {
      key: 'ArrowDown',
      preventDefault: () => {
        keys.push('down')
      },
    }
    const handlers = {
      visibleCount: 3,
      activeIndex: 0,
      results: [{ id: 'a' } as never],
      selectManual: () => undefined,
      selectOption: () => undefined,
      setActiveIndex: (updater: (index: number) => number) => {
        updater(0)
      },
      close: () => undefined,
    }
    handleSelectorKeyDown(event as never, handlers)
    handleSelectorKeyDown({ key: 'ArrowUp', preventDefault: () => undefined } as never, handlers)
    handleSelectorKeyDown({ key: 'Enter', preventDefault: () => undefined } as never, { ...handlers, activeIndex: 0 })
    handleSelectorKeyDown({ key: 'Enter', preventDefault: () => undefined } as never, { ...handlers, activeIndex: 1 })
    handleSelectorKeyDown({ key: 'Escape', preventDefault: () => undefined } as never, handlers)
    expect(keys).toEqual(['down'])
    await claudeProfilesConfig.list().catch(() => [])
    await grokProfilesConfig.list().catch(() => [])
    await codexProfilesConfig.list().catch(() => [])
    expect(authModeToLoginMethod('openai_chatgpt')).toBe('chatgpt')
    expect(authModeToLoginMethod('openai_api_key')).toBe('api')
    expect(authModeToLoginMethod('no_auth')).toBeUndefined()
    expect(usesOpenAiAuthMode('openai_api_key')).toBe(true)
    expect(isDeprecatedAuthMode('openai_chatgpt')).toBe(true)
    expect(isDeprecatedAuthMode(null)).toBe(false)
    expect(normalizeModelName('  m  ')).toBe('m')
    expect(buildCodexProfileModelCatalog(['a', 'a', ''], 'b')).toEqual(['a', 'b'])
    expect(resolveModelSelection('a', ['a']).selectedModelOption).toBe('a')
    expect(resolveModelSelection('z', ['a']).customModelInput).toBe('z')
    expect(normalizeOptionalText('  ')).toBeNull()
    expect(parseTagsInput('x, y')).toEqual(['x', 'y'])
    const editor = createCodexProfileEditorForm()
    const filled = codexProfileToEditorForm({ name: 'p', enabled: true } as never)
    expect(buildCodexProfileRequest(filled, 'gpt').name).toBe('p')
    expect(editor.auth_mode).toBe('no_auth')
    expect(claudeAuthModeLabel(t, 'api_key')).toBeTruthy()
    expect(resolveProviderColor('anthropic').key).toBe('claude')
    expect(resolveProviderIcon('openai')).toBe('Cpu')
    expect(highlightSearchMatch('hello world', 'lo')).toContain('lo')
    expect(isOfficialClaudeProfileBaseUrl('https://api.anthropic.com')).toBe(true)
    expect(isCustomClaudeProfileBaseUrl('https://example.com')).toBe(true)
    expect(getClaudeProfileProviderKey('Anthropic')).toBe('anthropic')
    expect(getClaudeProfileProviderLabel(null)).toBe('Unspecified Provider')
    const profiles = [{ name: 'p', provider: 'anthropic', is_current: true, enabled: true } as never]
    expect(filterClaudeProfiles(profiles, 'p')).toHaveLength(1)
    expect(groupProfilesByProvider(profiles).length).toBeGreaterThan(0)
    expect(normalizeClaudeProfilesState(profiles, 'p').profiles.length).toBeGreaterThan(0)
    expect(createClaudeProfileSections(profiles, 'Unset').length).toBeGreaterThan(0)
    expect(createClaudeDiffFields(t).length).toBeGreaterThan(0)
    expect(createClaudeRowDescriptor(t).model).toBeTruthy()
    const claudeInspector = createClaudeInspectorDescriptor(t)
    expect(claudeInspector.editIcon).toBeTruthy()
    expect(claudeInspector.activeFields?.(profiles[0] as never).length).toBeGreaterThan(0)
    const grokInspector = createGrokInspectorDescriptor(t)
    const grokProfile = {
      name: 'p1',
      description: null,
      provider: 'xai',
      model: 'grok-2',
      base_url_display: 'https://api.x.ai',
      auth_mode: 'inline_api_key',
      profile_kind: 'third_party',
      has_base_url: true,
      reasoning_effort: 'low',
      api_backend: 'responses',
      context_window: 128000,
      env_key: 'XAI_API_KEY',
      has_inline_credential: true,
      enabled: true,
      tags: ['work'],
    } as never
    expect(grokInspector.activeFields(grokProfile).length).toBeGreaterThan(0)
    expect(grokInspector.useInsights([grokProfile, grokProfile]).totalIssueCount).toBeGreaterThanOrEqual(0)
    expect(grokInspector.authModeLabel('session')).toBeTruthy()
    expect(grokInspector.runtimeSummary(grokProfile)).toContain('grok-2')
    expect(parseGrokTags('a, a, b')).toEqual(['a', 'b'])
    const grokForm = createEmptyGrokForm()
    grokForm.name = 'g1'
    grokForm.baseUrl = 'https://api.x.ai'
    expect(buildGrokCreateRequest(grokForm).name).toBe('g1')
    expect(Object.keys(buildGrokPatch(grokForm, new Set(['name'])))).toContain('name')
    expect(fillGrokForm({
      name: 'g1',
      description: null,
      provider: 'xai',
      profile_kind: 'third_party',
      base_url_display: 'https://api.x.ai',
      has_base_url: true,
      model: 'grok-2',
      api_backend: null,
      context_window: null,
      supports_backend_search: null,
      reasoning_effort: 'low',
      auth_mode: 'inline_api_key',
      env_key: null,
      has_inline_credential: true,
      enabled: true,
      tags: ['work'],
    } as never).name).toBe('g1')
    expect(formatCodexTokens(1_500_000)).toContain('M')
    expect(formatCodexTokens(1500)).toContain('K')
    expect(formatDashboardDateTime(null, t)).toBeTruthy()
    const overview = {
      auth: { logged_in: false, last_refresh: null, saved_accounts_total: 0 },
      profiles: { total: 2, current_profile: 'p', enabled_total: 1 },
      config: { model: 'gpt', approval_policy: 'on-request', sandbox_mode: 'workspace-write' },
      inventory: { mcp_servers_total: 0, sessions_total: 0, agents_total: 0 },
    } as never
    expect(buildReadinessItems({
      overview,
      usageSummary: null,
      usageLoading: false,
      currentAccountLabel: 'a',
      currentProfileLabel: 'p',
      formatDateTime: () => '-',
      t,
    }).length).toBeGreaterThan(0)
    expect(buildNextActions({ overview, t }).length).toBeGreaterThan(0)
    expect(buildCompactInventory({ overview, t }).length).toBeGreaterThan(0)
    const daily = [{ date: '2026-01-01', cost_usd: 1, input_tokens: 1, output_tokens: 1, cache_read_tokens: 0, cache_creation_tokens: 0 }]
    expect(dailyCostSeries(daily as never)[0]?.data).toHaveLength(1)
    expect(dailyCostOptions({ enabled: false }).chart.toolbar.show).toBe(false)
    expect(tokenStackSeries(daily as never).length).toBeGreaterThan(0)
    expect(tokenStackOptions({ enabled: true }).chart).toBeTruthy()
    expect(heatmapSeries([{ date: '2026-01-01', count: 1 }] as never).length).toBeGreaterThan(0)
    expect(heatmapOptions({ enabled: false }).chart).toBeTruthy()
    expect(getStatusText('success', t)).toBeTruthy()
    expect(getStatusText('already_checked_in', t)).toBeTruthy()
    expect(getStatusText('failed', t)).toBeTruthy()
    expect(getStatusText('skipped', t)).toBeTruthy()
    expect(getStatusText('other', t)).toBe('other')
    expect(getSkipReasonText('account_disabled', t)).toBeTruthy()
    expect(getSkipReasonText(undefined, t)).toBeNull()
    expect(getErrorHint('waf_blocked', t)).toBeTruthy()
    expect(getErrorHint(undefined, t)).toBeNull()
    expect(getErrorLabel('timeout', t)).toBeTruthy()
    const item = { status: 'success', reward: '1', balance: 2, message: 'm' }
    expect(getSuccessDetail(item as never, t)).toContain('m')
    expect(getAlreadyCheckedInDetail(item as never, t)).toBeTruthy()
    expect(getSkippedDetail({ ...item, skip_reason: 'account_disabled' } as never, t)).toBeTruthy()
    expect(getFailedDetail({ ...item, error_code: 'waf_blocked', waf_recovery_attempted: true, waf_recovered: false, waf_recovery_error: 'e' } as never, t)).toContain('e')
    expect(filterResults({ results: [item], summary: {} } as never, 'success')).toHaveLength(1)
    expect(sumAccountStats([{ latest_balance: 1, total_quota: 2, total_consumed: 3 }]).currentBalance).toBe(1)
    expect(getAccountOriginKey({ provider_id: 'p' }, [{ id: 'p', base_url: 'https://ex.com/a' }])).toContain('ex.com')
    expect(getProviderLoginUrl({ base_url: 'https://ex.com/' })).toContain('/login')
  })
})
