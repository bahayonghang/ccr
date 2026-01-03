import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const args = process.argv.slice(2)
const isDebug = args.includes('--debug')
const profile = isDebug ? 'debug' : 'release'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const frontendDir = path.resolve(scriptDir, '..')
const projectDir = path.resolve(frontendDir, '..')
const repoRoot = path.resolve(projectDir, '..')
const backendDir = path.join(projectDir, 'backend')
const tauriDir = path.join(frontendDir, 'src-tauri')
const binDir = path.join(tauriDir, 'bin')

const detectHostTriple = () => {
  const result = spawnSync('rustc', ['-vV'], { encoding: 'utf8' })
  if (result.status !== 0) {
    throw new Error(`Failed to run rustc -vV: ${result.stderr || result.stdout}`)
  }
  const hostLine = result.stdout.split(/\r?\n/).find(line => line.startsWith('host: '))
  if (!hostLine) {
    throw new Error('Unable to detect host triple from rustc -vV')
  }
  return hostLine.replace('host: ', '').trim()
}

const resolvedTargetTriple =
  process.env.TAURI_TARGET_TRIPLE || process.env.TARGET || detectHostTriple()

const explicitTargetTriple =
  process.env.TAURI_TARGET_TRIPLE || process.env.TARGET || ''

const run = (command, commandArgs, cwd) => {
  const result = spawnSync(command, commandArgs, {
    cwd,
    stdio: 'inherit',
    shell: false,
  })
  if (result.status !== 0) {
    throw new Error(`${command} ${commandArgs.join(' ')} failed`)
  }
}

const buildArgs = ['build']
if (!isDebug) {
  buildArgs.push('--release')
}
if (explicitTargetTriple) {
  buildArgs.push('--target', explicitTargetTriple)
}

console.log(`[sidecar] Building backend (${profile})...`)
run('cargo', buildArgs, backendDir)

const ext = process.platform === 'win32' ? '.exe' : ''
const binaryName = `ccr-ui-backend${ext}`

const candidateDirs = []
const cargoTargetDir = process.env.CARGO_TARGET_DIR

if (cargoTargetDir) {
  if (explicitTargetTriple) {
    candidateDirs.push(path.join(cargoTargetDir, explicitTargetTriple, profile))
  }
  candidateDirs.push(path.join(cargoTargetDir, profile))
}

if (explicitTargetTriple) {
  candidateDirs.push(path.join(backendDir, 'target', explicitTargetTriple, profile))
}
candidateDirs.push(path.join(backendDir, 'target', profile))

if (explicitTargetTriple) {
  candidateDirs.push(path.join(projectDir, 'target', explicitTargetTriple, profile))
}
candidateDirs.push(path.join(projectDir, 'target', profile))

if (explicitTargetTriple) {
  candidateDirs.push(path.join(repoRoot, 'target', explicitTargetTriple, profile))
}
candidateDirs.push(path.join(repoRoot, 'target', profile))

const sourceBin = candidateDirs
  .map(dir => path.join(dir, binaryName))
  .find(candidate => fs.existsSync(candidate))

if (!sourceBin) {
  const attempted = candidateDirs.map(dir => path.join(dir, binaryName)).join('\n')
  throw new Error(`Backend binary not found. Checked:\n${attempted}`)
}

fs.mkdirSync(binDir, { recursive: true })
const destBin = path.join(binDir, `ccr-ui-backend-${resolvedTargetTriple}${ext}`)
fs.copyFileSync(sourceBin, destBin)

if (process.platform !== 'win32') {
  fs.chmodSync(destBin, 0o755)
}

console.log(`[sidecar] Copied backend -> ${destBin}`)
