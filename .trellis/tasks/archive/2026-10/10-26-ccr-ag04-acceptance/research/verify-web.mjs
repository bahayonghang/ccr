import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import fs from 'node:fs'
import { execFileSync } from 'node:child_process'

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..')
const require = createRequire(path.join(repo, 'ccr-ui/package.json'))
const { chromium } = require('playwright')
const out = process.argv[2] ? path.resolve(repo, process.argv[2]) : path.join(repo, '.trellis/tasks/10-26-ccr-ag04-acceptance/evidence/web')
fs.mkdirSync(out, { recursive: true })
const results = {
  startedAt: new Date().toISOString(), routes: [], windows: [], failures: [],
  limitations: [
    'Browser-only actual unavailable/empty/error states; no real IPC, user config or authentication writes.',
    'Synthetic parameter routes prove error/unavailable presentation, not existence of a real agent/account.',
    'Native WebView/DPI/tray/macOS and actual engine without backdrop-filter support are NOT_RUN.',
  ],
  head: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repo, encoding: 'utf8' }).trim(),
}
const browser = await chromium.launch({ executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe', headless: true })
results.browser = browser.version()
const contextFor = async (theme, flavor) => {
  const context = await browser.newContext({ viewport: { width: 1200, height: 800 }, reducedMotion: 'reduce' })
  await context.addInitScript(({ theme, flavor }) => {
    localStorage.setItem('ccr-theme', theme)
    localStorage.setItem('ccr-flavor', flavor)
    localStorage.setItem('ccr-accent', 'clay')
    localStorage.setItem('ccr-locale', 'zh-CN')
  }, { theme, flavor })
  return context
}
const material = page => page.evaluate(() => {
  const sheet = document.querySelector('.layout-shell')
  const style = sheet ? getComputedStyle(sheet, '::before') : null
  const filters = []
  for (const el of document.querySelectorAll('*')) {
    const rect = el.getBoundingClientRect()
    if (!rect.width || !rect.height || rect.bottom <= 0 || rect.top >= innerHeight || rect.right <= 0 || rect.left >= innerWidth) continue
    for (const pseudo of [null, '::before', '::after']) {
      const css = getComputedStyle(el, pseudo)
      if (css.display === 'none' || css.visibility === 'hidden' || css.opacity === '0') continue
      if (pseudo && ['none', 'normal'].includes(css.content)) continue
      if (css.backdropFilter !== 'none') filters.push({ tag: el.tagName, class: String(el.className), pseudo, filter: css.backdropFilter })
    }
  }
  return {
    dataset: { ...document.documentElement.dataset }, path: location.pathname,
    width: innerWidth, height: innerHeight, filters,
    sheet: style ? { bg: style.backgroundColor, filter: style.backdropFilter, radius: style.borderRadius } : null,
    mainWidth: document.querySelector('main')?.getBoundingClientRect().width,
    scrollOwner: document.querySelector('.content-scroll-area') ? getComputedStyle(document.querySelector('.content-scroll-area')).overflowY : null,
    bodyOverflow: document.documentElement.scrollWidth > innerWidth,
  }
})
const ready = async (page, tray = false) => {
  await page.locator(tray ? '#app' : '.route-page').waitFor({ state: 'visible', timeout: 30000 })
  await page.evaluate(() => document.fonts.ready)
  await page.waitForFunction(({ tray }) => {
    const root = document.querySelector(tray ? '#app' : '.route-page')
    return root?.innerText.trim().length > 5 && !root.innerText.includes('Route placeholder')
  }, { tray }, { timeout: 15000 })
}
const hydrate = route => route.replace(':client?', 'ccr').replace(':accountId', 'ag04-fixture').replace(':platform', 'claude-code').replace(':name', 'ag04-fixture')
try {
  for (const theme of ['light', 'dark']) {
    const context = await contextFor(theme, 'neutral')
    const page = await context.newPage()
    await page.goto('http://127.0.0.1:5173/')
    await ready(page)
    const catalog = await page.evaluate(async () => (await import('/src/shell/routeCatalog.ts')).flattenCatalog())
    results.catalog = catalog
    for (const route of catalog) {
      if (!route.id && !route.redirect) continue // layout parent is not a second dashboard
      const target = hydrate(route.path)
      try {
        await page.goto('http://127.0.0.1:5173' + target)
        await ready(page, route.path === '/tray/codex')
        const state = await material(page)
        const text = await page.locator(route.path === '/tray/codex' ? '#app' : '.route-page').innerText()
        const identity = await page.locator(route.path === '/tray/codex' ? '#app' : '.route-page').evaluate(el => ({
          class: el.firstElementChild?.className,
          headings: [...el.querySelectorAll('h1,h2')].map(x => x.textContent).slice(0, 5),
          testIds: [...el.querySelectorAll('[data-testid]')].slice(0, 8).map(x => x.getAttribute('data-testid')),
        }))
        const expected = route.redirect ? hydrate(route.redirect).split('?')[0] : target
        if (state.path !== expected) throw new Error(`Unexpected target: ${state.path}; expected ${expected}`)
        if (text.includes('Route placeholder') || text.includes('Unexpected Application Error')) throw new Error('Placeholder or router error')
        if (state.filters.length > 1) throw new Error(`Blur budget: ${state.filters.length}`)
        if (state.dataset.theme !== theme || state.dataset.flavor !== 'neutral') throw new Error('Preference mismatch')
        const screenshot = `${route.id ?? route.path.replace(/[^a-z0-9]+/gi, '-')}-${theme}.png`
        await page.screenshot({ path: path.join(out, screenshot) })
        results.routes.push({ id: route.id, catalogPath: route.path, theme, redirect: route.redirect, identity, excerpt: text.slice(0, 300), screenshot, ...state, status: 'PASS_WEB_RENDER' })
      } catch (error) {
        results.failures.push({ id: route.id, theme, route: target, error: String(error) })
      }
    }
    await context.close()
  }
  for (const theme of ['light', 'dark']) for (const flavor of ['neutral', 'clay']) {
    const context = await contextFor(theme, flavor)
    const page = await context.newPage()
    await page.goto('http://127.0.0.1:5173/settings')
    await ready(page)
    for (const [width, height] of [[390, 844], [800, 600], [959, 800], [960, 800], [1023, 800], [1024, 800], [1440, 900]]) {
      await page.setViewportSize({ width, height })
      await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))))
      const state = await material(page)
      const expected = width < 960 ? 'none' : 'blur(20px) saturate(1.15)'
      if (state.sheet?.filter !== expected) results.failures.push({ theme, flavor, width, error: `Expected ${expected}, got ${state.sheet?.filter}` })
      if (width < 960 && /rgba\(.+0\.\d+\)/.test(state.sheet?.bg ?? '')) results.failures.push({ theme, flavor, width, error: 'Fallback alpha is not 1' })
      const screenshot = `settings-${theme}-${flavor}-${width}x${height}.png`
      await page.screenshot({ path: path.join(out, screenshot) })
      results.windows.push({ theme, flavor, screenshot, ...state })
    }
    await page.setViewportSize({ width: 1200, height: 800 })
    const cdp = await context.newCDPSession(page)
    await cdp.send('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-reduced-transparency', value: 'reduce' }, { name: 'prefers-reduced-motion', value: 'reduce' }] })
    // CDP can resolve before the renderer has applied the new media rules.
    await page.waitForFunction(() => matchMedia('(prefers-reduced-transparency: reduce)').matches && getComputedStyle(document.querySelector('.layout-shell'), '::before').backdropFilter === 'none')
    const emulated = await page.evaluate(() => matchMedia('(prefers-reduced-transparency: reduce)').matches)
    if (!emulated) throw new Error('Reduced transparency emulation failed')
    const state = await material(page)
    if (state.sheet?.filter !== 'none' || state.filters.length) results.failures.push({ theme, flavor, error: 'Reduced transparency left blur enabled' })
    const screenshot = `settings-${theme}-${flavor}-reduced-transparency.png`
    await page.screenshot({ path: path.join(out, screenshot) })
    results.windows.push({ theme, flavor, emulatedReducedTransparency: true, screenshot, ...state })
    await context.close()
  }
} finally {
  results.finishedAt = new Date().toISOString()
  fs.writeFileSync(path.join(out, 'results.json'), JSON.stringify(results, null, 2))
  await browser.close()
}
console.log(JSON.stringify({ routes: results.routes.length, windows: results.windows.length, failures: results.failures }))
if (results.failures.length) process.exitCode = 1
