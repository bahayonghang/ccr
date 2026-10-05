import { describe, expect, it } from 'vitest'
import { commandsRouteLoaders } from '@/features/commands/routeLoaders'
import { mcpRouteLoaders } from '@/features/mcp/routeLoaders'
import { monitoringRouteLoaders } from '@/features/monitoring/routeLoaders'
import { syncRouteLoaders } from '@/features/sync/routeLoaders'
import { trayRouteLoaders } from '@/features/tray/routeLoaders'

describe('sync-tools route loaders', () => {
  it('exports catalog ids for remaining tool views', () => {
    expect(Object.keys(commandsRouteLoaders)).toEqual(expect.arrayContaining(['commands', 'slash-commands']))
    expect(Object.keys(mcpRouteLoaders)).toEqual(expect.arrayContaining(['mcp-manager']))
    expect(Object.keys(monitoringRouteLoaders)).toEqual(expect.arrayContaining(['monitoring']))
    expect(Object.keys(syncRouteLoaders)).toEqual(
      expect.arrayContaining(['sync', 'wsl-management', 'wsl', 'ssh-management', 'ssh']),
    )
    expect(Object.keys(trayRouteLoaders)).toEqual(expect.arrayContaining(['codex-tray-panel', 'tray/codex']))
  })

  it('resolves each loader to a Component', async () => {
    const loaders = [
      ...Object.values(commandsRouteLoaders),
      ...Object.values(mcpRouteLoaders),
      ...Object.values(monitoringRouteLoaders),
      ...Object.values(syncRouteLoaders),
      ...Object.values(trayRouteLoaders),
    ]
    for (const loader of loaders) {
      const mod = await loader()
      expect(typeof mod.Component).toBe('function')
    }
  })
})
