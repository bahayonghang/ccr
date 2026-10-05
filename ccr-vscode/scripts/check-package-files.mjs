import { spawnSync } from 'node:child_process'
import { createRequire } from 'node:module'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const require = createRequire(import.meta.url)
// Resolve the ZIP reader from the declared packaging tool, not a hoisted package.
const vsceRequire = createRequire(require.resolve('@vscode/vsce/package.json'))
const { open } = vsceRequire('yauzl')
const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')

export const packageFiles = Object.freeze([
  'package.json',
  'README.md',
  'CHANGELOG.md',
  'LICENSE',
  'icon.png',
  'dist/extension.js',
  'resources/icons/ccr.svg',
  'resources/icons/claude.svg',
  'resources/icons/codex.svg',
  'resources/icons/droid.svg',
  'resources/icons/gemini.svg',
  'resources/icons/iflow.svg',
  'resources/icons/qwen.svg',
])

const renamedFiles = new Map([
  ['README.md', 'readme.md'],
  ['CHANGELOG.md', 'changelog.md'],
  ['LICENSE', 'LICENSE.txt'],
])
const vsixFiles = [
  'extension.vsixmanifest',
  '[Content_Types].xml',
  ...packageFiles.map(file => 'extension/' + (renamedFiles.get(file) ?? file)),
]

export function validatePackageFiles(files, { vsix = false } = {}) {
  const expected = new Set(vsix ? vsixFiles : packageFiles)
  const seen = new Set()
  const errors = []
  for (const file of files) {
    if (!expected.has(file)) errors.push('Unexpected package entry: ' + file)
    if (seen.has(file)) errors.push('Duplicate package entry: ' + file)
    seen.add(file)
  }
  for (const file of expected) {
    if (!seen.has(file)) errors.push('Missing package entry: ' + file)
  }
  if (errors.length > 0) throw new Error(errors.join('\n'))
}

export function listPackageFiles(cwd = packageRoot) {
  const result = spawnSync(process.execPath, [
    require.resolve('@vscode/vsce/vsce'), 'ls', '--no-dependencies',
  ], { cwd, encoding: 'utf8' })
  if (result.error) throw result.error
  if (result.status !== 0) {
    throw new Error('vsce ls failed: ' + (result.stderr || result.stdout || result.signal))
  }
  return result.stdout.split(/\r?\n/).filter(Boolean)
}

export async function readVsixEntries(filePath) {
  return await new Promise((resolveEntries, reject) => {
    open(filePath, { lazyEntries: true, strictFileNames: true }, (error, archive) => {
      if (error) return reject(error)
      const entries = []
      archive.on('error', archiveError => {
        archive.close()
        reject(archiveError)
      })
      archive.on('entry', entry => {
        entries.push(entry.fileName)
        archive.readEntry()
      })
      archive.on('end', () => resolveEntries(entries))
      archive.readEntry()
    })
  })
}

async function main(args) {
  if (args.length === 0) {
    const files = listPackageFiles()
    validatePackageFiles(files)
    console.log('Package file check passed: ' + files.length + ' runtime files')
    return
  }
  if (args.length !== 2 || args[0] !== '--vsix') {
    throw new Error('Usage: node scripts/check-package-files.mjs [--vsix <path>]')
  }
  const files = await readVsixEntries(resolve(args[1]))
  validatePackageFiles(files, { vsix: true })
  console.log('VSIX file check passed: ' + files.length + ' archive entries')
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch(error => {
    console.error(error.message)
    process.exitCode = 1
  })
}
