import { readFile, readdir } from 'node:fs/promises'
import { spawnSync } from 'node:child_process'
import { join, relative, resolve, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

const generatedRoot = fileURLToPath(new URL('../src/types/generated/', import.meta.url))
const uiRoot = fileURLToPath(new URL('../', import.meta.url))
const normalizer = './scripts/normalize-generated-bindings.mjs'
const writeError = (message) => process.stderr.write(`${message}\n`)
const writeOutput = (message) => process.stdout.write(`${message}\n`)

export const snapshotDirectory = async (directory = generatedRoot) => {
  const files = new Map()
  const relativePath = (path) => relative(directory, path).split(sep).join('/')

  const visit = async (current) => {
    let entries
    try {
      entries = await readdir(current, { withFileTypes: true })
    } catch (error) {
      if (error.code === 'ENOENT') return
      throw error
    }

    for (const entry of entries) {
      const path = join(current, entry.name)
      if (entry.isDirectory()) {
        await visit(path)
        continue
      }
      if (entry.isFile()) files.set(relativePath(path), await readFile(path))
    }
  }

  await visit(directory)
  return files
}

export const diffSnapshots = (before, after) => {
  const paths = new Set([...before.keys(), ...after.keys()])
  return [...paths]
    .sort((left, right) => left.localeCompare(right))
    .filter((path) => {
      const previous = before.get(path)
      const current = after.get(path)
      return !previous || !current || !previous.equals(current)
    })
}

const runCommand = (command, args) => {
  const result = spawnSync(command, args, {
    cwd: uiRoot,
    env: {
      ...process.env,
      RUST_TEST_THREADS: '1',
    },
    stdio: 'inherit',
  })

  if (result.error) throw result.error
  return result.status ?? 1
}

const runNormalizer = () => {
  const command = process.platform === 'win32' ? 'bun.exe' : 'bun'
  const result = spawnSync(command, [normalizer], {
    cwd: uiRoot,
    stdio: 'inherit',
  })

  if (result.error) throw result.error
  return result.status ?? 1
}

const runBindings = () => {
  const command = process.platform === 'win32' ? 'just.exe' : 'just'
  return runCommand(command, ['bindings'])
}

const main = async () => {
  const initial = await snapshotDirectory()
  if (initial.size > 0 && runNormalizer() !== 0) {
    process.exitCode = 1
    return
  }
  const before = await snapshotDirectory()
  const status = runBindings()
  if (status !== 0) {
    process.exitCode = status
    return
  }

  const after = await snapshotDirectory()
  const changed = diffSnapshots(before, after)
  if (changed.length > 0) {
    writeError('❌ TypeScript 绑定漂移：重新生成改变了当前工作区的生成物')
    for (const path of changed) {
      const kind = before.has(path) ? (after.has(path) ? 'M' : 'D') : 'A'
      writeError(`  ${kind} src/types/generated/${path}`)
    }
    writeError('请审阅生成结果，并将 Rust DTO 与生成物一起提交')
    process.exitCode = 1
    return
  }

  writeOutput('✅ TypeScript 绑定与当前工作区生成物同步')
}

const isMain = process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])
if (isMain) {
  main().catch((error) => {
    writeError(`❌ TypeScript 绑定检查失败: ${error.message}`)
    process.exitCode = 1
  })
}
