import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import fs from 'node:fs'
import { execFileSync } from 'node:child_process'
import crypto from 'node:crypto'

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..')
const require = createRequire(path.join(repo, 'ccr-ui/package.json'))
const { chromium } = require('playwright')
const phase = process.argv[2] || 'before'
if (!['before', 'after'].includes(phase)) throw new Error('Expected before or after')
const out = path.join(repo, '.trellis/tasks/10-26-ccr-ag01-foundation/evidence', process.argv[3] || phase)
fs.mkdirSync(out, { recursive: true })
const git = (...args) => execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim()
const productDiff = git('diff', '--', 'ccr-ui')
const metadata = {
  phase, startedAt: new Date().toISOString(), head: git('rev-parse', 'HEAD'),
  productDiffSha256: crypto.createHash('sha256').update(productDiff).digest('hex'),
  status: git('status', '--short'), node: process.version, platform: process.platform,
  playwright: require('playwright/package.json').version,
  preview: 'Vite dev; task wrapper watch:null; same before/after; no Tauri runtime or IPC mocks',
}
fs.writeFileSync(path.join(out, 'metadata.json'), JSON.stringify(metadata, null, 2))
fs.writeFileSync(path.join(out, 'product.diff'), productDiff)
const tokenText = fs.readFileSync(path.join(repo, 'ccr-ui/src/styles/tokens.css'), 'utf8')
fs.writeFileSync(path.join(out, 'token-names.txt'), [...new Set(tokenText.match(/--[a-zA-Z0-9-]+(?=\s*:)/g))].sort().join('\n') + '\n')
const executablePath = 'C:/Program Files/Google/Chrome/Application/chrome.exe'
const browser = await chromium.launch({ headless: true, executablePath })
metadata.browser = browser.version()
const results = { metadata, visuals: [], coldBoot: [], routes: [], scroll: [], limitations: [] }
const routeSet = [
  { path: '/', selector: '.dashboard-view', key: 'home' },
  { path: '/settings', selector: '.app-settings-view', key: 'settings' },
  { path: '/claude-code/profiles', selector: '.route-page h1', key: 'profiles' },
  { path: '/usage', selector: '.usage-page', key: 'usage' },
]
const openContext = async (theme = 'light', flavor = 'neutral', viewport = { width: 1200, height: 800 }) => {
  const context = await browser.newContext({ viewport, deviceScaleFactor: 1, reducedMotion: 'reduce' })
  await context.addInitScript(({ theme, flavor }) => {
    localStorage.setItem('ccr-theme', theme)
    localStorage.setItem('ccr-flavor', flavor)
    localStorage.setItem('ccr-accent', 'clay')
    localStorage.setItem('ccr-locale', 'zh-CN')
    window.__agLongTasks = []
    new PerformanceObserver((list) => window.__agLongTasks.push(...list.getEntries().map(e => ({ start: e.startTime, duration: e.duration })))).observe({ type: 'longtask', buffered: true })
  }, { theme, flavor })
  return context
}
const ready = async (page, route) => {
  await page.locator(route.selector).first().waitFor({ state: 'visible', timeout: 45000 })
  await page.evaluate(() => document.fonts.ready)
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))))
  if (new URL(page.url()).pathname !== route.path) throw new Error(`Unexpected redirect at ${route.path}`)
  if ((await page.locator('.route-page').innerText()).includes('Route placeholder')) throw new Error('Placeholder is not a page')
}
const inspect = page => page.evaluate(() => {
  const root = document.documentElement
  const blur = [...document.querySelectorAll('*')].flatMap(el => {
    const rect = el.getBoundingClientRect()
    const own = getComputedStyle(el)
    if (!rect.width || !rect.height || rect.bottom <= 0 || rect.top >= innerHeight || rect.right <= 0 || rect.left >= innerWidth || own.display === 'none' || own.visibility === 'hidden') return []
    return [null, '::before', '::after'].flatMap(pseudo => {
      const style = getComputedStyle(el, pseudo)
      if (pseudo && (style.content === 'none' || style.content === 'normal')) return []
      return style.backdropFilter !== 'none' ? [{ tag: el.tagName, class: String(el.className), pseudo, filter: style.backdropFilter }] : []
    })
  })
  return { path: location.pathname, dataset: { ...root.dataset }, blur, nodeCount: document.querySelectorAll('*').length, width: innerWidth, height: innerHeight, heading: document.querySelector('.route-page h1')?.textContent, bg: getComputedStyle(document.body).backgroundColor, longTaskMs: window.__agLongTasks.reduce((sum, e) => sum + e.duration, 0) }
})
try {
  const context = await openContext()
  const page = await context.newPage()
  const started = performance.now()
  await page.goto('http://127.0.0.1:5173/', { waitUntil: 'domcontentloaded' })
  await ready(page, routeSet[0])
  results.coldBoot.push({ browserColdViteMayBeWarm: true, routeReadyMs: performance.now() - started, ...(await inspect(page)) })
  const catalog = await page.evaluate(async () => (await import('/src/shell/routeCatalog.ts')).flattenCatalog())
  fs.writeFileSync(path.join(out, 'routes.json'), JSON.stringify(catalog, null, 2))
  for (const route of routeSet) { await page.goto('http://127.0.0.1:5173' + route.path); await ready(page, route) }
  for (let run = 0; run < 3; run++) {
    for (const route of routeSet) {
      const start = performance.now()
      await page.goto('http://127.0.0.1:5173' + route.path, { waitUntil: 'domcontentloaded' })
      await ready(page, route)
      results.routes.push({ run: run + 1, key: route.key, routeReadyMs: performance.now() - start, ...(await inspect(page)) })
    }
  }
  await page.goto('http://127.0.0.1:5173/monitoring')
  await page.getByTestId('monitoring-connection-status').waitFor()
  for (let run = 0; run < 4; run++) {
    if (run) { await page.reload(); await page.getByTestId('monitoring-connection-status').waitFor() }
    await page.evaluate(async () => {
      const { logger } = await import('/src/utils/logger.ts')
      for (let i = 0; i < 500; i++) logger.info('AG01 isolated synthetic log ' + i, { fixture: true })
    })
    await page.waitForFunction(() => document.querySelectorAll('[data-testid="monitoring-log-row"]').length === 500, { timeout: 10000 })
    const sample = await page.evaluate(async () => {
      const rows = document.querySelectorAll('[data-testid="monitoring-log-row"]')
      const container = rows[0]?.parentElement
      if (!container || container.scrollHeight <= container.clientHeight) throw new Error('No scrollable data; refuse false sample')
      container.scrollTop = 0
      const stamps = []
      let raf
      const collect = t => { stamps.push(t); raf = requestAnimationFrame(collect) }
      raf = requestAnimationFrame(collect)
      const start = performance.now()
      for (let i = 0; i < 12; i++) { container.scrollTop += 400; await new Promise(resolve => setTimeout(resolve, 100)) }
      cancelAnimationFrame(raf)
      const intervals = stamps.slice(1).map((t, i) => t - stamps[i])
      const sorted = [...intervals].sort((a, b) => a - b)
      if (!intervals.length || container.scrollTop < 1000) throw new Error('Invalid scroll sample')
      return { rows: rows.length, scrollTop: container.scrollTop, scrollHeight: container.scrollHeight, clientHeight: container.clientHeight, frameIntervals: intervals, frameP95: sorted[Math.floor(sorted.length * 0.95)], longTaskMs: window.__agLongTasks.filter(e => e.start >= start).reduce((sum, e) => sum + e.duration, 0) }
    })
    if (run) results.scroll.push({ run, ...sample })
  }
  await context.close()
  for (const theme of ['light', 'dark']) for (const flavor of ['neutral', 'clay']) {
    const ctx = await openContext(theme, flavor)
    const pg = await ctx.newPage()
    for (const route of routeSet) {
      await pg.goto('http://127.0.0.1:5173' + route.path)
      await ready(pg, route)
      const record = await inspect(pg)
      if (record.dataset.theme !== theme || record.dataset.flavor !== flavor || record.dataset.accent !== 'clay') throw new Error('Theme preference was not applied')
      const file = `${route.key}-${theme}-${flavor}-1200x800.png`
      await pg.screenshot({ path: path.join(out, file) })
      results.visuals.push({ ...record, theme, flavor, screenshot: file })
    }
    await pg.setViewportSize({ width: 800, height: 600 })
    await pg.goto('http://127.0.0.1:5173/')
    await ready(pg, routeSet[0])
    const file = `home-${theme}-${flavor}-800x600.png`
    await pg.screenshot({ path: path.join(out, file) })
    results.visuals.push({ ...(await inspect(pg)), theme, flavor, screenshot: file })
    await ctx.close()
  }
  results.limitations.push('Web only; no native window or real IPC validation.', 'Profiles/Usage use honest browser-unavailable states; chart data, usage tab performance and populated virtual session list NOT_RUN.', 'Route timings measure warm module full-document navigations, not SPA router latency; keep same method for comparison.', 'Scroll workload is 500 synthetic monitoring entries through the real logger, not virtual session rows.', 'Reduced motion is enabled for reproducibility; animation/default-motion acceptance is separate.')
} finally {
  metadata.finishedAt = new Date().toISOString()
  fs.writeFileSync(path.join(out, 'results.json'), JSON.stringify(results, null, 2))
  await browser.close()
}
console.log(JSON.stringify({ phase, screenshots: results.visuals.length, routeSamples: results.routes.length, scrollSamples: results.scroll.length, out }))
