import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const docsRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const repoRoot = path.resolve(docsRoot, '..')

const read = (...segments) => fs.readFileSync(path.join(repoRoot, ...segments), 'utf8')
const readIfExists = (...segments) => {
  const target = path.join(repoRoot, ...segments)
  return fs.existsSync(target) ? fs.readFileSync(target, 'utf8') : null
}

const configText = readIfExists('docs', '.vitepress', 'config.mjs')
const cliDefinitions = read('crates', 'ccr-cli', 'src', 'cli', 'definitions.rs')
const workspaceManifest = read('Cargo.toml')
const uiRouter = read('ccr-ui', 'src', 'router', 'index.ts')

const failures = new Set()
const internalOnlyFiles = new Set(['README.md', 'AGENTS.md', 'TODO.md'])
const internalOnlyPrefixes = ['reports/']
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

function relFromDocs(filePath) {
  return path.relative(docsRoot, filePath).replaceAll(path.sep, '/')
}

function isInternalOnly(relPath) {
  return internalOnlyFiles.has(relPath)
    || internalOnlyPrefixes.some(prefix => relPath.startsWith(prefix))
}

function publishedMarkdownFiles() {
  return walkMarkdownFiles(docsRoot).filter(file => !isInternalOnly(relFromDocs(file)))
}

function activeMarkdownFiles() {
  return publishedMarkdownFiles().filter(file => !historicalFiles.has(relFromDocs(file)))
}

function collectDocText(prefix = '') {
  return activeMarkdownFiles()
    .filter(file => relFromDocs(file).startsWith(prefix))
    .map(file => fs.readFileSync(file, 'utf8'))
    .join('\n')
}

function assert(condition, message) {
  if (!condition) failures.add(message)
}

function targetExists(basePath) {
  const candidates = [basePath, `${basePath}.md`, path.join(basePath, 'index.md')]
  return candidates.some(candidate => fs.existsSync(candidate))
}

