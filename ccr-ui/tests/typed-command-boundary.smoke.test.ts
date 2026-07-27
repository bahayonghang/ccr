import { readdir, readFile } from 'node:fs/promises'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

interface ManifestCommand {
  id: string
  input_schema: string
  output_schema: string
}

const walkRustFiles = async (dir: string): Promise<string[]> => {
  const entries = await readdir(dir, { withFileTypes: true })
  const files: string[] = []

  for (const entry of entries) {
    const path = join(dir, entry.name)
    if (entry.isDirectory()) files.push(...(await walkRustFiles(path)))
    else if (entry.name.endsWith('.rs')) files.push(path)
  }
  return files
}

describe('typed Rust command boundary', () => {
  it('keeps raw serde_json::Value out of manifest-typed command signatures', async () => {
    const manifest = JSON.parse(
      await readFile('src/api/generated/command-manifest.json', 'utf8'),
    ) as { commands: ManifestCommand[] }
    const typedCommands = new Set(
      manifest.commands
        .filter(command => command.input_schema === 'generated' && command.output_schema === 'generated')
        .map(command => command.id),
    )
    const signatures = new Map<string, { file: string; header: string }>()

    for (const file of await walkRustFiles('src-tauri/src/commands')) {
      const source = await readFile(file, 'utf8')
      for (const match of source.matchAll(
        /#\[ccr_tauri_command_macros::command\](?:\s*#\[[^\]]+\])*\s*pub\s+async\s+fn\s+(\w+)([\s\S]*?)\{/g,
      )) {
        signatures.set(match[1], { file, header: match[2] })
      }
    }

    const missing = [...typedCommands].filter(command => !signatures.has(command))
    const rawValueReturns = [...typedCommands].flatMap((command) => {
      const signature = signatures.get(command)
      if (!signature) return []
      const returnType = signature.header.split('->', 2)[1] ?? ''
      return /\b(?:serde_json::)?Value\b/.test(returnType)
        ? [`${signature.file}:${command}`]
        : []
    })

    expect(missing, 'every manifest-typed command must have an inspectable Rust command signature')
      .toEqual([])
    expect(rawValueReturns, 'manifest-typed commands must return generated DTOs, not raw Value')
      .toEqual([])
  })
})
