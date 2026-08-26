import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'
import { OC_THEME_VAR_PREFIX, OC_THEME_VARS } from '@/features/opencode/theme/ocThemeVars'

const CORE = join(process.cwd(), 'src/styles/core.css')

describe('opencode theme namespace', () => {
  it('keeps OpenCode theme vars off the CCR @theme namespace', () => {
    const source = readFileSync(CORE, 'utf8')
    expect(source).not.toContain(OC_THEME_VAR_PREFIX)
    for (const name of OC_THEME_VARS) {
      expect(source).not.toContain(name)
    }
  })

  it('prefixes every OpenCode theme render variable with --oc-', () => {
    expect(OC_THEME_VARS.every((name) => name.startsWith(OC_THEME_VAR_PREFIX))).toBe(true)
    const inspector = readFileSync(
      join(process.cwd(), 'src/features/opencode/home/OpenCodeInspector.tsx'),
      'utf8',
    )
    expect(inspector).toContain('OC_THEME_VAR_PREFIX')
    expect(inspector).not.toMatch(/--color-oc-/)
  })
})
