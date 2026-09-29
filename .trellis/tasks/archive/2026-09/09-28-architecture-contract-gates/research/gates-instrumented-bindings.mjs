import { cp, mkdtemp, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { checkBindings } from '../../../../ccr-ui/scripts/check-generated-bindings.mjs'
import { generatedRoot, normalizationStep, runBindingStep } from '../../../../ccr-ui/scripts/generate-bindings.mjs'
const root = await mkdtemp(join(tmpdir(), 'ccr-bindings-stage-'))
let count = 0
const status = await checkBindings({ runStep: async (step) => {
  const code = runBindingStep(step)
  if (code === 0 && step === normalizationStep) await cp(generatedRoot, join(root, String(++count)), { recursive: true })
  return code
} })
await writeFile(new URL('./gates-instrumented-bindings.json', import.meta.url), JSON.stringify({ root, status, normalizationCount: count }, null, 2))
process.stdout.write(JSON.stringify({ root, status, normalizationCount: count }))
process.exitCode = status
