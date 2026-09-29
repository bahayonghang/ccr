import { mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises'
import { join, relative, sep } from 'node:path'

export const snapshotDirectory = async (directory) => {
  const snapshot = { exists: false, directories: [], files: new Map() }
  const relativePath = (path) => relative(directory, path).split(sep).join('/')
  const visit = async (current) => {
    const entries = await readdir(current, { withFileTypes: true })
    for (const entry of entries) {
      const path = join(current, entry.name)
      if (entry.isDirectory()) {
        snapshot.directories.push(relativePath(path))
        await visit(path)
      } else if (entry.isFile()) {
        snapshot.files.set(relativePath(path), await readFile(path))
      } else {
        throw new Error(`Unsupported generated-directory entry: ${path}`)
      }
    }
  }
  try {
    await visit(directory)
    snapshot.exists = true
  } catch (error) {
    // Only an absent root is an empty baseline; disappearing children fail.
    if (error.code !== 'ENOENT' || error.path !== directory) throw error
  }
  return snapshot
}

export const restoreDirectory = async (directory, snapshot) => {
  await rm(directory, { recursive: true, force: true })
  if (!snapshot.exists) return
  await mkdir(directory, { recursive: true })
  for (const path of snapshot.directories) await mkdir(join(directory, path), { recursive: true })
  for (const [path, bytes] of snapshot.files) await writeFile(join(directory, path), bytes)
}

export const diffSnapshots = (before, after) => {
  const paths = new Set([...before.keys(), ...after.keys()])
  return [...paths].sort().filter((path) => {
    const previous = before.get(path)
    const current = after.get(path)
    return !previous || !current || !previous.equals(current)
  })
}

// Direct generation commits only success. A check always restores its caller's bytes.
export const withGeneratedDirectory = async (directory, operation, {
  restoreAlways = false,
  restore = restoreDirectory,
} = {}) => {
  const before = await snapshotDirectory(directory)
  let status = 1
  let failure
  try {
    status = await operation(before)
  } catch (error) {
    failure = error
  }
  if (restoreAlways || failure || status !== 0) {
    try {
      await restore(directory, before)
    } catch (error) {
      throw new AggregateError(
        failure ? [failure, error] : [error],
        `Failed to restore generated bindings at ${directory}: ${error.message}`,
      )
    }
  }
  if (failure) throw failure
  return status
}
