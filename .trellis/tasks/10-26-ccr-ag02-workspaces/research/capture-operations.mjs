import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import fs from 'node:fs'
import { execFileSync } from 'node:child_process'
import crypto from 'node:crypto'
const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..')
const require = createRequire(path.join(repo, 'ccr-ui/package.json'))
const { chromium } = require('playwright')
const out = path.join(repo, '.trellis/tasks/10-26-ccr-ag02-workspaces/evidence/web', new Date().toISOString().replaceAll(':', '-'))
fs.mkdirSync(out, { recursive: true })
const diff = execFileSync('git', ['-c', 'core.safecrlf=false', 'diff', '--', 'ccr-ui'], { cwd: repo, stdio: ['ignore', 'pipe', 'ignore'] })
const results = { startedAt: new Date().toISOString(), sourceDiffSha256: crypto.createHash('sha256').update(diff).digest('hex'), routes: [], controls: [], dialogs: [], limitations: ['Web preview and isolated display fixture only; no native capability, backend writes, real credential or real user data validation.'] }
const browser = await chromium.launch({ headless: true, executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe' })
results.browser = browser.version()
const inspect = () => {
  const summary = { path: location.pathname, dataset: { ...document.documentElement.dataset }, heading: [...document.querySelectorAll('.route-page h1')].map(e => e.textContent), visibleBlur: [] }
  for (const el of document.querySelectorAll('*')) {
    const s = getComputedStyle(el), r = el.getBoundingClientRect()
    if (s.display !== 'none' && s.visibility !== 'hidden' && r.width && r.height && r.bottom > 0 && r.top < innerHeight && s.backdropFilter !== 'none') summary.visibleBlur.push({ class: String(el.className), filter: s.backdropFilter })
    for (const pseudo of ['::before', '::after']) {
      const ps = getComputedStyle(el, pseudo)
      if (s.display !== 'none' && r.width && r.height && r.bottom > 0 && r.top < innerHeight && ps.content !== 'none' && ps.content !== 'normal' && ps.backdropFilter !== 'none') summary.visibleBlur.push({ class: String(el.className), pseudo, filter: ps.backdropFilter })
    }
  }
  return summary
}
try {
  for (const theme of ['light', 'dark']) for (const flavor of ['neutral', 'clay']) {
    const context = await browser.newContext({ viewport: { width: 1200, height: 800 }, deviceScaleFactor: 1, reducedMotion: 'reduce' })
    await context.addInitScript(({ theme, flavor }) => {
      for (const [k, v] of Object.entries({ 'ccr-theme': theme, 'ccr-flavor': flavor, 'ccr-accent': 'clay', 'ccr-locale': 'zh-CN' })) localStorage.setItem(k, v)
    }, { theme, flavor })
    const page = await context.newPage()
    for (const route of ['/configs', '/converter', '/mcp-manager', '/ssh', '/skills', '/claude-code/profiles']) {
      await page.goto('http://127.0.0.1:5173' + route)
      await page.locator('.route-page h1').first().waitFor({ timeout: 45000 })
      await page.evaluate(() => document.fonts.ready)
      await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))))
      const state = await page.evaluate(inspect)
      if (state.path !== route || state.dataset.theme !== theme || state.dataset.flavor !== flavor || state.dataset.accent !== 'clay' || (await page.locator('.route-page').innerText()).includes('Route placeholder')) throw new Error('Wrong rendered page or preference: ' + route)
      const file = `${route.slice(1).replaceAll('/', '-')}-${theme}-${flavor}.png`
      await page.screenshot({ path: path.join(out, file) })
      results.routes.push({ ...state, theme, flavor, screenshot: file, dataSource: 'Real route in browser-unavailable/empty state; no IPC fixture.' })
    }
    await page.goto('http://127.0.0.1:5173/settings')
    await page.locator('.app-settings-view').waitFor()
    await page.evaluate(async () => {
      const resources = performance.getEntriesByType('resource').map(e => e.name)
      const reactUrl = resources.find(s => /\/react\.js\?/.test(s))
      const clientUrl = resources.find(s => /\/react-dom_client\.js\?/.test(s))
      if (!reactUrl || !clientUrl) throw new Error('Cannot share actual app React instance')
      const reactModule = await import(reactUrl), clientModule = await import(clientUrl)
      const React = reactModule.default || reactModule
      const { createRoot } = clientModule.default || clientModule
      const { Button } = await import('/src/ui/button.tsx')
      const { Checkbox } = await import('/src/ui/checkbox.tsx')
      const select = await import('/src/ui/select.tsx')
      await import('/src/components/profiles/profile-editor-shell.css')
      const e = React.createElement
      const host = document.createElement('section')
      host.dataset.testid = 'ag02-controls'
      Object.assign(host.style, { position: 'fixed', inset: '100px 40px auto 300px', zIndex: '100', padding: '24px', background: 'var(--color-bg-surface)', color: 'var(--color-text-primary)', border: '1px solid var(--color-border-default)', borderRadius: '16px', display: 'grid', gap: '20px' })
      document.body.append(host)
      function Fixture() {
        const [checked, setChecked] = React.useState(true)
        return e(React.Fragment, null,
          e('h2', null, 'AG02 isolated real UI controls / 独立展示样本'),
          e('div', { style: { display: 'flex', flexWrap: 'wrap', gap: '8px' } }, ...['primary', 'secondary', 'ghost', 'warning', 'danger', 'accent-soft'].map(variant => e(Button, { key: variant, variant, size: 'sm', 'data-variant': variant }, variant)), e(Button, { variant: 'primary', size: 'sm', disabled: true }, 'Disabled')),
          e('label', { style: { display: 'flex', gap: '12px' } }, e(Checkbox, { checked, onCheckedChange: setChecked, 'aria-label': 'Fixture checked state' }), 'Checked state; local component only'),
          e(select.Select, { defaultValue: 'one' }, e(select.SelectTrigger, { 'aria-label': 'Fixture choice' }, e(select.SelectValue)), e(select.SelectContent, null, e(select.SelectItem, { value: 'one' }, 'One / 第一项'), e(select.SelectItem, { value: 'two' }, 'Two / 第二项'))),
          e('input', { className: 'pe-input', 'aria-label': 'Fixture profile field', defaultValue: 'Local-only draft / 保留草稿' }),
          e('input', { className: 'pe-input pe-input--invalid', 'aria-invalid': true, 'aria-label': 'Fixture invalid field', defaultValue: 'Validation error / 校验失败' }),
        )
      }
      window.__ag02Root = createRoot(host)
      window.__ag02Root.render(e(Fixture))
    })
    await page.getByTestId('ag02-controls').locator('.ui-btn').first().waitFor()
    await page.getByTestId('ag02-controls').locator('[data-variant="primary"]').focus()
    await page.screenshot({ path: path.join(out, `controls-focus-${theme}-${flavor}.png`) })
    const controlState = await page.getByTestId('ag02-controls').evaluate(el => {
      const css = node => { const s = getComputedStyle(node); return { color: s.color, background: s.backgroundColor, border: s.borderTopColor, outline: s.outline, outlineOffset: s.outlineOffset, opacity: s.opacity, disabled: node.disabled, checked: node.getAttribute('data-state') } }
      return { buttons: [...el.querySelectorAll('.ui-btn')].map(node => ({ variant: node.dataset.variant || 'disabled', ...css(node) })), fields: [...el.querySelectorAll('input')].map(css), select: css(el.querySelector('[role="combobox"]')), checkbox: css(el.querySelector('[role="checkbox"]')) }
    })
    controlState.interactions = []
    for (const variant of ['primary', 'secondary', 'warning', 'danger']) {
      const button = page.getByTestId('ag02-controls').locator('[data-variant="' + variant + '"]')
      await button.hover()
      const read = el => { const s = getComputedStyle(el); return { color: s.color, background: s.backgroundColor, transform: s.transform } }
      const hover = await button.evaluate(read)
      await page.mouse.down()
      const pressed = await button.evaluate(read)
      await page.mouse.up()
      controlState.interactions.push({ variant, hover, pressed })
    }
    results.controls.push({ theme, flavor, ...controlState })
    await page.getByRole('combobox', { name: 'Fixture choice' }).click()
    await page.getByRole('option').first().waitFor()
    results.controls.at(-1).selectOpen = await page.getByRole('listbox').evaluate(el => ({ background: getComputedStyle(el).backgroundColor, backdropFilter: getComputedStyle(el).backdropFilter }))
    await page.screenshot({ path: path.join(out, `controls-select-${theme}-${flavor}.png`) })
    await page.keyboard.press('Escape')
    await page.evaluate(() => { window.__ag02Root.unmount(); document.querySelector('[data-testid="ag02-controls"]').remove() })
    for (const type of ['danger', 'warning', 'info']) {
      await page.locator('.app-settings-view button').first().evaluate(el => { el.dataset.ag02FocusReturn = 'true'; el.focus() })
      await page.evaluate(async type => {
        const { useUIStore } = await import('/src/shell/stores/ui.ts')
        useUIStore.getState().clearToasts()
        window.__ag02ConfirmResult = null
        void useUIStore.getState().requestConfirm({ title: 'AG02 isolated ' + type, message: 'Display and keyboard verification only. No operation is attached. 独立展示，不执行写入。', confirmText: 'Confirm fixture', cancelText: 'Cancel fixture', type, surface: 'glass' }).then(value => { window.__ag02ConfirmResult = value })
      }, type)
      const dialog = page.getByRole('dialog', { name: 'AG02 isolated ' + type })
      await dialog.waitFor()
      await page.screenshot({ path: path.join(out, `dialog-${type}-${theme}-${flavor}.png`) })
      const state = await dialog.evaluate(el => ({ background: getComputedStyle(el).backgroundColor, backdropFilter: getComputedStyle(el).backdropFilter, focusedInside: el.contains(document.activeElement) }))
      state.tabTrapped = true
      for (let step = 0; step < 6; step++) {
        await page.keyboard.press('Tab')
        state.tabTrapped &&= await dialog.evaluate(el => el.contains(document.activeElement))
      }
      const material = await page.evaluate(inspect)
      if (state.backdropFilter !== 'none' || !state.focusedInside || !state.tabTrapped || material.visibleBlur.length > 1) throw new Error('Dialog material/focus contract failure')
      await page.keyboard.press('Escape')
      await dialog.waitFor({ state: 'hidden' })
      state.escapeCancelled = await page.evaluate(() => window.__ag02ConfirmResult === false)
      state.focusRestored = await page.evaluate(() => document.activeElement?.dataset.ag02FocusReturn === 'true')
      if (!state.escapeCancelled) throw new Error('Escape cancellation contract failure')
      state.focusRestoreMethod = 'Programmatic open after focusing existing Settings control; this is an observation, not a real triggering action.'
      results.dialogs.push({ theme, flavor, type, ...state, material })
    }
    await context.close()
  }
} catch (error) {
  results.error = String(error)
  throw error
} finally {
  results.finishedAt = new Date().toISOString()
  fs.writeFileSync(path.join(out, 'results.json'), JSON.stringify(results, null, 2))
  await browser.close()
}
console.log(JSON.stringify({ routes: results.routes.length, controls: results.controls.length, dialogs: results.dialogs.length, out }))
