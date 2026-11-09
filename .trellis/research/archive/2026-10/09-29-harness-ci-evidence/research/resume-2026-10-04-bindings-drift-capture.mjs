import { spawnSync } from 'node:child_process'
import { mkdtemp, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { diffSnapshots, snapshotDirectory, withGeneratedDirectory } from '../../../../ccr-ui/scripts/bindings-transaction.mjs'
import { generateContents, generatedRoot, normalizationStep, runBindingStep } from '../../../../ccr-ui/scripts/generate-bindings.mjs'

const outputDir = dirname(fileURLToPath(import.meta.url))
const diffPath = join(outputDir, 'resume-2026-10-04-bindings-drift.diff')
const summaryPath = join(outputDir, 'resume-2026-10-04-bindings-drift.json')

const textOf = (bytes) => bytes ? bytes.toString('utf8') : ''
const whitespaceOnly = (before, after) => textOf(before).replace(/\s/g, '') === textOf(after).replace(/\s/g, '')

const status = await withGeneratedDirectory(generatedRoot, async (initial) => {
  if (initial.files.size > 0) {
    const normalized = await runBindingStep(normalizationStep)
    if (normalized !== 0) return normalized
  }
  const before = await snapshotDirectory(generatedRoot)
  const generated = await generateContents(generatedRoot, runBindingStep)
  if (generated !== 0) return generated
  const after = await snapshotDirectory(generatedRoot)
  const changed = diffSnapshots(before.files, after.files)
  const temporary = await mkdtempCompat()
  const sections = []
  const files = []
  for (const path of changed) {
    const previous = before.files.get(path)
    const current = after.files.get(path)
    const label = `ccr-ui/src/types/generated/${path}`
    files.push({
      path: label,
      kind: previous ? (current ? 'M' : 'D') : 'A',
      whitespace_only: Boolean(previous && current && whitespaceOnly(previous, current)),
    })
    const left = join(temporary, 'before')
    const right = join(temporary, 'after')
    await writeFile(left, previous ?? Buffer.alloc(0))
    await writeFile(right, current ?? Buffer.alloc(0))
    const result = spawnSync('python', ['-c', [
      'import difflib, pathlib, sys',
      'left, right, label = sys.argv[1:]',
      'a = pathlib.Path(left).read_bytes().decode("utf-8").splitlines(True)',
      'b = pathlib.Path(right).read_bytes().decode("utf-8").splitlines(True)',
      'sys.stdout.writelines(difflib.unified_diff(a, b, fromfile="a/"+label, tofile="b/"+label))',
    ].join('\n'), left, right, label], { encoding: 'utf8' })
    if (result.status !== 0) throw new Error(result.stderr || `diff failed for ${label}`)
    sections.push(result.stdout)
  }
  await writeFile(diffPath, sections.join(''))
  await writeFile(summaryPath, JSON.stringify({
    date_utc: new Date().toISOString(),
    changed_count: changed.length,
    files,
    whitespace_only_count: files.filter((file) => file.whitespace_only).length,
    diff: 'resume-2026-10-04-bindings-drift.diff',
    product_bytes: 'restored by the bindings transaction',
  }, null, 2) + '\n')
  process.stdout.write(`BINDINGS_DIFF ${changed.length}\n`)
  return 0
}, { restoreAlways: true })

if (status !== 0) {
  process.stderr.write(`BINDINGS_CAPTURE_FAILED ${status}\n`)
  process.exitCode = status
}

function mkdtempCompat() {
  return mkdtemp(join(tmpdir(), 'ccr-bindings-drift-'))
}
