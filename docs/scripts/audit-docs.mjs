import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const docsRoot = path.resolve(__dirname, '..')
const repoRoot = path.resolve(docsRoot, '..')

const read = (...segments) => fs.readFileSync(path.join(repoRoot, ...segments), 'utf8')
const readIfExists = (...segments) => {
  const target = path.join(repoRoot, ...segments)
  return fs.existsSync(target) ? fs.readFileSync(target, 'utf8') : null
}

const configText = readIfExists('docs', '.vitepress', 'config.mjs')
const cliDefinitions = read('crates', 'ccr-cli', 'src', 'cli', 'definitions.rs')

const failures = new Set()
const ignoredParityFiles = new Set([
  'README.md'
])
const historicalFiles = new Set([
  'reference/changelog.md',
  'en/reference/changelog.md'
])
const allowedHistoricalMentions = new Set([
  'guide/entrypoints.md',
  'en/guide/entrypoints.md',
  'reference/migration.md',
  'en/reference/migration.md',
  'reference/platforms/migration.md',
  'en/reference/platforms/migration.md'
])

function walkMarkdownFiles(dir) {
  const files = []
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'node_modules' || entry.name === '.vitepress') continue
    const fullPath = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...walkMarkdownFiles(fullPath))
    } else if (entry.isFile() && entry.name.endsWith('.md')) {
      files.push(fullPath)
    }
  }
  return files
}

function markdownExistsForLink(link) {
  const normalized = link.replace(/^\/+|\/+$/g, '')
  if (!normalized) {
    return fs.existsSync(path.join(docsRoot, 'index.md'))
  }

  const candidates = [
    path.join(docsRoot, `${normalized}.md`),
    path.join(docsRoot, normalized, 'index.md')
  ]

  return candidates.some(candidate => fs.existsSync(candidate))
}

function relFromDocs(filePath) {
  return path.relative(docsRoot, filePath).replaceAll(path.sep, '/')
}

function assert(condition, message) {
  if (!condition) failures.add(message)
}

function extractUiDefaults() {
  const uiPort = cliDefinitions.match(/Ui \{[\s\S]*?default_value_t = (\d+)\)\s*\n\s*port: u16/)
  const backendPort = cliDefinitions.match(/Ui \{[\s\S]*?default_value_t = (\d+)\)\s*\n\s*backend_port: u16/)

  return {
    uiPort: uiPort?.[1],
    backendPort: backendPort?.[1]
  }
}

function activeMarkdownFiles() {
  return walkMarkdownFiles(docsRoot).filter(file => {
    const rel = relFromDocs(file)
    return !historicalFiles.has(rel) && !ignoredParityFiles.has(rel)
  })
}

function collectDocText(prefix = '') {
  return activeMarkdownFiles()
    .filter(file => relFromDocs(file).startsWith(prefix))
    .map(file => fs.readFileSync(file, 'utf8'))
    .join('\n')
}

function checkInternalLinks() {
  assert(Boolean(configText), 'Missing docs/.vitepress/config.mjs')
  if (!configText) return

  const links = [...configText.matchAll(/link:\s*'([^']+)'/g)]
    .map(match => match[1])
    .filter(link => link.startsWith('/'))

  for (const link of links) {
    assert(markdownExistsForLink(link), `Missing markdown target for nav/sidebar link: ${link}`)
  }
}

