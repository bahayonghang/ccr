import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useUIStore } from '@/stores/ui'
import type {
  PlatformMcpCapability,
  UnifiedMcpListResponse,
  UnifiedMcpServer,
} from '@/types/unifiedMcp'

const apiMocks = vi.hoisted(() => ({
  listUnifiedMcp: vi.fn(),
  addUnifiedMcp: vi.fn(),
  updateUnifiedMcp: vi.fn(),
  deleteUnifiedMcp: vi.fn(),
  toggleUnifiedMcp: vi.fn(),
}))

const loggerMocks = vi.hoisted(() => ({
  error: vi.fn(),
}))

vi.mock('@/api', () => apiMocks)
vi.mock('@/utils/logger', () => ({
  logger: loggerMocks,
}))

import { useUnifiedMcp } from '@/composables/useUnifiedMcp'

const capability = (
  platform: string,
  supportsUrl: boolean
): PlatformMcpCapability => ({
  platform,
  supports_toggle: true,
  supports_url: supportsUrl,
  supports_headers: supportsUrl,
  supports_timeout: supportsUrl,
  supports_cwd: platform === 'claude',
  supports_trust: platform === 'claude',
  supports_include_tools: platform === 'claude',
})

const server = (
  overrides: Partial<UnifiedMcpServer> & Pick<UnifiedMcpServer, 'platform' | 'name'>
): UnifiedMcpServer => ({
  command: 'npx',
  url: null,
  args: [],
  env: {},
  disabled: false,
  scope: 'user',
  effective: true,
  ...overrides,
})

const response = (servers: UnifiedMcpServer[] = []): UnifiedMcpListResponse => ({
  servers,
  capabilities: [capability('claude', true), capability('codex', false)],
  diagnostics: [{ level: 'warning', message: 'shadowed entry' }],
})

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  apiMocks.listUnifiedMcp.mockResolvedValue(response())
  apiMocks.addUnifiedMcp.mockResolvedValue({ message: 'added by backend' })
  apiMocks.updateUnifiedMcp.mockResolvedValue('updated by backend')
  apiMocks.deleteUnifiedMcp.mockResolvedValue({})
  apiMocks.toggleUnifiedMcp.mockResolvedValue({ message: 'toggled by backend' })

  const uiStore = useUIStore()
  vi.spyOn(uiStore, 'showSuccess').mockReturnValue(1)
  vi.spyOn(uiStore, 'showError').mockReturnValue(2)
  vi.spyOn(uiStore, 'showWarning').mockReturnValue(3)
})

