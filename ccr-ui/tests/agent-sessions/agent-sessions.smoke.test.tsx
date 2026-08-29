import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { AgentSessionProviderStrip } from '@/features/agent-sessions/AgentSessionProviderStrip'
import { AGENT_SESSION_AGENTS } from '@/features/agent-sessions/model'
import { mainLayoutNavSections } from '@/config/mainLayoutShell'
import { flattenCatalog } from '@/shell/routeCatalog'
import type { AgentSessionAgentDto } from '@/types/generated/agent_sessions/AgentSessionAgentDto'
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
})
