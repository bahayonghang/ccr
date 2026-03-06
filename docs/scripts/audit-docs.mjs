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
const cliDefinitions = read('crates', 'ccr', 'src', 'cli', 'definitions.rs')
const webServer = read('crates', 'ccr', 'src', 'web', 'server.rs')

const failures = new Set()

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

function extractCliDefaults() {
  const webHost = cliDefinitions.match(/default_value = "([^"]+)"\)\s*\n\s*host: std::net::IpAddr/)
  const webPort = cliDefinitions.match(/default_value_t = (\d+)\)\s*\n\s*port: u16/)
  const uiPort = cliDefinitions.match(/Ui \{[\s\S]*?default_value_t = (\d+)\)\s*\n\s*port: u16/)
  const backendPort = cliDefinitions.match(/Ui \{[\s\S]*?default_value_t = (\d+)\)\s*\n\s*backend_port: u16/)

  return {
    webHost: webHost?.[1],
    webPort: webPort?.[1],
    uiPort: uiPort?.[1],
    backendPort: backendPort?.[1]
  }
}

function extractRoutes() {
  return new Set(
    [...webServer.matchAll(/"((?:\/api\/)[^"]+)"/g)]
      .map(match => match[1])
      .filter(route => route.startsWith('/api/'))
  )
}

function collectDocText(prefix = '') {
  return walkMarkdownFiles(docsRoot)
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
    .filter(file => file !== 'README.md')

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
    'docs/en/guide/web-guide.md',
    'docs/en/guide/ui-overview.md',
    'docs/en/guide/ui-modules.md',
    'docs/en/reference/api.md',
    'docs/en/reference/platforms/index.md',
    'docs/en/reference/commands/codex.md',
    'docs/en/reference/commands/ui.md',
    'docs/en/reference/commands/web.md',
    'docs/en/reference/commands/temp-token.md',
    'docs/en/reference/commands/provider.md'
  ]

  for (const file of coreFiles) {
    if (!fs.existsSync(path.join(repoRoot, file))) continue
    const content = fs.readFileSync(path.join(repoRoot, file), 'utf8')
    for (const pattern of placeholderPatterns) {
      assert(!pattern.test(content), `Placeholder translation marker found in ${file}`)
    }
  }
}

function checkFactSync() {
  const defaults = extractCliDefaults()
  const routes = extractRoutes()
  const zhDocs = collectDocText('')
  const enDocs = collectDocText('en/')

  const requiredRoutes = [
    '/api/system',
    '/api/platforms',
    '/api/codex/profiles',
    '/api/stats/cost/summary',
    '/api/budget/status',
    '/api/pricing/list',
    '/api/sync/status'
  ]

  for (const route of requiredRoutes) {
    assert(routes.has(route), `Route missing from code truth set: ${route}`)
  }

  const apiFiles = [
    path.join(docsRoot, 'reference', 'api.md'),
    path.join(docsRoot, 'en', 'reference', 'api.md')
  ]

  for (const file of apiFiles) {
    if (!fs.existsSync(file)) continue
    const content = fs.readFileSync(file, 'utf8')
    for (const route of requiredRoutes) {
      assert(content.includes(route), `API doc missing route ${route} in ${relFromDocs(file)}`)
    }
    assert(!content.includes('/api/provider-health/test'), `Stale provider-health route found in ${relFromDocs(file)}`)
    assert(!content.includes('/api/provider-health/test-all'), `Stale provider-health route found in ${relFromDocs(file)}`)
  }

  const uiFacts = [defaults.uiPort, defaults.backendPort].filter(Boolean)
  const webFacts = [defaults.webHost, defaults.webPort].filter(Boolean)

  for (const fact of uiFacts) {
    assert(zhDocs.includes(String(fact)), `Chinese docs missing ccr ui fact: ${fact}`)
    assert(enDocs.includes(String(fact)), `English docs missing ccr ui fact: ${fact}`)
  }

  for (const fact of webFacts) {
    assert(zhDocs.includes(String(fact)), `Chinese docs missing ccr web fact: ${fact}`)
    assert(enDocs.includes(String(fact)), `English docs missing ccr web fact: ${fact}`)
  }

  const stalePatterns = [
    { pattern: /默认地址\s*0\.0\.0\.0/, label: 'stale web default host wording (zh)' },
    { pattern: /Default\s*\(0\.0\.0\.0:19527\)/, label: 'stale web default host wording (en)' },
    { pattern: /默认 `3000`/, label: 'stale ccr ui default port wording (zh)' },
    { pattern: /default `3000`/i, label: 'stale ccr ui default port wording (en)' }
  ]

  for (const { pattern, label } of stalePatterns) {
    assert(!pattern.test(zhDocs), `Detected ${label}`)
    assert(!pattern.test(enDocs), `Detected ${label}`)
  }
}

checkInternalLinks()
checkLocaleParity()
checkPlaceholderTranslations()
checkFactSync()

if (failures.size > 0) {
  console.error('docs audit failed:')
  for (const failure of failures) {
    console.error(`- ${failure}`)
  }
  process.exit(1)
}

console.log('docs audit passed')
