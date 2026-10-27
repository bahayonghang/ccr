import { createRequire } from 'node:module'
import fs from 'node:fs'
import path from 'node:path'
const require = createRequire(path.resolve('ccr-ui/package.json'))
const { chromium } = require('playwright')
const browser = await chromium.launch({headless:true, executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'})
const out='.trellis/tasks/10-26-ccr-ag01-foundation/evidence/material-runtime'
fs.mkdirSync(out,{recursive:true})
const samples=[]
try {
for(const theme of ['light','dark']) for(const flavor of ['neutral','clay']) {
 const context=await browser.newContext({viewport:{width:1200,height:800},reducedMotion:'reduce'})
 await context.addInitScript(({theme,flavor})=>{localStorage.setItem('ccr-theme',theme);localStorage.setItem('ccr-flavor',flavor);localStorage.setItem('ccr-accent','clay')},{theme,flavor})
 const page=await context.newPage();await page.goto('http://127.0.0.1:5173/settings');await page.locator('.app-settings-view').waitFor()
 const inspect=()=>page.evaluate(()=>{
  const el=document.querySelector('.layout-shell'); const cs=getComputedStyle(el,'::before');const rect=el.getBoundingClientRect()
  const activeBlur = [...document.querySelectorAll('*')].flatMap(node => [null, '::before', '::after'].filter(pseudo => {const style=getComputedStyle(node,pseudo); const bounds=node.getBoundingClientRect(); return bounds.width && bounds.height && style.display !== 'none' && style.visibility !== 'hidden' && (!pseudo || (style.content !== 'none' && style.content !== 'normal')) && style.backdropFilter !== 'none'}).map(pseudo => String(node.className)+(pseudo || '')))
  return {activeBlur,dataset:{...document.documentElement.dataset},width:innerWidth,background:cs.backgroundColor,filter:cs.backdropFilter,radius:cs.borderRadius,shell:{x:rect.x,y:rect.y,width:rect.width,height:rect.height},scroll:document.querySelector('.content-scroll-area').clientHeight,bodyScroll:document.body.scrollHeight,bodyHeight:document.body.clientHeight}
 })
 for(const width of [1200,959,960,1023,1024,390]) {await page.setViewportSize({width,height:800});await page.waitForFunction(width=>{const c=getComputedStyle(document.querySelector('.layout-shell'),'::before');return c.backgroundColor!=='rgba(0, 0, 0, 0)' && (width<960?c.backdropFilter==='none':c.backdropFilter.includes('blur(20px)'))},width); const sample=await inspect(); if(sample.dataset.theme!==theme||sample.dataset.flavor!==flavor||sample.dataset.resolvedFlavor!==flavor||sample.dataset.accent!=='clay')throw Error('Theme mismatch'); if(sample.activeBlur.length!==(width<960?0:1)|| (width<960 && sample.background.includes('rgba')))throw Error('Blur budget/viewport fallback failed'); samples.push({theme,flavor,state:'normal',...sample})}
 await page.setViewportSize({width:1200,height:800})
 const client=await context.newCDPSession(page)
 await client.send('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-transparency',value:'reduce'}]})
 await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))))
 await page.waitForFunction(()=>matchMedia('(prefers-reduced-transparency: reduce)').matches && getComputedStyle(document.querySelector('.layout-shell'),'::before').backdropFilter === 'none')
 const reduced=await inspect();if(reduced.filter!=='none'||reduced.background.includes('rgba')||reduced.activeBlur.length)throw Error('Transparency fallback failed')
 samples.push({theme,flavor,state:'reduce-transparency',...reduced})
 await client.send('Emulation.setEmulatedMedia',{features:[]})
 await context.close()
}
fs.writeFileSync(path.join(out,'results.json'),JSON.stringify({samples,limitations:['No-support fallback source guard only; installed Chrome supports backdrop-filter.','Native window/DPI/macOS remain NOT_RUN.']},null,2))
console.log(JSON.stringify({samples:samples.length,out}))
}finally{await browser.close()}
