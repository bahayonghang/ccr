import { describe, expect, it } from 'vitest'
import { systemPromptsConfigs } from '@/features/platform'
import { geminiRouteLoaders } from '@/features/gemini/routeLoaders'
import { opencodeRouteLoaders } from '@/features/opencode/routeLoaders'

describe('generic view reuse', () => {
  it('keeps SystemPrompts configs for five platforms without narrowing optionality', () => {
    expect(Object.keys(systemPromptsConfigs).sort()).toEqual(['claude', 'codex', 'gemini', 'opencode'])
    expect(systemPromptsConfigs.claude.features.hierarchyNote).toBe(true)
    expect(systemPromptsConfigs.gemini.features.geminiNote).toBe(true)
    expect(systemPromptsConfigs.opencode.features.hierarchyNote).toBeUndefined()
  })

  it('exposes AgentDetail and Agents home through gemini loaders for generic routes', () => {
    expect(geminiRouteLoaders['agent-detail']).toBeTypeOf('function')
    expect(geminiRouteLoaders.agents).toBeTypeOf('function')
  })

  it('exposes OpenCode system prompts through the OpenCode loader table', () => {
    expect(opencodeRouteLoaders['opencode-system-prompts']).toBeTypeOf('function')
  })
})
