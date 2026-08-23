import { readdir, readFile } from 'node:fs/promises'
import { join } from 'node:path'

const SOURCE_FILE_RE = /\.(ts|mts|tsx|vue)$/

const stripTypeScriptComments = (source: string): string => {
  return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:])\/\/.*$/gm, '$1')
}

export const extractInvokeCommands = (source: string): string[] => {
  const code = stripTypeScriptComments(source)

  return Array.from(
    code.matchAll(/\binvoke(?:<[^>]+>)?\(\s*['"]([^'"]+)['"]/g),
    (match) => match[1],
  )
}

export const walkSourceFiles = async (dir: string): Promise<string[]> => {
  const entries = await readdir(dir, { withFileTypes: true })
  const files: string[] = []

  for (const entry of entries) {
    const fullPath = join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...(await walkSourceFiles(fullPath)))
    } else if (SOURCE_FILE_RE.test(entry.name)) {
      files.push(fullPath)
    }
  }

  return files
}

export const collectInvokeCommandsFromDir = async (dir: string): Promise<Set<string>> => {
  const commands = new Set<string>()
  for (const file of await walkSourceFiles(dir)) {
    for (const command of extractInvokeCommands(await readFile(file, 'utf8'))) {
      commands.add(command)
    }
  }
  return commands
}