function checkLocaleParity() {
  const allDocs = walkMarkdownFiles(docsRoot)
    .map(relFromDocs)
    .filter(file => !ignoredParityFiles.has(file))

  const zh = new Set(allDocs.filter(file => !file.startsWith('en/')))
  const en = new Set(allDocs.filter(file => file.startsWith('en/')).map(file => file.replace(/^en\//, '')))

  const missingInEn = [...zh].filter(file => !en.has(file))
  const missingInZh = [...en].filter(file => !zh.has(file))

  assert(missingInEn.length === 0, `Missing English mirror pages: ${missingInEn.join(', ')}`)
  assert(missingInZh.length === 0, `Missing Chinese mirror pages: ${missingInZh.join(', ')}`)
}

function checkPlaceholderTranslations() {
  const placeholderPatterns = [
    /Translation in progress/i,
    /This documentation is being translated/i,
    /please refer to the \[Chinese version\]/i
  ]

  const coreFiles = [
    'docs/en/index.md',
    'docs/en/guide/quick-start.md',
    'docs/en/guide/configuration.md',
    'docs/en/guide/cli-workflows.md',
    'docs/en/guide/entrypoints.md',
    'docs/en/guide/ui-overview.md',
    'docs/en/guide/ui-modules.md',
    'docs/en/reference/architecture.md',
    'docs/en/reference/migration.md',
    'docs/en/reference/internals/crate-map.md',
    'docs/en/reference/internals/runtime-flows.md',
    'docs/en/reference/commands/index.md',
    'docs/en/reference/commands/ui.md',
    'docs/en/reference/commands/tui.md'
  ]

  for (const file of coreFiles) {
    const full = path.join(repoRoot, file)
    if (!fs.existsSync(full)) continue
    const content = fs.readFileSync(full, 'utf8')
    for (const pattern of placeholderPatterns) {
      assert(!pattern.test(content), `Placeholder translation marker found in ${file}`)
    }
  }
}

function checkRemovedAndRequiredPages() {
  const removedFiles = [
    'docs/reference/api.md',
    'docs/en/reference/api.md',
    'docs/reference/commands/web.md',
    'docs/en/reference/commands/web.md',
    'docs/reference/commands/migrate.md',
    'docs/en/reference/commands/migrate.md',
    'docs/guide/web-guide.md',
    'docs/en/guide/web-guide.md'
  ]

  const requiredFiles = [
    'docs/guide/entrypoints.md',
    'docs/en/guide/entrypoints.md',
    'docs/reference/internals/crate-map.md',
    'docs/en/reference/internals/crate-map.md',
    'docs/reference/internals/runtime-flows.md',
    'docs/en/reference/internals/runtime-flows.md'
  ]

  for (const file of removedFiles) {
    assert(!fs.existsSync(path.join(repoRoot, file)), `Removed doc still exists: ${file}`)
  }

  for (const file of requiredFiles) {
    assert(fs.existsSync(path.join(repoRoot, file)), `Required doc is missing: ${file}`)
  }

  if (!configText) return

  const removedLinks = [
    '/reference/api',
    '/en/reference/api',
    '/reference/commands/web',
    '/en/reference/commands/web',
    '/guide/web-guide',
    '/en/guide/web-guide'
  ]

  for (const link of removedLinks) {
    assert(!configText.includes(link), `Removed link still present in nav/sidebar config: ${link}`)
  }
}

function checkFactSync() {
  const defaults = extractUiDefaults()
  const zhDocs = collectDocText('')
  const enDocs = collectDocText('en/')

  for (const fact of [defaults.uiPort, defaults.backendPort].filter(Boolean)) {
    assert(zhDocs.includes(String(fact)), `Chinese docs missing ccr ui fact: ${fact}`)
    assert(enDocs.includes(String(fact)), `English docs missing ccr ui fact: ${fact}`)
  }

  const bannedPatterns = [
    { pattern: /\bccr web\b/i, label: 'removed ccr web command' },
    { pattern: /\bccr migrate\b/i, label: 'removed ccr migrate command' },
    { pattern: /\bplatform migrate\b/i, label: 'nonexistent platform migrate command' },
    { pattern: /\bccr restore\b/i, label: 'nonexistent ccr restore command' },
    { pattern: /\bccr backup(?:s)?\b/i, label: 'nonexistent ccr backup command' },
    { pattern: /\/reference\/api\b/i, label: 'removed API reference route' },
    { pattern: /\/en\/reference\/api\b/i, label: 'removed English API reference route' },
    { pattern: /\/reference\/commands\/web\b/i, label: 'removed web command route' },
    { pattern: /\/en\/reference\/commands\/web\b/i, label: 'removed English web command route' },
    { pattern: /web 特性/, label: 'stale zh web-feature wording' },
    { pattern: /web feature/i, label: 'stale en web-feature wording' }
  ]

  for (const file of activeMarkdownFiles()) {
    const rel = relFromDocs(file)
    const content = fs.readFileSync(file, 'utf8')
    if (allowedHistoricalMentions.has(rel)) continue
    for (const { pattern, label } of bannedPatterns) {
      assert(!pattern.test(content), `Detected ${label} in ${rel}`)
    }
  }
}

checkInternalLinks()
checkLocaleParity()
checkPlaceholderTranslations()
checkRemovedAndRequiredPages()
checkFactSync()

if (failures.size > 0) {
  console.error('docs audit failed:')
  for (const failure of failures) {
    console.error(`- ${failure}`)
  }
  process.exit(1)
}

console.log('docs audit passed')
