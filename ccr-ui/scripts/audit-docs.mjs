import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const uiRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const docsRoot = path.join(uiRoot, 'docs')

const failures = new Set()
const requiredFiles = [
  'README.md',
  'architecture/overview.md',
  'design-system/page-templates-and-surfaces.md',
  'development/verification.md',
  'archive/README.md'
]
const historicalTopLevelDirs = new Set(['artifacts', 'plans', 'spark', 'superpowers'])

function walkFiles(dir) {
  if (!fs.existsSync(dir)) return []

  return fs.readdirSync(dir, { withFileTypes: true }).flatMap(entry => {
    const fullPath = path.join(dir, entry.name)
    return entry.isDirectory() ? walkFiles(fullPath) : [fullPath]
  })
}

function relativeToDocs(filePath) {
  return path.relative(docsRoot, filePath).replaceAll(path.sep, '/')
}

function assert(condition, message) {
  if (!condition) failures.add(message)
}

function checkStructure() {
  for (const file of requiredFiles) {
    assert(fs.existsSync(path.join(docsRoot, file)), `Missing maintained document: ${file}`)
  }

  for (const entry of fs.readdirSync(docsRoot, { withFileTypes: true })) {
    if (!entry.isDirectory() || !historicalTopLevelDirs.has(entry.name)) continue
    const files = walkFiles(path.join(docsRoot, entry.name))
    assert(files.length === 0, `Historical material remains outside archive/: ${entry.name}`)
  }
}

function checkArchiveIndex() {
  const archiveIndexPath = path.join(docsRoot, 'archive', 'README.md')
  if (!fs.existsSync(archiveIndexPath)) return

  const archiveIndex = fs.readFileSync(archiveIndexPath, 'utf8')
  const archivedFiles = walkFiles(path.join(docsRoot, 'archive'))
    .filter(file => path.basename(file) !== 'README.md')

  for (const file of archivedFiles) {
    assert(
      archiveIndex.includes(path.basename(file)),
      `Archived file is missing from archive/README.md: ${relativeToDocs(file)}`
    )
  }

  for (const file of archivedFiles.filter(file => file.endsWith('.md'))) {
    const content = fs.readFileSync(file, 'utf8')
    assert(content.includes('Archive status:'), `Archived Markdown is missing status metadata: ${relativeToDocs(file)}`)
  }
}

function checkMarkdownLinks() {
  const markdownFiles = walkFiles(docsRoot).filter(file => file.endsWith('.md'))
  const linkPattern = /\[[^\]]*\]\(([^)]+)\)/g

  for (const file of markdownFiles) {
    const content = fs.readFileSync(file, 'utf8')
    for (const match of content.matchAll(linkPattern)) {
      const rawTarget = match[1].trim().split(/\s+['"]/)[0]
      if (!rawTarget || /^(?:https?:|mailto:|#)/.test(rawTarget)) continue

      const targetWithoutAnchor = decodeURIComponent(rawTarget.split('#')[0])
      if (!targetWithoutAnchor) continue

      const target = path.resolve(path.dirname(file), targetWithoutAnchor)
      assert(fs.existsSync(target), `Broken local link in ${relativeToDocs(file)}: ${rawTarget}`)
    }
  }
}

function checkStaleReferences() {
  const activeMarkdown = walkFiles(docsRoot)
    .filter(file => file.endsWith('.md'))
    .filter(file => !relativeToDocs(file).startsWith('archive/'))

  for (const file of activeMarkdown) {
    const content = fs.readFileSync(file, 'utf8')
    assert(
      !content.includes('bun run test:playwright:snapshots'),
      `Unsupported snapshot command in ${relativeToDocs(file)}`
    )
  }
}

checkStructure()
checkArchiveIndex()
checkMarkdownLinks()
checkStaleReferences()

if (failures.size > 0) {
  process.stderr.write(`ccr-ui docs audit failed:\n${[...failures].map(failure => `- ${failure}`).join('\n')}\n`)
  process.exit(1)
}

process.stdout.write('ccr-ui docs audit passed\n')
