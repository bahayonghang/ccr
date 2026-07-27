import { readdir, readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

const generatedRoot = fileURLToPath(new URL('../src/types/generated/', import.meta.url))

const normalizeDirectory = async (directory) => {
  const entries = await readdir(directory, { withFileTypes: true })

  for (const entry of entries) {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) {
      await normalizeDirectory(path)
      continue
    }
    if (!entry.name.endsWith('.ts')) continue

    const source = await readFile(path, 'utf8')
    const normalized = `${source.replace(/[ \t]+$/gm, '').trimEnd()}\n`
    if (source !== normalized) await writeFile(path, normalized, 'utf8')
  }
}

await normalizeDirectory(generatedRoot)
