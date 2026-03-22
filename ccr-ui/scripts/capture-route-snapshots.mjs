/* eslint-disable no-console */
import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { chromium } from 'playwright'

const baseUrl = process.env.PLAYWRIGHT_BASE_URL || 'http://127.0.0.1:5173'
const outputRoot = path.resolve(process.cwd(), 'tests/artifacts/route-snapshots')

const routes = [
  { slug: 'home', path: '/' },
  { slug: 'skills', path: '/skills' },
  { slug: 'skills-add', path: '/skills/add' },
  { slug: 'usage', path: '/usage' },
  { slug: 'sync', path: '/sync' },
  { slug: 'commands', path: '/commands' },
]

const modes = [
  { name: 'light', theme: 'light', reducedMotion: 'no-preference' },
  { name: 'dark', theme: 'dark', reducedMotion: 'no-preference' },
  { name: 'light-reduced-motion', theme: 'light', reducedMotion: 'reduce' },
]

async function ensureDir(dir) {
  await mkdir(dir, { recursive: true })
}

async function main() {
  const browser = await chromium.launch({ headless: true })
  const manifest = {
    generatedAt: new Date().toISOString(),
    baseUrl,
    routes,
    modes,
    captures: [],
  }

  try {
    for (const mode of modes) {
      const modeDir = path.join(outputRoot, mode.name)
      await ensureDir(modeDir)

      const context = await browser.newContext({
        viewport: { width: 1440, height: 1080 },
        colorScheme: mode.theme,
        reducedMotion: mode.reducedMotion,
      })

      await context.addInitScript((theme) => {
        localStorage.setItem('ccr-theme', theme)
        document.documentElement.classList.toggle('dark', theme === 'dark')
        document.documentElement.setAttribute('data-theme', theme)
      }, mode.theme)

      const page = await context.newPage()

      for (const route of routes) {
        const routeUrl = new URL(route.path, baseUrl).toString()
        const screenshotPath = path.join(modeDir, `${route.slug}.png`)

        try {
          await page.goto(routeUrl, { waitUntil: 'domcontentloaded', timeout: 15000 })
          await page.waitForTimeout(1500)
          await page.screenshot({ path: screenshotPath, fullPage: true })
          manifest.captures.push({
            mode: mode.name,
            route: route.path,
            file: path.relative(process.cwd(), screenshotPath).replaceAll('\\', '/'),
            ok: true,
          })
        } catch (error) {
          manifest.captures.push({
            mode: mode.name,
            route: route.path,
            ok: false,
            error: error instanceof Error ? error.message : String(error),
          })
        }
      }

      await context.close()
    }
  } finally {
    await browser.close()
  }

  await ensureDir(outputRoot)
  await writeFile(
    path.join(outputRoot, 'manifest.json'),
    JSON.stringify(manifest, null, 2),
    'utf8'
  )

  const failures = manifest.captures.filter((capture) => !capture.ok)
  if (failures.length > 0) {
    console.error(`Snapshot capture completed with ${failures.length} failures`)
    process.exitCode = 1
    return
  }

  console.log(`Captured ${manifest.captures.length} route snapshots into ${outputRoot}`)
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})