describe('useUnifiedMcp coverage', () => {
  it('loads normalized data and evaluates every filter and capability branch', async () => {
    const servers = [
      server({ platform: 'claude', name: 'stdio-user', scope: 'user' }),
      server({
        platform: 'claude',
        name: 'project-http',
        command: null,
        url: 'https://mcp.example.test',
        scope: 'project',
      }),
      server({ platform: 'codex', name: 'local-codex', scope: 'local' }),
      server({
        platform: 'gemini',
        name: 'hidden-gemini',
        hidden_by: 'project',
        effective: false,
      }),
    ]
    apiMocks.listUnifiedMcp.mockResolvedValueOnce(response(servers))

    const state = useUnifiedMcp()
    await state.loadServers('claude')

    expect(apiMocks.listUnifiedMcp).toHaveBeenCalledWith('claude')
    expect(state.loading.value).toBe(false)
    expect(state.error.value).toBeNull()
    expect(state.diagnostics.value).toHaveLength(1)
    expect(state.sourceDiagnostics.value).toBe(state.diagnostics.value)
    expect(state.filteredServers.value.map((item) => item.name)).toEqual([
      'stdio-user',
      'project-http',
      'local-codex',
    ])
    expect(state.scopeCounts.value).toEqual({
      effective: 3,
      local: 1,
      project: 1,
      user: 2,
      hidden: 1,
    })
    expect(state.platformCounts.value).toEqual({ claude: 2, codex: 1, gemini: 1 })
    expect(state.currentCapability.value?.platform).toBe('claude')
    expect(state.supportsFeature('claude', 'supports_headers')).toBe(true)
    expect(state.supportsFeature('gemini', 'supports_headers')).toBe(false)

    state.filterPlatform.value = 'claude'
    expect(state.filteredServers.value).toHaveLength(2)
    state.filterProtocol.value = 'stdio'
    expect(state.filteredServers.value.map((item) => item.name)).toEqual(['stdio-user'])
    state.filterProtocol.value = 'http'
    expect(state.filteredServers.value.map((item) => item.name)).toEqual(['project-http'])
    state.filterProtocol.value = 'all'
    state.filterScope.value = 'project'
    expect(state.filteredServers.value.map((item) => item.name)).toEqual(['project-http'])
    state.filterScope.value = 'hidden'
    state.filterPlatform.value = ''
    expect(state.filteredServers.value.map((item) => item.name)).toEqual(['hidden-gemini'])
    state.filterScope.value = 'effective'
    state.filterKeyword.value = 'MCP.EXAMPLE'
    expect(state.filteredServers.value.map((item) => item.name)).toEqual(['project-http'])
    state.filterKeyword.value = 'NPX'
    expect(state.filteredServers.value.map((item) => item.name)).toEqual([
      'stdio-user',
      'local-codex',
    ])
    expect(state.hasActiveFilters.value).toBe(true)

    state.resetFilters()
    expect(state.hasActiveFilters.value).toBe(false)
    state.formData.value.platform = ''
    expect(state.currentCapability.value).toBeNull()
  })

  it('normalizes malformed list fields and reports Error and non-Error failures', async () => {
    const state = useUnifiedMcp()
    const uiStore = useUIStore()

    apiMocks.listUnifiedMcp.mockResolvedValueOnce({
      servers: null,
      capabilities: {},
      diagnostics: 'bad diagnostics',
    })
    await state.loadServers()
    expect(state.servers.value).toEqual([])
    expect(state.capabilities.value).toEqual([])
    expect(state.diagnostics.value).toEqual([])

    apiMocks.listUnifiedMcp.mockRejectedValueOnce(new Error('offline'))
    await state.loadServers()
    expect(state.error.value).toBe('offline')
    expect(uiStore.showError).toHaveBeenLastCalledWith('加载 MCP 服务器失败: offline')

    apiMocks.listUnifiedMcp.mockRejectedValueOnce('offline string')
    await state.loadServers()
    expect(state.error.value).toBe('Unknown error')
    expect(loggerMocks.error).toHaveBeenCalledTimes(2)
  })

  it('validates add forms and builds platform-specific stdio and HTTP requests', async () => {
    const state = useUnifiedMcp()
    const uiStore = useUIStore()

    state.openAddForm('codex', 'project')
    expect(state.showForm.value).toBe(true)
    expect(state.formData.value.scope).toBeNull()
    expect(await state.submitForm()).toBe(false)
    expect(uiStore.showWarning).toHaveBeenLastCalledWith('服务器名称不能为空')

    state.formData.value.name = 'codex-mcp'
    expect(await state.addServer()).toBe(false)
    expect(uiStore.showWarning).toHaveBeenLastCalledWith('STDIO 模式必须提供 command')

    state.formData.value.command = 'node'
    state.formData.value.url = 'https://discarded.example.test'
    state.formData.value.headers = { Authorization: 'secret' }
    state.formData.value.timeout = 30
    state.formData.value.cwd = 'C:/tmp'
    state.formData.value.trust = true
    state.formData.value.disabled = true
    state.argInput.value = '  --stdio   config.json  '
    state.includeToolInput.value = ' read, write, '
    state.envKey.value = 'TOKEN'
    state.envValue.value = 'value'
    state.addEnvVar()
    state.headerKey.value = 'X-Test'
    state.headerValue.value = 'yes'
    state.addHeader()

    expect(await state.submitForm()).toBe(true)
    expect(apiMocks.addUnifiedMcp).toHaveBeenCalledWith({
      platform: 'codex',
      name: 'codex-mcp',
      scope: null,
      command: 'node',
      url: null,
      args: ['--stdio', 'config.json'],
      env: { TOKEN: 'value' },
      headers: null,
      timeout: null,
      cwd: null,
      trust: null,
      include_tools: null,
      disabled: null,
    })
    expect(uiStore.showSuccess).toHaveBeenCalledWith('added by backend')
    expect(state.showForm.value).toBe(false)

    state.openAddForm('claude', 'local')
    state.formData.value.name = 'http-mcp'
    state.isHttpMode.value = true
    expect(await state.addServer()).toBe(false)
    expect(uiStore.showWarning).toHaveBeenLastCalledWith('HTTP 模式必须提供 url')

    state.formData.value.command = 'discarded-command'
    state.formData.value.url = 'https://http.example.test'
    state.argInput.value = '--discarded'
    apiMocks.addUnifiedMcp.mockResolvedValueOnce({})
    expect(await state.addServer()).toBe(true)
    expect(apiMocks.addUnifiedMcp).toHaveBeenLastCalledWith(
      expect.objectContaining({
        platform: 'claude',
        scope: 'local',
        command: null,
        url: 'https://http.example.test',
        args: null,
      })
    )
    expect(uiStore.showSuccess).toHaveBeenLastCalledWith('添加成功')
  })

  it('edits secrets safely and covers environment/header removal branches', async () => {
    const state = useUnifiedMcp()
    const current = server({
      platform: 'claude',
      name: 'editable',
      scope: 'project',
      args: ['--old'],
      env: { TOKEN: '••••', KEEP: 'plain' },
      headers: { Authorization: '••••', Accept: 'json' },
      include_tools: ['read', 'write'],
      timeout: 10,
      cwd: '/tmp',
      trust: true,
    })

    state.openEditForm(current)
    expect(state.editingServer.value).toMatchObject(current)
    expect(state.argInput.value).toBe('--old')
    expect(state.includeToolInput.value).toBe('read, write')

    state.removeEnvVar('KEEP')
    state.removeHeader('Accept')
    state.removeEnvVar('TOKEN')
    state.removeHeader('Authorization')
    expect(state.formData.value.env).toBeNull()
    expect(state.formData.value.headers).toBeNull()

    state.formData.value.env = { TOKEN: '••••', NEW: 'new-secret' }
    state.formData.value.headers = { Authorization: '••••', 'X-New': 'new-header' }
    state.argInput.value = ''
    state.includeToolInput.value = ''
    expect(await state.updateServer()).toBe(true)

    expect(apiMocks.updateUnifiedMcp).toHaveBeenCalledWith(
      'claude',
      'editable',
      expect.objectContaining({
        scope: 'project',
        args: null,
        include_tools: null,
        env: { NEW: 'new-secret' },
        headers: { 'X-New': 'new-header' },
      })
    )
    expect(state.editingServer.value).toBeNull()
    expect(await state.updateServer()).toBe(false)
  })

  it('covers CRUD success fallbacks, scoped requests, and all rejection messages', async () => {
    const state = useUnifiedMcp()
    const uiStore = useUIStore()
    const target = server({ platform: 'gemini', name: 'target', scope: null, disabled: true })

    expect(await state.deleteServer(target)).toBe(true)
    expect(apiMocks.deleteUnifiedMcp).toHaveBeenCalledWith('gemini', 'target', undefined)
    expect(uiStore.showSuccess).toHaveBeenLastCalledWith('删除成功')

    expect(await state.toggleServer(target)).toBe(true)
    expect(apiMocks.toggleUnifiedMcp).toHaveBeenCalledWith('gemini', 'target', false, undefined)
    expect(uiStore.showSuccess).toHaveBeenLastCalledWith('toggled by backend')

    const scoped = server({ platform: 'claude', name: 'scoped', scope: 'user' })
    apiMocks.deleteUnifiedMcp.mockRejectedValueOnce(new Error('delete denied'))
    expect(await state.deleteServer(scoped)).toBe(false)
    expect(uiStore.showError).toHaveBeenLastCalledWith('删除失败: delete denied')

    apiMocks.toggleUnifiedMcp.mockRejectedValueOnce('toggle denied')
    expect(await state.toggleServer(scoped)).toBe(false)
    expect(uiStore.showError).toHaveBeenLastCalledWith('切换状态失败: Unknown error')

    state.openAddForm()
    state.formData.value.name = 'broken-add'
    state.formData.value.command = 'node'
    apiMocks.addUnifiedMcp.mockRejectedValueOnce(new Error('add denied'))
    expect(await state.addServer()).toBe(false)
    expect(uiStore.showError).toHaveBeenLastCalledWith('添加失败: add denied')

    state.openEditForm(scoped)
    apiMocks.updateUnifiedMcp.mockRejectedValueOnce('update denied')
    expect(await state.updateServer()).toBe(false)
    expect(uiStore.showError).toHaveBeenLastCalledWith('更新失败: Unknown error')

    state.envKey.value = ''
    state.envValue.value = 'ignored'
    state.addEnvVar()
    state.headerKey.value = 'ignored'
    state.headerValue.value = ''
    state.addHeader()
    state.formData.value.env = null
    state.formData.value.headers = null
    state.removeEnvVar('missing')
    state.removeHeader('missing')
    state.closeForm()
    expect(state.showForm.value).toBe(false)
  })
})
