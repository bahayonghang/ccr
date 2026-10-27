import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import fs from 'node:fs'
import { execFileSync } from 'node:child_process'
const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..')
const require = createRequire(path.join(repo, 'ccr-ui/package.json'))
const { chromium } = require('playwright')
const out = path.join(repo, '.trellis/tasks/10-26-ccr-ag02-workspaces/evidence/cards', new Date().toISOString().replaceAll(':', '-'))
fs.mkdirSync(out, { recursive: true })
const results = { startedAt: new Date().toISOString(), states: [], baselineModalDiff: execFileSync('git', ['diff', '--', 'ccr-ui/src/ui/base-modal.tsx'], { cwd: repo, encoding: 'utf8' }), limitations: ['QueryClient memory-only display data. No operation is invoked; scroll/display only. No native capability proven.'] }
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
  await page.locator('[data-config-name="ag02-fixture-ready"]').scrollIntoViewIfNeeded()
  await page.screenshot({ path:path.join(out,`cards-${theme}-${flavor}.png`) })
  state.filters = await page.locator('.filter-btn').evaluateAll(nodes => nodes.map(el => ({ text:el.textContent, width:el.getBoundingClientRect().width, height:el.getBoundingClientRect().height, whiteSpace:getComputedStyle(el).whiteSpace })))
  state.viewports = []
  for (const width of [1200, 800, 390]) {
   await page.setViewportSize({width,height:800})
   await page.waitForFunction(width => document.querySelector('#primary-navigation-panel').classList.contains('fixed') === (width < 1024), width)
   await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))))
   await page.locator('.filter-btn').first().scrollIntoViewIfNeeded()
   const controls = await page.evaluate(() => {
    const button=document.querySelector('.configs-page button.shrink-0.border-border-strong')
    const bounds=button.getBoundingClientRect()
    const content=document.querySelector('.content-scroll-area').getBoundingClientRect()
    return {width:innerWidth,refresh:{left:bounds.left,right:bounds.right,width:bounds.width},content:{left:content.left,right:content.right}}
   })
   await page.screenshot({path:path.join(out,`controls-${theme}-${flavor}-${width}.png`)})
   if(controls.refresh.left<controls.content.left || controls.refresh.right>controls.content.right+1) throw new Error('Refresh action clipped: '+JSON.stringify(controls))
   state.viewports.push(controls)
  }
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
console.log(JSON.stringify({out,states:results.states.length,filters:results.states.map(s=>s.filters)}))
