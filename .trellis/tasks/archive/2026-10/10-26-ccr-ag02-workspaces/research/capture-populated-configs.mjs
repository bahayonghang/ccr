import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import fs from 'node:fs'
import { execFileSync } from 'node:child_process'
const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..')
const require = createRequire(path.join(repo, 'ccr-ui/package.json'))
const { chromium } = require('playwright')
const out = path.join(repo, '.trellis/tasks/10-26-ccr-ag02-workspaces/evidence/populated', new Date().toISOString().replaceAll(':', '-'))
fs.mkdirSync(out, { recursive: true })
const results = { startedAt: new Date().toISOString(), states: [], baselineModalDiff: execFileSync('git', ['diff', '--', 'ccr-ui/src/ui/base-modal.tsx'], { cwd: repo, encoding: 'utf8' }), limitations: ['QueryClient memory-only display data. Actual existing Switch handler opens confirmation; Escape cancels before any backend action. No native capability proven.'] }
const browser = await chromium.launch({ headless: true, executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe' })
try {
 for (const theme of ['light', 'dark']) for (const flavor of ['neutral', 'clay']) {
  const context = await browser.newContext({ viewport: { width: 1200, height: 800 }, reducedMotion: 'reduce' })
  await context.addInitScript(({ theme, flavor }) => { for (const [key,value] of Object.entries({ 'ccr-theme': theme, 'ccr-flavor': flavor, 'ccr-accent': 'clay', 'ccr-locale': 'en-US' })) localStorage.setItem(key,value) }, { theme, flavor })
  const page = await context.newPage()
  await page.goto('http://127.0.0.1:5173/configs')
  await page.locator('.configs-page').waitFor()
  await page.evaluate(async () => {
   const { queryClient } = await import('/src/shell/queryClient.ts')
   const { configsKeys } = await import('/src/features/configs/queries.ts')
   await queryClient.cancelQueries({ queryKey: configsKeys.all })
   const row = (name, props={}) => ({ version: 'ag02-display-fixture', name, description: 'Isolated draft and confirmation fixture / 独立数据，不写入配置', base_url: 'https://fixture.invalid', auth_token: 'masked', is_current: false, is_default: false, usage_count: 42, enabled: true, provider: 'Fixture', model: 'fixture-model', ...props })
   queryClient.setQueryData(configsKeys.list(), { configs: [row('ag02-fixture-current', { is_current: true, is_default: true, provider_type: 'official_relay' }), row('ag02-fixture-ready', { provider_type: 'third_party_model' }), row('ag02-fixture-disabled', { enabled: false })], current_config: 'ag02-fixture-current', default_config: 'ag02-fixture-current' })
  })
  await page.locator('[data-config-name="ag02-fixture-ready"]').waitFor()
  const state = await page.evaluate(() => {
   const css = el => { const s=getComputedStyle(el);return { color:s.color,background:s.backgroundColor,border:s.borderTopColor,opacity:s.opacity } }
   return { dataset:{...document.documentElement.dataset}, rows:[...document.querySelectorAll('[data-config-name]')].map(el=>({ name:el.dataset.configName,...css(el),heading:css(el.querySelector('h3')),badges:[...el.querySelectorAll('h3+div span')].map(css) })) }
  })
  if (state.dataset.theme!==theme || state.dataset.flavor!==flavor) throw new Error('Wrong dataset')
  await page.screenshot({ path:path.join(out,`configs-${theme}-${flavor}.png`) })
  const card=page.locator('[data-config-name="ag02-fixture-ready"]')
  await card.hover()
  const trigger=card.locator('.switch-btn')
  await trigger.focus()
  state.focusBefore=await trigger.evaluate(el=>document.activeElement===el)
  await trigger.click()
  const dialog=page.getByRole('dialog')
  await dialog.waitFor()
  state.dialogFocusInside=await dialog.evaluate(el=>el.contains(document.activeElement))
  await page.screenshot({ path:path.join(out,`switch-confirm-${theme}-${flavor}.png`) })
  await page.keyboard.press('Escape')
  await dialog.waitFor({ state:'hidden' })
  try { await page.waitForFunction(()=>document.activeElement?.closest('[data-config-name="ag02-fixture-ready"]')?.querySelector('.switch-btn')===document.activeElement,{},{timeout:2000}) } catch { /* record observed focus failure */ }
  state.focusRestored=await trigger.evaluate(el=>document.activeElement===el)
  state.activeAfter=await page.evaluate(()=>({tag:document.activeElement?.tagName,ariaLabel:document.activeElement?.getAttribute('aria-label')}))
  state.rowsRemain=await page.locator('[data-config-name]').count()
  results.states.push({ theme,flavor,...state })
  await context.close()
 }
} catch (error) {
 results.error=String(error)
 throw error
} finally {
 results.finishedAt=new Date().toISOString()
 fs.writeFileSync(path.join(out,'results.json'),JSON.stringify(results,null,2))
 await browser.close()
}
console.log(JSON.stringify({out,states:results.states.length,focusRestored:results.states.map(s=>s.focusRestored)}))
