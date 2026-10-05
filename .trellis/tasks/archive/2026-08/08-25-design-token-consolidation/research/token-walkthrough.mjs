import { chromium } from 'playwright'
import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.dirname(fileURLToPath(import.meta.url))
const outDir = path.join(root, 'walkthrough-artifacts')
const baseURL = process.env.CCR_WEB_BASE ?? 'http://127.0.0.1:5173'

const PAGES = [
  { id: 'Dashboard', path: '/' },
  { id: 'Profiles', path: '/configs' },
  { id: 'MCP', path: '/mcp-manager' },
  { id: 'Commands', path: '/commands' },
  { id: 'Sync', path: '/sync' },
  { id: 'Check-ins', path: '/checkin' },
  { id: 'Usage', path: '/usage' },
  { id: 'Settings', path: '/settings' },
]

const COMBOS = [
  { theme: 'light', flavor: 'neutral' },
  { theme: 'light', flavor: 'clay' },
  { theme: 'dark', flavor: 'neutral' },
  { theme: 'dark', flavor: 'clay' },
]

const TOKEN_NAMES = [
  '--color-border-subtle',
  '--color-border-default',
  '--color-border-strong',
  '--color-bg-elevated',
  '--color-bg-surface',
  '--radius-sm',
  '--radius-lg',
  '--radius-2xl',
  '--radius-full',
  '--color-platform-opencode',
  '--color-success-tint',
  '--color-warning-tint',
]

const readPageMetrics = () => {
  const rootEl = document.documentElement
  const styles = getComputedStyle(rootEl)
  const tokens = {}
  for (const name of [
    '--color-border-subtle',
    '--color-border-default',
    '--color-border-strong',
    '--color-bg-elevated',
    '--color-bg-surface',
    '--radius-sm',
    '--radius-lg',
    '--radius-2xl',
    '--radius-full',
    '--color-platform-opencode',
    '--color-success-tint',
    '--color-warning-tint',
  ]) {
    tokens[name] = styles.getPropertyValue(name).trim()
  }
  return {
    dataset: {
      theme: rootEl.getAttribute('data-theme'),
      flavor: rootEl.getAttribute('data-flavor'),
      accent: rootEl.getAttribute('data-accent'),
    },
    tokens,
    overflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    title: document.title,
  }
}

const main = async () => {
  await mkdir(outDir, { recursive: true })
  const browser = await chromium.launch({ headless: true })
  const rows = []

  try {
    for (const combo of COMBOS) {
      const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } })
      await context.addInitScript(
        ({ theme, flavor }) => {
          localStorage.setItem('ccr-theme', theme)
          localStorage.setItem('ccr-flavor', flavor)
          localStorage.setItem('ccr-accent', 'clay')
        },
        combo,
      )
      const page = await context.newPage()

      for (const route of PAGES) {
        await page.goto(`${baseURL}${route.path}`, { waitUntil: 'domcontentloaded', timeout: 30_000 })
        await page.waitForTimeout(400)
        const metrics = await page.evaluate(readPageMetrics)
        const row = {
          page: route.id,
          path: route.path,
          combo: `${combo.theme}×${combo.flavor}`,
          ...metrics,
        }
        rows.push(row)

        if (route.id === 'Dashboard' || route.id === 'Settings') {
          const shot = `${combo.theme}-${combo.flavor}-${route.id.toLowerCase()}.png`
          await page.screenshot({ path: path.join(outDir, shot), fullPage: false })
          row.screenshot = shot
        }
      }

      await context.close()
    }

    // custom accent spot-check on settings + dashboard
    const accentContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } })
    await accentContext.addInitScript(() => {
      localStorage.setItem('ccr-theme', 'dark')
      localStorage.setItem('ccr-flavor', 'clay')
      localStorage.setItem('ccr-accent', 'clay')
    })
    const accentPage = await accentContext.newPage()
    await accentPage.goto(`${baseURL}/settings`, { waitUntil: 'domcontentloaded' })
    await accentPage.waitForTimeout(500)
    const accentBefore = await accentPage.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue('--color-accent-primary').trim(),
    )
    await accentPage.evaluate(() => {
      const root = document.documentElement
      root.setAttribute('data-accent', 'custom')
      let tag = document.getElementById('ccr-custom-accent')
      if (!tag) {
        tag = document.createElement('style')
        tag.id = 'ccr-custom-accent'
        document.head.appendChild(tag)
      }
      tag.textContent = `:root,[data-theme='dark']{--color-accent-primary:#3b82f6;--color-accent-primary-rgb:59 130 246;}`
    })
    const accentAfter = await accentPage.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue('--color-accent-primary').trim(),
    )
    rows.push({
      page: 'Settings',
      path: '/settings',
      combo: 'dark×clay custom-accent-spot',
      accentBefore,
      accentAfter,
      followed: accentAfter !== accentBefore,
    })
    await accentContext.close()
  } finally {
    await browser.close()
  }

  await writeFile(path.join(outDir, 'metrics.json'), JSON.stringify(rows, null, 2), 'utf8')
  console.log(JSON.stringify({ rows: rows.length, outDir }, null, 2))
}

main().catch((error) => {
  console.error(error)
  process.exit(1)
})
