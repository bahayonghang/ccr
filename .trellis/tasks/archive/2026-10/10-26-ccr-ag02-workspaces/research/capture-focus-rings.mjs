import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import fs from 'node:fs'
const repo=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../../..')
const require=createRequire(path.join(repo,'ccr-ui/package.json'))
const {chromium}=require('playwright')
const out=path.join(repo,'.trellis/tasks/10-26-ccr-ag02-workspaces/evidence/focus',new Date().toISOString().replaceAll(':','-'))
fs.mkdirSync(out,{recursive:true})
const result={startedAt:new Date().toISOString(),samples:[],limitation:'Real UI components mounted in memory-only task display fixture; no backend or native actions.'}
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'})
try{
 for(const theme of ['light','dark'])for(const flavor of ['neutral','clay']){
  const context=await browser.newContext({viewport:{width:1200,height:800},reducedMotion:'reduce'})
  await context.addInitScript(({theme,flavor})=>{for(const[k,v]of Object.entries({'ccr-theme':theme,'ccr-flavor':flavor,'ccr-accent':'clay'}))localStorage.setItem(k,v)},{theme,flavor})
  const page=await context.newPage()
  await page.goto('http://127.0.0.1:5173/settings')
  await page.locator('.app-settings-view').waitFor()
  await page.evaluate(async()=>{
   const urls=performance.getEntriesByType('resource').map(e=>e.name)
   const reactModule=await import(urls.find(u=>/\/react\.js\?/.test(u))),clientModule=await import(urls.find(u=>/\/react-dom_client\.js\?/.test(u)))
   const React=reactModule.default||reactModule,{createRoot}=clientModule.default||clientModule,e=React.createElement
   const{Checkbox}=await import('/src/ui/checkbox.tsx'),select=await import('/src/ui/select.tsx'),combo=await import('/src/ui/combobox.tsx')
   const host=document.createElement('section')
   host.dataset.testid='ag02-focus-fixture'
   Object.assign(host.style,{position:'fixed',inset:'150px 50px auto 300px',zIndex:'100',padding:'24px',background:'var(--color-bg-surface)',color:'var(--color-text-primary)',border:'1px solid var(--color-border-default)',borderRadius:'16px',display:'grid',gap:'28px'})
   document.body.append(host)
   createRoot(host).render(e(React.Fragment,null,e('h2',null,'AG02 solid focus indicators / 键盘焦点'),e('label',{style:{display:'flex',gap:'12px'}},e(Checkbox,{'aria-label':'Focus checkbox',defaultChecked:true}),'Checkbox'),e(select.Select,{defaultValue:'one'},e(select.SelectTrigger,{'aria-label':'Focus select'},e(select.SelectValue)),e(select.SelectContent,null,e(select.SelectItem,{value:'one'},'Select fixture'))),e(combo.Combobox,null,e(combo.ComboboxInput,{'aria-label':'Focus combobox',placeholder:'Combobox fixture'}),e(combo.ComboboxList,null))))
  })
  await page.getByRole('checkbox',{name:'Focus checkbox'}).waitFor()
  await page.keyboard.press('Tab')
  for(const[name,role]of [['Focus checkbox','checkbox'],['Focus select','combobox'],['Focus combobox','combobox']]){
   const el=page.getByRole(role,{name,exact:true})
   await el.focus()
   await page.waitForFunction(label=>{
    const node=document.querySelector('[aria-label="'+label+'"]')
    const rgb=getComputedStyle(document.documentElement).getPropertyValue('--color-accent-primary-rgb').trim().split(/\s+/).join(', ')
    return node&&getComputedStyle(node).boxShadow.includes('rgb('+rgb+')')
   },name,{timeout:5000})
   const computed=await el.evaluate(el=>{const s=getComputedStyle(el);return{dataset:{...document.documentElement.dataset},focusVisible:el.matches(':focus-visible'),boxShadow:s.boxShadow,border:s.borderTopColor,accent:getComputedStyle(document.documentElement).getPropertyValue('--color-accent-primary').trim()}})
   if(!computed.focusVisible||computed.boxShadow==='none')throw new Error('Missing focus ring '+name)
   const screenshot=`${name.replaceAll(' ','-').toLowerCase()}-${theme}-${flavor}.png`
   await page.screenshot({path:path.join(out,screenshot)})
   result.samples.push({theme,flavor,name,...computed,screenshot})
  }
  await context.close()
 }
}finally{result.finishedAt=new Date().toISOString();fs.writeFileSync(path.join(out,'results.json'),JSON.stringify(result,null,2));await browser.close()}
console.log(JSON.stringify({out,samples:result.samples.length}))
