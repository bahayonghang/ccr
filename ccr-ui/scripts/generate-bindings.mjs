import { spawnSync } from 'node:child_process'
import { mkdir, rm } from 'node:fs/promises'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { withGeneratedDirectory } from './bindings-transaction.mjs'

export const generatedRoot = fileURLToPath(new URL('../src/types/generated/', import.meta.url))
const uiRoot = fileURLToPath(new URL('../', import.meta.url))
const cargo = process.platform === 'win32' ? 'cargo.exe' : 'cargo'

export const generationSteps = [
  { command: cargo, args: ['test', '--manifest-path', '../Cargo.toml', '-p', 'ccr-cli', '--features', 'ts', 'export_bindings'], exports: true },
  { command: cargo, args: ['test', '--manifest-path', '../Cargo.toml', '-p', 'ccr-usage', '--features', 'ts', 'export_bindings'], exports: true },
  { command: cargo, args: ['--config', '../.cargo/tauri-ci.toml', 'test', '--manifest-path', 'src-tauri/Cargo.toml', 'export_bindings'], exports: true },
  { command: process.execPath, args: ['./scripts/normalize-generated-bindings.mjs'] },
]
export const normalizationStep = generationSteps[generationSteps.length - 1]

export const runBindingStep = (step) => {
  const result = spawnSync(step.command, step.args, {
    cwd: uiRoot,
    // ts-rs exports share files. Ordinary behavior tests keep their parallelism.
    env: step.exports ? { ...process.env, RUST_TEST_THREADS: '1' } : process.env,
    stdio: 'inherit',
  })
  if (result.error) throw result.error
  return result.status ?? 1
}

export const generateContents = async (directory, runStep = runBindingStep) => {
  await rm(directory, { recursive: true, force: true })
  await mkdir(directory, { recursive: true })
  for (const step of generationSteps) {
    const status = await runStep(step)
    if (status !== 0) return status
  }
  return 0
}

export const generateBindings = ({ directory = generatedRoot, runStep = runBindingStep } = {}) =>
  withGeneratedDirectory(directory, () => generateContents(directory, runStep))

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  generateBindings().then((status) => {
    process.exitCode = status
    if (status === 0) process.stdout.write('TypeScript bindings generated: src/types/generated/\n')
  }).catch((error) => {
    process.stderr.write(`TypeScript bindings generation failed: ${error.message}\n`)
    process.exitCode = 1
  })
}
