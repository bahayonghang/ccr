import { readdir, readFile } from 'node:fs/promises'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

const TAURI_FACADE_PATH = 'src/api/tauri.ts'

// 允许直接持有 invoke / @tauri-apps/api/core 的位置（API 封装层 + 基础设施豁免）
const INVOKE_ALLOWED_PATHS = [
  'src/api/domains/',
  'src/api/generated/', // registry-owned typed clients; drift is checked by Rust generation tests
  'src/api/runtime/',
  'src/api/tauri.ts', // 兼容门面，命令清单由下方冻结测试守护
  'src/utils/logger.ts', // 日志桥：动态 import 后上报 append_frontend_logs
] as const

const COMPATIBILITY_MARKERS = [
  'Tauri API compatibility facade for CCR Desktop',
  'Compatibility-only',
  'Do not add new business API wrappers or direct `invoke()` calls',
  'New APIs must live in `src/api/domains/*`',
] as const

const ALLOWED_TAURI_FACADE_COMMANDS = [
  'update_config',
  'list_mcp_presets',
  'get_mcp_preset',
  'install_mcp_preset',
  'install_mcp_preset_single',
  'list_source_mcp_servers',
  'sync_mcp_server',
  'sync_all_mcp_servers',
  'health_check',
] as const

const stripTypeScriptComments = (source: string): string => {
  return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:])\/\/.*$/gm, '$1')
}

const extractInvokeCommands = (source: string): string[] => {
  const code = stripTypeScriptComments(source)

  return Array.from(
    code.matchAll(/\binvoke(?:<[^>]+>)?\(\s*['"]([^'"]+)['"]/g),
    (match) => match[1]
  )
}

const walkSourceFiles = async (dir: string): Promise<string[]> => {
  const entries = await readdir(dir, { withFileTypes: true })
  const files: string[] = []

  for (const entry of entries) {
    const fullPath = join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...(await walkSourceFiles(fullPath)))
    } else if (/\.(ts|mts|vue)$/.test(entry.name)) {
      files.push(fullPath)
    }
  }

  return files
}

const toPosix = (path: string): string => path.replace(/\\/g, '/')

describe('API facade boundary', () => {
  it('keeps invoke() usage inside the API layer across all of src/', async () => {
    const files = await walkSourceFiles('src')
    const violations: string[] = []

    for (const file of files) {
      const posixPath = toPosix(file)
      if (INVOKE_ALLOWED_PATHS.some((allowed) => posixPath.startsWith(allowed))) {
        continue
      }

      const source = await readFile(file, 'utf8')
      const code = stripTypeScriptComments(source)
      const importsTauriCore = code.includes('@tauri-apps/api/core')
      const hasInvokeCall = /\binvoke(?:<[^>]+>)?\(/.test(code)

      if (importsTauriCore || hasInvokeCall) {
        violations.push(posixPath)
      }
    }

    expect(
      violations,
      '裸 invoke 必须收敛到 src/api/domains/* 或生成客户端（详见 api-facade-boundary spec）'
    ).toEqual([])
  })
  it('marks tauri.ts as a compatibility-only facade', async () => {
    const source = await readFile(TAURI_FACADE_PATH, 'utf8')

    for (const marker of COMPATIBILITY_MARKERS) {
      expect(source).toContain(marker)
    }
  })

  it('freezes legacy direct invoke calls in tauri.ts', async () => {
    const source = await readFile(TAURI_FACADE_PATH, 'utf8')

    expect(extractInvokeCommands(source)).toEqual(ALLOWED_TAURI_FACADE_COMMANDS)
  })

  it('keeps manifest-typed commands behind generated clients', async () => {
    const manifest = JSON.parse(
      await readFile('src/api/generated/command-manifest.json', 'utf8'),
    ) as { commands: Array<{ id: string; input_schema: string; output_schema: string }> }
    const typedCommands = new Set(
      manifest.commands
        .filter(command => command.input_schema === 'generated' && command.output_schema === 'generated')
        .map(command => command.id),
    )
    const files = await walkSourceFiles('src/api')
    const violations: string[] = []

    for (const file of files) {
      const posixPath = toPosix(file)
      if (posixPath.startsWith('src/api/generated/')) {
        continue
      }
      const commands = extractInvokeCommands(await readFile(file, 'utf8'))
      for (const command of commands) {
        if (typedCommands.has(command)) violations.push(`${posixPath}:${command}`)
      }
    }

    expect(violations, 'manifest-typed commands must be invoked through generated clients').toEqual([])
  })
})
