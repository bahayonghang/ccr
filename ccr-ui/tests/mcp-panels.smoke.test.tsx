import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { useForm } from 'react-hook-form'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import {
  McpCreatePanel,
  McpDetailPanel,
  McpImportPanel,
  McpListPanel,
  parseMcpImportJson,
} from '@/features/mcp'
import type { McpGroup } from '@/types/mcpManager'
import type { PlatformMeta, UnifiedMcpPlatform, UnifiedMcpRequest } from '@/types/unifiedMcp'
import type { TranslateFunction } from '@/utils/tf'

const t: TranslateFunction = (key, values) => {
  if (!values) return key
  return Object.entries(values).reduce(
    (text, [name, value]) => text.replace(`{${name}}`, String(value)),
    key,
  )
}

const platformMeta: Record<string, PlatformMeta> = {
  claude: { id: 'claude', label: 'Claude Code', color: '#d97706', icon: 'terminal' },
  codex: { id: 'codex', label: 'Codex', color: '#10b981', icon: 'code' },
  gemini: { id: 'gemini', label: 'Antigravity CLI', color: '#8b5cf6', icon: 'sparkles' },
}

const platforms: UnifiedMcpPlatform[] = ['claude', 'codex', 'gemini']

const sampleGroup: McpGroup = {
  name: 'filesystem',
  transportType: 'stdio',
  transportLabel: 'npx -y @modelcontextprotocol/server-filesystem',
  items: [
    {
      platform: 'claude',
      name: 'filesystem',
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-filesystem'],
      env: { API_KEY: 'sk-secret-value' },
      disabled: false,
      effective: true,
      scope: 'user',
      source_path: '/home/user/.claude.json',
    },
  ],
  platforms: ['claude'],
  scopes: ['user'],
  hiddenCount: 0,
}

beforeAll(() => {
  class ResizeObserverStub {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  }
  if (typeof globalThis.ResizeObserver === 'undefined') {
    globalThis.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver
  }

  const mouseEventCtor = (globalThis.MouseEvent ?? Event) as unknown as typeof MouseEvent
  class PointerEventStub extends mouseEventCtor {
    readonly pointerId: number
    readonly pointerType: string
    readonly isPrimary: boolean

    constructor(type: string, params: PointerEventInit = {}) {
      super(type, {
        bubbles: params.bubbles,
        cancelable: params.cancelable,
        button: params.button ?? 0,
        ctrlKey: params.ctrlKey ?? false,
      })
      this.pointerId = params.pointerId ?? 0
      this.pointerType = params.pointerType ?? 'mouse'
      this.isPrimary = params.isPrimary ?? true
    }
  }
  if (typeof globalThis.PointerEvent === 'undefined') {
    const stub = PointerEventStub as unknown as typeof PointerEvent
    globalThis.PointerEvent = stub
    window.PointerEvent = stub
  }

  if (typeof Element.prototype.scrollIntoView !== 'function') {
    Element.prototype.scrollIntoView = () => {}
  }
})

