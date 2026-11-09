import { createRequire } from 'node:module'
import fs from 'node:fs'
import path from 'node:path'
const require = createRequire(path.resolve('ccr-ui/package.json'))
const { chromium } = require('playwright')
const out = '.trellis/tasks/10-26-ccr-ag04-acceptance/evidence/shell-final'
fs.mkdirSync(out, { recursive: true })
const browser = await chromium.launch({ headless: true, executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe' })
const results = { samples: [], failures: [], limitations: ['Browser interaction only; no native APIs or writes. Browser 200% zoom and OS DPI remain NOT_RUN.'] }
try {
  for (const theme of ['light', 'dark']) {
    const context = await browser.newContext({ viewport: { width: 1200, height: 800 }, reducedMotion: 'reduce' })
    await context.addInitScript(theme => {
      localStorage.setItem('ccr-theme', theme)
      localStorage.setItem('ccr-flavor', 'neutral')
      localStorage.setItem('ccr-locale', 'en-US')
    }, theme)
    const page = await context.newPage()
    await page.goto('http://127.0.0.1:5173/settings')
    await page.locator('.app-settings-view').waitFor()
    const nav = page.locator('#primary-navigation-panel')
    const handle = nav.locator('> button').first()
    const before = await nav.evaluate(el => el.getBoundingClientRect().width)
    await handle.focus()
    await handle.press('ArrowRight')
    await page.waitForFunction(before => document.querySelector('#primary-navigation-panel').getBoundingClientRect().width > before, before)
    const keyboardWidth = await nav.evaluate(el => el.getBoundingClientRect().width)
    const box = await handle.boundingBox()
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
    await page.mouse.down()
    await page.mouse.move(box.x + box.width / 2 + 30, box.y + box.height / 2)
    await page.mouse.up()
    const pointerWidth = await nav.evaluate(el => el.getBoundingClientRect().width)
    if (pointerWidth <= keyboardWidth) results.failures.push({ theme, error: 'Pointer resize did not increase width' })
    await page.setViewportSize({ width: 959, height: 800 })
    const toggle = page.locator('#main-content button[aria-expanded]')
    await toggle.click()
    await page.waitForFunction(() => document.querySelector('#main-content button[aria-expanded]')?.getAttribute('aria-expanded') === 'true')
    await page.waitForFunction(() => Math.abs(document.querySelector('#primary-navigation-panel').getBoundingClientRect().x) < 1)
    const drawer = await nav.boundingBox()
    if (Math.abs(drawer.x) > 1 || drawer.height < 700) results.failures.push({ theme, error: 'Fixed mobile drawer misplaced', drawer })
    await page.screenshot({ path: path.join(out, `mobile-drawer-${theme}.png`) })
    await page.locator('.layout-layer-modal-backdrop').click({ position: { x: 800, y: 200 } })
    await page.waitForFunction(() => document.querySelector('#main-content button[aria-expanded]')?.getAttribute('aria-expanded') === 'false')
    results.samples.push({ theme, before, keyboardWidth, pointerWidth, drawer, mobileBackdropCloses: true })
    await page.setViewportSize({ width: 456, height: 620 })
    await page.goto('http://127.0.0.1:5173/tray/codex')
    await page.locator('.codex-tray-panel').waitFor()
    const tray = await page.locator('.codex-tray-panel').evaluate(el => ({ width: el.getBoundingClientRect().width, bg: getComputedStyle(el).backgroundColor, blur: getComputedStyle(el).backdropFilter, sheet: Boolean(document.querySelector('.layout-shell')) }))
    if (tray.sheet || tray.blur !== 'none' || tray.width > 456) results.failures.push({ theme, tray })
    await page.screenshot({ path: path.join(out, `tray-${theme}-456x620.png`) })
    results.samples.push({ theme, tray, viewport: { width: 456, height: 620 } })
    await context.close()
  }
} catch (error) {
  results.failures.push({ error: String(error) })
} finally {
  fs.writeFileSync(path.join(out, 'results.json'), JSON.stringify(results, null, 2))
  await browser.close()
}
console.log(JSON.stringify(results))
if (results.failures.length) process.exitCode = 1
