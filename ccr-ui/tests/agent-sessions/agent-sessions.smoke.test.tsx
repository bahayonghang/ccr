import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { AgentSessionProviderStrip } from '@/features/agent-sessions/AgentSessionProviderStrip'
import { AgentSessionTranscript } from '@/features/agent-sessions/AgentSessionTranscript'
import { resolveActiveArchiveId } from '@/features/agent-sessions/AgentSessionPageStates'
import { AGENT_SESSION_AGENTS } from '@/features/agent-sessions/model'
import { mainLayoutNavSections } from '@/config/mainLayoutShell'
import { flattenCatalog } from '@/shell/routeCatalog'
import type { AgentSessionAgentDto } from '@/types/generated/agent_sessions/AgentSessionAgentDto'
import type { AgentSessionListItemDto } from '@/types/generated/agent_sessions/AgentSessionListItemDto'
import type { TranslateFunction } from '@/utils/tf'

const t: TranslateFunction = (key) => key

describe('Agent Sessions workspace', () => {
  it('keeps all eight provider families visible and independently selectable', () => {
    const onToggle = vi.fn<(agent: AgentSessionAgentDto) => void>()
    render(<AgentSessionProviderStrip statuses={[]} selectedAgents={[]} pending={false} failed={false} t={t} onToggle={onToggle} />)

    expect(screen.getAllByRole('button')).toHaveLength(8)
    expect(AGENT_SESSION_AGENTS).toEqual([
      'grok',
      'claude',
      'codex',
      'opencode',
      'pi',
      'omp',
      'antigravity',
      'kimi',
    ])
    fireEvent.click(screen.getByRole('button', { name: /agentSessions\.agents\.omp/ }))
    expect(onToggle).toHaveBeenCalledWith('omp')
  })

  it('places the independent route immediately above MCP Manager', () => {
    const workspace = mainLayoutNavSections.find((section) => section.id === 'workspace')
    expect(workspace?.items.map((item) => item.to)).toEqual(['/agent-sessions', '/mcp-manager'])
    expect(flattenCatalog().some((route) => route.path === '/agent-sessions' && route.id === 'agent-sessions')).toBe(true)
    expect(flattenCatalog().some((route) => route.path === '/agents' && route.id === 'agents')).toBe(true)
  })

  it('keeps transcript locators private and virtualizes both long surfaces', () => {
    const root = resolve(process.cwd(), 'src')
    const listItemDto = readFileSync(resolve(root, 'types/generated/agent_sessions/AgentSessionListItemDto.ts'), 'utf8')
    const detailDto = readFileSync(resolve(root, 'types/generated/agent_sessions/AgentSessionDetailDto.ts'), 'utf8')
    const listSource = readFileSync(resolve(root, 'features/agent-sessions/AgentSessionList.tsx'), 'utf8')
    const transcriptSource = readFileSync(resolve(root, 'features/agent-sessions/AgentSessionTranscript.tsx'), 'utf8')

    expect(listItemDto).not.toContain('file_path')
    expect(listItemDto).not.toContain('source_member_id')
    expect(detailDto).not.toContain('file_path')
    expect(detailDto).not.toContain('source_member_id')
    expect(listSource).toContain('useVirtualizer')
    expect(transcriptSource).toContain('useVirtualizer')
    expect(listSource).toContain('virtualizer.measureElement')
    expect(transcriptSource).toContain('virtualizer.measureElement')
  })

  it('does not render raw source validation codes in the transcript empty state', () => {
    const session: AgentSessionListItemDto = {
      archive_id: 'as-demo',
      session_id: 'sess-demo',
      agent: 'codex',
      variant: 'codex-live',
      cwd: '',
      message_count: 0,
      user_message_count: 0,
      assistant_message_count: 0,
      tool_use_count: 0,
      created_at: '2026-09-02T00:00:00.000Z',
      updated_at: '2026-09-02T00:00:00.000Z',
      source_state: 'live',
      fidelity: 'full',
    }
    const translate: TranslateFunction = (key) => {
      const catalog: Record<string, string> = {
        'agentSessions.missing': 'Source missing',
        'agentSessions.sourceUnavailableDescription': 'This session source is no longer on disk.',
        'agentSessions.error': 'Error',
        'agentSessions.errors.generic': 'Generic session error',
        'agentSessions.errors.agent_session_source_validation_failed': 'Source failed validation',
        'common.retry': 'Retry',
      }
      return catalog[key] ?? key
    }
    const onRetry = vi.fn()
    const { rerender } = render(
      <AgentSessionTranscript
        session={session}
        details={[]}
        locale="en-US"
        pending={false}
        error="agent_session_source_unavailable"
        hasOlder={false}
        fetchingOlder={false}
        t={translate}
        onLoadOlder={() => undefined}
        onRetry={onRetry}
      />,
    )

    expect(screen.queryByText('agent_session_source_validation_failed')).toBeNull()
    expect(screen.queryByText('agent_session_source_unavailable')).toBeNull()
    expect(screen.getByText('Source missing')).toBeTruthy()
    expect(screen.getByText('This session source is no longer on disk.')).toBeTruthy()
    fireEvent.click(screen.getByRole('button', { name: 'Retry' }))
    expect(onRetry).toHaveBeenCalledTimes(1)

    rerender(
      <AgentSessionTranscript
        session={session}
        details={[]}
        locale="en-US"
        pending={false}
        error="agent_session_source_validation_failed"
        hasOlder={false}
        fetchingOlder={false}
        t={translate}
        onLoadOlder={() => undefined}
        onRetry={onRetry}
      />,
    )
    expect(screen.queryByText('agent_session_source_validation_failed')).toBeNull()
    expect(screen.queryByText('Generic session error')).toBeNull()
    expect(screen.getByText('Source failed validation')).toBeTruthy()
  })

  it('auto-selects the first readable session instead of a missing source', () => {
    const missing: AgentSessionListItemDto = {
      archive_id: 'as-missing',
      session_id: 'missing',
      agent: 'codex',
      variant: 'codex-live',
      cwd: '',
      message_count: 0,
      user_message_count: 0,
      assistant_message_count: 0,
      tool_use_count: 0,
      created_at: '2026-03-06T10:36:00.000Z',
      updated_at: '2026-03-06T10:36:00.000Z',
      source_state: 'missing',
      fidelity: 'full',
    }
    const live: AgentSessionListItemDto = {
      ...missing,
      archive_id: 'as-live',
      session_id: 'live',
      agent: 'claude',
      source_state: 'live',
      message_count: 214,
    }
    expect(resolveActiveArchiveId([missing, live], '')).toBe('as-live')
    expect(resolveActiveArchiveId([missing, live], 'as-missing')).toBe('as-missing')
    expect(resolveActiveArchiveId([missing, live], '', new Set(['as-live']))).toBe('')
  })

  it('starts incremental refresh when the local Agent Sessions page mounts', () => {
    const source = readFileSync(resolve(process.cwd(), 'src/features/agent-sessions/AgentSessionsView.tsx'), 'utf8')
    expect(source).toContain('bootstrapRefreshRef')
    expect(source).toContain('refreshMutation.mutate()')
  })
})