function resolveDocTarget(sourceFile, rawTarget) {
  const trimmed = rawTarget.trim().split(/\s+['"]/)[0].replace(/^<|>$/g, '')
  if (!trimmed || /^(?:https?:|mailto:|tel:|#)/.test(trimmed)) return null

  let decoded
  try {
    decoded = decodeURIComponent(trimmed.split(/[?#]/)[0])
  } catch {
    return { target: trimmed, exists: false }
  }
  if (!decoded) return null

  if (decoded === '/') {
    return { target: trimmed, exists: fs.existsSync(path.join(docsRoot, 'index.md')) }
  }

  const basePath = decoded.startsWith('/')
    ? path.join(docsRoot, decoded.replace(/^\/+/, ''))
    : path.resolve(path.dirname(sourceFile), decoded)

  return { target: trimmed, exists: targetExists(basePath) }
}

function checkInternalLinks() {
  assert(Boolean(configText), 'Missing docs/.vitepress/config.mjs')
  if (configText) {
    const links = [...configText.matchAll(/link:\s*'([^']+)'/g)]
      .map(match => match[1])
      .filter(link => link.startsWith('/'))

    for (const link of links) {
      const resolved = resolveDocTarget(path.join(docsRoot, 'index.md'), link)
      assert(resolved?.exists, `Missing markdown target for nav/sidebar link: ${link}`)
    }
  }

  const linkPattern = /!?\[[^\]]*\]\(([^)]+)\)/g
  for (const file of publishedMarkdownFiles()) {
    const content = fs.readFileSync(file, 'utf8')
    for (const match of content.matchAll(linkPattern)) {
      const resolved = resolveDocTarget(file, match[1])
      if (!resolved) continue
      assert(resolved.exists, `Broken local link in ${relFromDocs(file)}: ${resolved.target}`)
    }
  }
}

function checkLocaleParity() {
  const allDocs = publishedMarkdownFiles().map(relFromDocs)
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
}

function pascalToKebab(value) {
  return value.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase()
}

function extractTopLevelCommands() {
  const lines = cliDefinitions.split(/\r?\n/)
  const commands = []
  let insideCommands = false
  let explicitName = null

  for (const line of lines) {
    if (!insideCommands) {
      if (line.trim() === 'pub enum Commands {') insideCommands = true
      continue
    }
    if (line === '}') break

    const nameOverride = line.match(/^    #\[command\(name\s*=\s*"([^"]+)"\)\]$/)
    if (nameOverride) {
      explicitName = nameOverride[1]
      continue
    }

    const variant = line.match(/^    ([A-Z][A-Za-z0-9]*)(?:\s*\{|\(|,)/)
    if (!variant) continue
    commands.push(explicitName ?? pascalToKebab(variant[1]))
    explicitName = null
  }

  return commands
}

function checkCommandCoverage() {
  const excludedPages = new Set(['help'])
  const allowedExtraPages = new Set(['tui'])
  const commands = extractTopLevelCommands()
  const expectedPages = new Set(commands.filter(command => !excludedPages.has(command)))

  for (const command of expectedPages) {
    assert(
      fs.existsSync(path.join(docsRoot, 'reference', 'commands', `${command}.md`)),
      `Missing Chinese command page for: ${command}`
    )
    assert(
      fs.existsSync(path.join(docsRoot, 'en', 'reference', 'commands', `${command}.md`)),
      `Missing English command page for: ${command}`
    )
  }

  const actualPages = fs.readdirSync(path.join(docsRoot, 'reference', 'commands'))
    .filter(file => file.endsWith('.md') && file !== 'index.md')
    .map(file => file.slice(0, -3))
  const unexpectedPages = actualPages.filter(page => !expectedPages.has(page) && !allowedExtraPages.has(page))
  assert(unexpectedPages.length === 0, `Unexpected command pages: ${unexpectedPages.join(', ')}`)

  const zhOverview = read('docs', 'reference', 'commands', 'index.md')
  const enOverview = read('docs', 'en', 'reference', 'commands', 'index.md')
  assert(zhOverview.includes('ccr help'), 'Chinese command overview must document ccr help')
  assert(enOverview.includes('ccr help'), 'English command overview must document ccr help')
}

function checkWorkspaceCoverage() {
  const membersBlock = workspaceManifest.match(/members\s*=\s*\[([\s\S]*?)\]/)?.[1] ?? ''
  const crates = [...membersBlock.matchAll(/"(crates\/[^"\n]+)"/g)].map(match => match[1])
  const zhMap = read('docs', 'reference', 'internals', 'crate-map.md')
  const enMap = read('docs', 'en', 'reference', 'internals', 'crate-map.md')

  for (const crate of crates) {
    assert(zhMap.includes(`\`${crate}\``), `Chinese crate map missing workspace member: ${crate}`)
    assert(enMap.includes(`\`${crate}\``), `English crate map missing workspace member: ${crate}`)
  }
}

function checkUiModuleCoverage() {
  const routes = ['/claude-code', '/codex', '/antigravity', '/opencode', '/mcp-manager', '/usage', '/monitoring', '/wsl', '/ssh']
  const zhModules = read('docs', 'guide', 'ui-modules.md')
  const enModules = read('docs', 'en', 'guide', 'ui-modules.md')

  for (const route of routes) {
    assert(uiRouter.includes(`path: '${route.slice(1)}'`) || uiRouter.includes(`path: '${route}'`), `Router source missing expected route: ${route}`)
    assert(zhModules.includes(`\`${route}\``), `Chinese UI module map missing route: ${route}`)
    assert(enModules.includes(`\`${route}\``), `English UI module map missing route: ${route}`)
  }

  for (const staleRoute of ['`/droid`', '`/qwen`', '`/provider-health`']) {
    assert(!zhModules.includes(staleRoute), `Chinese UI module map contains stale route: ${staleRoute}`)
    assert(!enModules.includes(staleRoute), `English UI module map contains stale route: ${staleRoute}`)
  }
}

function extractUiDefaults() {
  const uiPort = cliDefinitions.match(/Ui \{[\s\S]*?default_value_t = (\d+)\)\s*\n\s*port: u16/)
  const backendPort = cliDefinitions.match(/Ui \{[\s\S]*?default_value_t = (\d+)\)\s*\n\s*backend_port: u16/)
  return { uiPort: uiPort?.[1], backendPort: backendPort?.[1] }
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
checkCommandCoverage()
checkWorkspaceCoverage()
checkUiModuleCoverage()
checkFactSync()

if (failures.size > 0) {
  process.stderr.write(`docs audit failed:\n${[...failures].map(failure => `- ${failure}`).join('\n')}\n`)
  process.exit(1)
}

process.stdout.write('docs audit passed\n')
