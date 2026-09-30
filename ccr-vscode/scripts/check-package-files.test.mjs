import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { createWriteStream } from 'node:fs'
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import { createRequire } from 'node:module'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { pipeline } from 'node:stream/promises'
import { test } from 'node:test'
import { fileURLToPath } from 'node:url'
import { listPackageFiles, readVsixEntries, validatePackageFiles } from './check-package-files.mjs'

const require = createRequire(import.meta.url)
const vsceRequire = createRequire(require.resolve('@vscode/vsce/package.json'))
const { ZipFile } = vsceRequire('yazl')
const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const requiredFiles = [
  'package.json', 'README.md', 'CHANGELOG.md', 'LICENSE', 'icon.png', 'dist/extension.js',
  ...['ccr', 'claude', 'codex', 'droid', 'gemini', 'iflow', 'qwen'].map(name => 'resources/icons/' + name + '.svg'),
]
const archiveFiles = [
  'extension.vsixmanifest', '[Content_Types].xml',
  ...requiredFiles.map(file => 'extension/' + ({
    'README.md': 'readme.md', 'CHANGELOG.md': 'changelog.md', LICENSE: 'LICENSE.txt',
  }[file] ?? file)),
]
const forbiddenFiles = [
  '.serena/project.local.yml', '.serena/project.yml', '.serena/.gitignore',
  '.abcoder/config.json', '.claude/settings.local.json', '.codex/config.toml',
  '.grok/settings.json', '.kimi-code/config.toml', '.omp/state.json', '.omc/state.json',
  'settings.local.json', 'resources/icons/custom.local.svg', 'dist/extension.js.map',
  'AGENTS.md', 'CLAUDE.md', 'code_map.md', 'src/extension.ts', 'dist/unexpected.js',
]

async function tempDirectory(t) {
  const directory = await mkdtemp(join(tmpdir(), 'ccr-vsix-fixture-'))
  t.after(() => rm(directory, { recursive: true, force: true }))
  return directory
}

async function writeFixtureFile(directory, file, content = 'Synthetic package fixture') {
  const filePath = join(directory, file)
  await mkdir(dirname(filePath), { recursive: true })
  await writeFile(filePath, content)
}

async function createArchive(directory, files) {
  const filePath = join(directory, 'fixture.vsix')
  const archive = new ZipFile()
  for (const file of files) archive.addBuffer(Buffer.from('Synthetic fixture'), file)
  const finished = pipeline(archive.outputStream, createWriteStream(filePath))
  archive.end()
  await finished
  return filePath
}

test('vsce ls keeps required files and excludes synthetic local state and source maps', async t => {
  const directory = await tempDirectory(t)
  await copyFile(join(packageRoot, '.vscodeignore'), join(directory, '.vscodeignore'))
  for (const file of [...requiredFiles, ...forbiddenFiles]) await writeFixtureFile(directory, file)
  await writeFixtureFile(directory, 'package.json', JSON.stringify({
    name: 'package-fixture', version: '1.0.0', publisher: 'fixture',
    engines: { vscode: '^1.85.0' }, main: './dist/extension.js',
    contributes: { commands: [{ command: 'fixture.run', title: 'Fixture' }] },
  }))
  const files = listPackageFiles(directory)
  assert.deepEqual(files.toSorted(), requiredFiles.toSorted())
  assert.doesNotThrow(() => validatePackageFiles(files))
})

test('the package allowlist preserves the manifest entry point and image assets', async () => {
  const manifest = JSON.parse(await readFile(join(packageRoot, 'package.json'), 'utf8'))
  const references = [manifest.main.replace(/^\.\//, ''), manifest.icon]
  for (const views of Object.values(manifest.contributes.viewsContainers)) {
    for (const view of views) references.push(view.icon)
  }
  for (const views of Object.values(manifest.contributes.views)) {
    for (const view of views) references.push(view.icon)
  }
  for (const file of references) assert.ok(requiredFiles.includes(file), 'Missing manifest asset: ' + file)
})

test('file checks reject local state, internal instructions, maps and unapproved runtime files', () => {
  for (const file of forbiddenFiles) {
    assert.throws(() => validatePackageFiles([...requiredFiles, file]), /Unexpected package entry/)
    assert.throws(() => validatePackageFiles([...archiveFiles, 'extension/' + file], { vsix: true }), /Unexpected package entry/)
  }
})

test('file checks reject missing runtime files and VSIX metadata', () => {
  for (const file of requiredFiles) {
    assert.throws(() => validatePackageFiles(requiredFiles.filter(entry => entry !== file)), /Missing package entry/)
  }
  for (const file of archiveFiles) {
    assert.throws(() => validatePackageFiles(archiveFiles.filter(entry => entry !== file), { vsix: true }), /Missing package entry/)
  }
})

test('file checks reject duplicate entries and noncanonical paths', () => {
  assert.throws(() => validatePackageFiles([...requiredFiles, 'package.json']), /Duplicate package entry/)
  for (const file of ['extension\\package.json', '../package.json', '/package.json', './package.json', 'PACKAGE.JSON']) {
    assert.throws(() => validatePackageFiles([...requiredFiles, file]), /Unexpected package entry/)
  }
})

test('final VSIX check reads archive names and accepts the complete runtime package', async t => {
  const directory = await tempDirectory(t)
  const filePath = await createArchive(directory, archiveFiles)
  const files = await readVsixEntries(filePath)
  assert.deepEqual(files, archiveFiles)
  assert.doesNotThrow(() => validatePackageFiles(files, { vsix: true }))
})

test('final VSIX check fails when an archive contains synthetic local configuration', async t => {
  const directory = await tempDirectory(t)
  const filePath = await createArchive(directory, [...archiveFiles, 'extension/.serena/project.local.yml'])
  const result = spawnSync(process.execPath, [
    join(packageRoot, 'scripts/check-package-files.mjs'), '--vsix', filePath,
  ], { encoding: 'utf8' })
  assert.equal(result.status, 1)
  assert.match(result.stderr, /Unexpected package entry: extension\/\.serena\/project.local.yml/)
})

test('final VSIX check fails on missing or malformed archives', async t => {
  const directory = await tempDirectory(t)
  const filePath = join(directory, 'invalid.vsix')
  await assert.rejects(readVsixEntries(filePath), /ENOENT/)
  await writeFile(filePath, 'Synthetic invalid archive')
  await assert.rejects(readVsixEntries(filePath), /End of central directory record signature not found/)
})
