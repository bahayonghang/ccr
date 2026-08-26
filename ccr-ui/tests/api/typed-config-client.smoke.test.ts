import { readFile } from 'node:fs/promises'
import { describe, expect, it } from 'vitest'

const invokeCommands = (source: string): string[] =>
  Array.from(source.matchAll(/\binvoke\(\s*['"]([^'"]+)['"]/g), (match) => match[1])

describe('generated config client', () => {
  it('owns every typed config invoke without generic escape hatches', async () => {
    const source = await readFile('src/api/generated/config.ts', 'utf8')

    expect(invokeCommands(source)).toEqual([
      'list_configs',
      'switch_config',
      'add_config',
      'delete_config',
      'rename_config',
      'duplicate_config',
      'validate_configs',
      'import_config',
      'restore_config',
      'export_config',
      'get_history',
      'clear_history',
    ])
    expect(source).not.toMatch(/<T\b|\bunknown\b|\bany\b/)
  })

  it('keeps the backend wire names for config mutation arguments', async () => {
    const source = await readFile('src/api/generated/config.ts', 'utf8')

    expect(source).toContain("invoke('add_config', input)")
    expect(source).toContain("invoke('duplicate_config', { source, target })")
    expect(source).toContain("invoke('export_config', { includeSecrets })")
    expect(source).toContain("invoke('import_config', { content: input.content, mode: input.mode ?? 'merge', backup: input.backup ?? true, confirmationToken:")
    expect(source).toContain("invoke('restore_config', { backupPath, confirmationToken:")
  })

  it('keeps migrated config invokes out of handwritten domain wrappers', async () => {
    const source = await readFile('src/api/domains/config.ts', 'utf8')

    expect(source).not.toMatch(/invoke\(['"](?:list|switch|add|delete|rename|duplicate|validate|import|restore|export)_config/)
    expect(source).not.toMatch(/invoke\(['"](?:get|clear)_history/)
  })
})
