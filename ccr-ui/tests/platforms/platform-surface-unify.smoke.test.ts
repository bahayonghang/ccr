import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'
import { settingsConfigs } from '@/configs/settings'
import {
  platformHasSurface,
  platformSurfaceDescriptors,
  PLATFORM_SURFACES,
} from '@/config/platformDescriptors'
import { flattenCatalog } from '@/shell/routeCatalog'
import { saveSettingsValues, visibleSettingsFields } from '@/features/platform'

const SRC = join(process.cwd(), 'src')

const walk = (dir: string, acc: string[] = []): string[] => {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry)
    if (statSync(full).isDirectory()) {
      walk(full, acc)
      continue
    }
    if (full.endsWith('.ts') || full.endsWith('.tsx')) acc.push(full)
  }
  return acc
}

const PLATFORM_COMPARE = /(?:platform|id)\s*===?\s*['"](?:claude|codex|grok|opencode|gemini|antigravity|claude-code)['"]/

const THIN_SHELLS = [
  'src/features/claude/ClaudeSettingsView.tsx',
  'src/features/claude/ClaudeProfilesView.tsx',
  'src/features/claude/ClaudePluginsView.tsx',
  'src/features/claude/ClaudeCommandsView.tsx',
  'src/features/claude/ClaudeAgentsView.tsx',
  'src/features/codex/CodexSettingsView.tsx',
  'src/features/codex/CodexProfilesView.tsx',
  'src/features/codex/CodexMcpView.tsx',
  'src/features/codex/CodexAgentsView.tsx',
  'src/features/grok/GrokSettingsView.tsx',
  'src/features/grok/GrokProfilesView.tsx',
  'src/features/grok/GrokAuthView.tsx',
  'src/features/opencode/OpenCodeSettingsView.tsx',
  'src/features/opencode/OpenCodeCommandsView.tsx',
  'src/features/opencode/OpenCodeMcpView.tsx',
  'src/features/opencode/OpenCodeAgentsView.tsx',
  'src/features/opencode/OpenCodePluginsView.tsx',
  'src/features/gemini/GeminiMcpView.tsx',
  'src/features/gemini/GeminiAgentsView.tsx',
  'src/features/gemini/GeminiPluginsView.tsx',
]

describe('platform unify', () => {
  it('keeps flattenCatalog at 76 paths', () => {
    expect(flattenCatalog()).toHaveLength(76)
  })

  it('declares surfaces without changing root paths', () => {
    expect(PLATFORM_SURFACES).toEqual([
      'settings',
      'profiles',
      'auth',
      'mcp',
      'agents',
      'plugins',
      'commands',
    ])
    expect(platformSurfaceDescriptors.claude.rootPath).toBe('/claude-code')
    expect(platformSurfaceDescriptors.codex.rootPath).toBe('/codex')
    expect(platformSurfaceDescriptors.grok.rootPath).toBe('/grok')
    expect(platformSurfaceDescriptors.opencode.rootPath).toBe('/opencode')
    expect(platformSurfaceDescriptors.gemini.rootPath).toBe('/antigravity')
    expect(platformHasSurface('grok', 'auth')).toBe(true)
    expect(platformHasSurface('grok', 'mcp')).toBe(false)
  })

  it('forbids platform name branches in features/platform', () => {
    const files = walk(join(SRC, 'features/platform')).filter((file) => {
      const name = file.split(/[/\\]/).pop() ?? ''
      return (
        name.startsWith('Base') ||
        name.endsWith('-model.ts') ||
        file.includes(`${join('platform', 'settings')}`) ||
        file.includes(`${join('platform', 'profiles')}`)
      )
    })
    const componentFiles = walk(join(SRC, 'components', 'profiles'))
    const hits: string[] = []
    for (const file of [...files, ...componentFiles]) {
      const text = readFileSync(file, 'utf8')
      if (PLATFORM_COMPARE.test(text)) hits.push(file.split('\\').join('/'))
    }
    expect(hits).toEqual([])
  })

  it('keeps thin shells at or below 100 lines', () => {
    for (const rel of THIN_SHELLS) {
      const lines = readFileSync(join(process.cwd(), rel), 'utf8').split('\n').length
      expect(lines, rel).toBeLessThanOrEqual(100)
    }
  })

  it('routes all settings configs through one visibleSettingsFields implementation', () => {
    const configs = Object.values(settingsConfigs)
    expect(configs.length).toBe(4)
    for (const config of configs) {
      expect(visibleSettingsFields(config).length).toBeGreaterThan(0)
    }
    const source = readFileSync(join(SRC, 'features/platform/settings/BaseSettings.tsx'), 'utf8')
    expect(source).toContain('saveSettingsValues')
    expect(source).toContain('fieldsForTab')
  })

  it('applies a shared save helper to every settings config', async () => {
    const save = async () => undefined
    const config = {
      ...settingsConfigs.claude,
      save,
      fields: settingsConfigs.claude.fields,
    }
    await saveSettingsValues(config, { model: 'sonnet' }, ['model'])
    const shells = [
      'src/features/claude/ClaudeSettingsView.tsx',
      'src/features/codex/CodexSettingsView.tsx',
      'src/features/grok/GrokSettingsView.tsx',
      'src/features/opencode/OpenCodeSettingsView.tsx',
    ]
    for (const rel of shells) {
      const text = readFileSync(join(process.cwd(), rel), 'utf8')
      expect(text).toContain('BaseSettings')
      expect(text).not.toContain('saveSettingsValues')
    }
  })
})