describe('features/mcp 共享面板（批次 3 前半）', () => {
  it('从 @/features/mcp 导出四个面板与 parseMcpImportJson', () => {
    expect(typeof McpListPanel).toBe('function')
    expect(typeof McpDetailPanel).toBe('function')
    expect(typeof McpCreatePanel).toBe('function')
    expect(typeof McpImportPanel).toBe('function')
    expect(typeof parseMcpImportJson).toBe('function')
  })

  it('McpListPanel 空列表渲染 empty 文案，点击列表项触发 onSelect', () => {
    const onSelect = vi.fn()
    const onCreate = vi.fn()
    const { rerender } = render(
      <McpListPanel
        groups={[]}
        searchQuery=""
        selectedKeys={new Set()}
        isMultiSelectMode={false}
        loading={false}
        t={t}
        onSearchQueryChange={vi.fn()}
        onSelect={onSelect}
        onCreate={onCreate}
        onImport={vi.fn()}
        onRefresh={vi.fn()}
        onToggleMultiSelect={vi.fn()}
        onBulkDelete={vi.fn()}
      />,
    )
    expect(screen.getByText('mcp.manager.list.empty')).toBeTruthy()

    rerender(
      <McpListPanel
        groups={[sampleGroup]}
        searchQuery=""
        selectedKeys={new Set(['filesystem'])}
        isMultiSelectMode={false}
        loading={false}
        t={t}
        onSearchQueryChange={vi.fn()}
        onSelect={onSelect}
        onCreate={onCreate}
        onImport={vi.fn()}
        onRefresh={vi.fn()}
        onToggleMultiSelect={vi.fn()}
        onBulkDelete={vi.fn()}
      />,
    )
    fireEvent.click(screen.getByRole('button', { name: /filesystem/ }))
    expect(onSelect).toHaveBeenCalledWith('filesystem')
  })

  it('McpDetailPanel 空态与详情；密钥被掩码；edit/delete/toggle 回调', () => {
    const onEdit = vi.fn()
    const onDelete = vi.fn()
    const onToggle = vi.fn()
    const { rerender } = render(
      <McpDetailPanel group={null} t={t} onEdit={onEdit} onDelete={onDelete} onToggle={onToggle} />,
    )
    expect(screen.getByText('mcp.manager.detail.emptyTitle')).toBeTruthy()

    rerender(
      <McpDetailPanel
        group={sampleGroup}
        t={t}
        onEdit={onEdit}
        onDelete={onDelete}
        onToggle={onToggle}
      />,
    )
    expect(screen.getByText('filesystem')).toBeTruthy()
    expect(screen.queryByText('sk-secret-value')).toBeNull()
    expect(screen.getByText(/sk-s••••ue/)).toBeTruthy()

    fireEvent.click(screen.getByRole('button', { name: 'common.edit' }))
    expect(onEdit).toHaveBeenCalledWith('filesystem')
    fireEvent.click(screen.getByRole('button', { name: 'common.delete' }))
    expect(onDelete).toHaveBeenCalledWith(sampleGroup)
    fireEvent.click(screen.getByRole('button', { name: 'claude:filesystem' }))
    expect(onToggle).toHaveBeenCalledTimes(1)
  })

  it('McpCreatePanel 使用 formApi.register，提交走 onSubmit', () => {
    const onSubmit = vi.fn()
    function Harness() {
      const formApi = useForm<UnifiedMcpRequest>({
        defaultValues: {
          platform: 'claude',
          name: '',
          scope: 'user',
          command: '',
          url: null,
        },
      })
      const formData = formApi.watch()
      return (
        <McpCreatePanel
          isEditing={false}
          formApi={formApi}
          formData={formData}
          isHttpMode={false}
          argInput=""
          envKey=""
          envValue=""
          headerKey=""
          headerValue=""
          platforms={platforms}
          platformMeta={platformMeta}
          t={t}
          onSubmit={onSubmit}
          onCancel={vi.fn()}
          onIsHttpModeChange={vi.fn()}
          onArgInputChange={vi.fn()}
          onEnvKeyChange={vi.fn()}
          onEnvValueChange={vi.fn()}
          onHeaderKeyChange={vi.fn()}
          onHeaderValueChange={vi.fn()}
          onAddEnv={vi.fn()}
          onRemoveEnv={vi.fn()}
          onAddHeader={vi.fn()}
          onRemoveHeader={vi.fn()}
        />
      )
    }

    render(<Harness />)
    expect(screen.getByText('mcp.manager.form.addTitle')).toBeTruthy()
    fireEvent.change(screen.getByPlaceholderText('my-mcp-server'), {
      target: { value: 'my-server' },
    })
    fireEvent.submit(screen.getByPlaceholderText('my-mcp-server').closest('form') as HTMLFormElement)
    expect(onSubmit).toHaveBeenCalledTimes(1)
  })

  it('McpImportPanel 解析 JSON 预览并 onImport', async () => {
    const onImport = vi.fn()
    render(
      <McpImportPanel
        platforms={platforms}
        platformMeta={platformMeta}
        t={t}
        onCancel={vi.fn()}
        onImport={onImport}
      />,
    )

    const textarea = screen.getByPlaceholderText(/mcpServers/)
    fireEvent.change(textarea, {
      target: {
        value: '{"mcpServers":{"demo":{"command":"npx","args":["-y","demo"]}}}',
      },
    })
    await waitFor(() => {
      expect(screen.getByText('demo')).toBeTruthy()
    })
    fireEvent.click(screen.getByRole('button', { name: /mcp.manager.import.submit/ }))
    expect(onImport).toHaveBeenCalledWith(
      [{ name: 'demo', type: 'stdio', command: 'npx', args: ['-y', 'demo'] }],
      'claude',
      'user',
    )
  })

  it('parseMcpImportJson 覆盖空串、非法 JSON、缺 command/url', () => {
    expect(parseMcpImportJson('', t)).toEqual({ servers: [], error: '' })
    expect(parseMcpImportJson('{', t).error).toBe('mcp.manager.import.errors.invalidJson')
    expect(parseMcpImportJson('[]', t)).toEqual({ servers: [], error: '' })
    expect(parseMcpImportJson('null', t).error).toBe('mcp.manager.import.errors.invalidFormat')
    expect(parseMcpImportJson('{"mcpServers":{"x":{}}}', t).error).toBe(
      'mcp.manager.import.errors.missingCommandOrUrl',
    )
    const ok = parseMcpImportJson(
      '{"mcpServers":{"http":{"url":"http://localhost:3000","headers":{"Auth":"t"}}}}',
      t,
    )
    expect(ok.error).toBe('')
    expect(ok.servers[0]?.type).toBe('http')
    expect(ok.servers[0]?.url).toBe('http://localhost:3000')
    expect(ok.servers[0]?.headers).toEqual({ Auth: 't' })
  })
})
