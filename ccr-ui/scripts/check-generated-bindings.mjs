import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { diffSnapshots, snapshotDirectory, withGeneratedDirectory } from './bindings-transaction.mjs'
import { generatedRoot, generateContents, normalizationStep, runBindingStep } from './generate-bindings.mjs'

export { diffSnapshots, snapshotDirectory } from './bindings-transaction.mjs'

export const checkBindings = ({
  directory = generatedRoot,
  runStep = runBindingStep,
  writeError = (message) => process.stderr.write(`${message}\n`),
} = {}) => withGeneratedDirectory(directory, async (initial) => {
  if (initial.files.size > 0) {
    const normalized = await runStep(normalizationStep)
    if (normalized !== 0) return normalized
  }
  const before = await snapshotDirectory(directory)
  const status = await generateContents(directory, runStep)
  if (status !== 0) return status

  const after = await snapshotDirectory(directory)
  const changed = diffSnapshots(before.files, after.files)
  if (changed.length > 0) {
    writeError('TypeScript bindings drift: regeneration changed the worktree baseline')
    for (const path of changed) {
      const kind = before.files.has(path) ? (after.files.has(path) ? 'M' : 'D') : 'A'
      writeError(`  ${kind} src/types/generated/${path}`)
    }
    writeError('Run just tauri-bindings from the repository root and review the generated changes')
    return 1
  }
  return 0
}, { restoreAlways: true })

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  checkBindings().then((status) => {
    process.exitCode = status
    if (status === 0) process.stdout.write(`TypeScript bindings match the worktree baseline\n`)
  }).catch((error) => {
    process.stderr.write(`TypeScript bindings check failed: ${error.message}\n`)
    process.exitCode = 1
  })
}
