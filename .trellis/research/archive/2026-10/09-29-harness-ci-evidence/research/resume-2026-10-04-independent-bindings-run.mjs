import { createHash } from 'node:crypto'
import { writeFile } from 'node:fs/promises'
import { dirname, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const root = resolve(here, '../../../..')
const load = (name) => import(pathToFileURL(resolve(root, 'ccr-ui/scripts', name)).href)
const { checkBindings, diffSnapshots, snapshotDirectory } = await load('check-generated-bindings.mjs')
const { generatedRoot, normalizationStep, runBindingStep } = await load('generate-bindings.mjs')
const initial = await snapshotDirectory(generatedRoot)
let normalizedBefore
let generatedAfter
let normalizedCalls = 0
const steps = []
const started = new Date().toISOString()
const status = await checkBindings({ runStep: async (step) => {
  const exitCode = runBindingStep(step)
  steps.push({ command: step.command, args: step.args, export_suite: Boolean(step.exports), exit_code: exitCode })
  if (step === normalizationStep && exitCode === 0) {
    const snapshot = await snapshotDirectory(generatedRoot)
    if (normalizedCalls++ === 0) normalizedBefore = snapshot
    else generatedAfter = snapshot
  }
  return exitCode
} })
const restored = await snapshotDirectory(generatedRoot)
const restoredChanges = diffSnapshots(initial.files, restored.files)
const changed = normalizedBefore && generatedAfter ? diffSnapshots(normalizedBefore.files, generatedAfter.files) : []
const hashes = (snapshot) => Object.fromEntries([...snapshot.files].map(([name, bytes]) => [name, createHash('sha256').update(bytes).digest('hex')]))
const selected = {}
for (const name of changed) {
  selected[name] = {}
  for (const [phase, snapshot] of [['before', normalizedBefore], ['after', generatedAfter]]) {
    const bytes = snapshot.files.get(name)
    const target = `resume-2026-10-04-independent-bindings-${phase}-${name.replaceAll('/', '-')}`
    await writeFile(resolve(here, target), bytes, { flag: 'wx' })
    selected[name][phase] = { evidence: target, bytes: bytes.length, sha256: createHash('sha256').update(bytes).digest('hex') }
  }
}
const receipt = { started_utc: started, finished_utc: new Date().toISOString(), gate_exit_code: status,
  official_check: 'checkBindings with runBindingStep wrapper; restoreAlways is unchanged', steps,
  initial_file_count: initial.files.size, restored_file_count: restored.files.size,
  initial_sha256: hashes(initial), restored_sha256: hashes(restored), restored_changes: restoredChanges,
  directory_shape_restored: JSON.stringify(initial.directories) === JSON.stringify(restored.directories),
  normalized_comparison_changed: changed, selected,
  boundary: 'Approved diagnostic capture. Generated root original bytes restored by official transaction. No retained product binding update.' }
await writeFile(resolve(here, 'resume-2026-10-04-independent-bindings-capture.json'), JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' })
if (restoredChanges.length || !receipt.directory_shape_restored) throw new Error('Generated root restoration failed')
process.stdout.write(JSON.stringify({ gate_exit_code: status, changed, restored_changes: restoredChanges }) + '\n')
process.exitCode = status
