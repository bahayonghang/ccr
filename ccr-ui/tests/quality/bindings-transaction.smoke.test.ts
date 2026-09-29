// @vitest-environment node
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, describe, expect, it } from 'vitest'
import { snapshotDirectory, withGeneratedDirectory } from '../../scripts/bindings-transaction.mjs'
import { checkBindings } from '../../scripts/check-generated-bindings.mjs'
import { generateBindings, generationSteps, normalizationStep, runBindingStep } from '../../scripts/generate-bindings.mjs'

const roots: string[] = []
const fixture = async (present = true) => {
  const parent = await mkdtemp(join(tmpdir(), 'ccr-bindings-'))
  roots.push(parent)
  const directory = join(parent, 'generated')
  if (present) {
    await mkdir(join(directory, 'nested', 'empty'), { recursive: true })
    await writeFile(join(directory, 'old.ts'), 'old content  \r\n')
    await writeFile(join(directory, 'nested', 'bytes.bin'), Buffer.from([0, 255, 13, 10, 128]))
  }
  return directory
}
afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })))
})

describe('generated bindings transaction', () => {
  it('commits successful direct generation and removes obsolete files', async () => {
    const directory = await fixture()
    const steps: unknown[] = []
    const status = await generateBindings({ directory, runStep: async (step: unknown) => {
      steps.push(step)
      await writeFile(join(directory, 'new.ts'), 'generated')
      return 0
    } })
    expect(status).toBe(0)
    expect(steps).toEqual(generationSteps)
    expect([...(await snapshotDirectory(directory)).files.keys()]).toEqual(['new.ts'])
  })

  it.each(['export', 'normalizer', 'throw', 'spawn'])('restores exact bytes after direct %s failure', async (failure) => {
    const directory = await fixture()
    const before = await snapshotDirectory(directory)
    const result = generateBindings({ directory, runStep: async (step: unknown) => {
      await writeFile(join(directory, 'partial.ts'), 'partial output')
      if (failure === 'throw') throw new Error('export exception')
      if (failure === 'spawn') return runBindingStep({ command: 'ccr-missing-bindings-command-9123', args: [] })
      if (failure === 'normalizer' && step !== normalizationStep) return 0
      return 23
    } })
    if (failure === 'throw' || failure === 'spawn') await expect(result).rejects.toThrow()
    else expect(await result).toBe(23)
    expect(await snapshotDirectory(directory)).toEqual(before)
  })

  it.each([false, true])('restores original root existence after failure (present=%s)', async (present) => {
    const directory = await fixture(present)
    const before = await snapshotDirectory(directory)
    expect(await generateBindings({ directory, runStep: async () => {
      await mkdir(join(directory, 'new', 'nested'), { recursive: true })
      await writeFile(join(directory, 'new', 'nested', 'partial.ts'), 'new')
      return 17
    } })).toBe(17)
    expect(await snapshotDirectory(directory)).toEqual(before)
  })

  it.each(['match', 'drift', 'export', 'normalizer', 'final-normalizer', 'throw', 'spawn'])('check preserves original bytes after %s', async (outcome) => {
    const directory = await fixture()
    const before = await snapshotDirectory(directory)
    let normalizationCount = 0
    const errors: string[] = []
    const result = checkBindings({ directory, writeError: (message: string) => errors.push(message), runStep: async (step: unknown) => {
      if (step === normalizationStep && ++normalizationCount === 1) {
        await writeFile(join(directory, 'old.ts'), 'old content\n')
        if (outcome === 'normalizer') return 19
        if (outcome === 'throw') throw new Error('normalizer exception')
        if (outcome === 'spawn') return runBindingStep({ command: 'ccr-missing-bindings-command-9123', args: [] })
        return 0
      }
      await mkdir(join(directory, 'nested', 'empty'), { recursive: true })
      await writeFile(join(directory, 'old.ts'), outcome === 'drift' ? 'changed' : 'old content\n')
      await writeFile(join(directory, 'nested', 'bytes.bin'), Buffer.from([0, 255, 13, 10, 128]))
      if (outcome === 'export') {
        await rm(join(directory, 'old.ts'))
        await writeFile(join(directory, 'new.ts'), 'partial')
        return 29
      }
      if (outcome === 'final-normalizer' && step === normalizationStep) return 31
      return 0
    } })
    if (outcome === 'throw' || outcome === 'spawn') await expect(result).rejects.toThrow()
    else expect(await result).toBe(outcome === 'match' ? 0 : outcome === 'drift' ? 1 : outcome === 'export' ? 29 : outcome === 'final-normalizer' ? 31 : 19)
    expect(await snapshotDirectory(directory)).toEqual(before)
    if (outcome === 'drift') expect(errors.some((line) => line.includes('M src/types/generated/old.ts'))).toBe(true)
  })

  it('check restores an initially absent directory after drift', async () => {
    const directory = await fixture(false)
    expect(await checkBindings({ directory, writeError: () => {}, runStep: async () => {
      await writeFile(join(directory, 'new.ts'), 'new')
      return 0
    } })).toBe(1)
    expect((await snapshotDirectory(directory)).exists).toBe(false)
  })

  it('reports restoration failure and retains the original operation error', async () => {
    const directory = await fixture()
    const operationError = new Error('export failed')
    const restoreError = new Error('disk unavailable')
    const result = withGeneratedDirectory(directory, () => { throw operationError }, {
      restore: () => { throw restoreError },
    })
    await expect(result).rejects.toThrow('Failed to restore generated bindings')
    await expect(result).rejects.toMatchObject({ errors: [operationError, restoreError] })
    expect(await readFile(join(directory, 'old.ts'), 'utf8')).toBe('old content  \r\n')
  })
})
