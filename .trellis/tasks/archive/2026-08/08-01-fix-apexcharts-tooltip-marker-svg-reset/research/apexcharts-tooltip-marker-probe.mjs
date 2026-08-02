import { createRequire } from 'node:module'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = resolve(SCRIPT_DIR, '../../../..')
const requireFromUi = createRequire(join(REPO_ROOT, 'ccr-ui', 'package.json'))
const { chromium } = requireFromUi('playwright')

const args = process.argv.slice(2)
const optionValue = (name) => {
  const index = args.indexOf(name)
  return index >= 0 ? args[index + 1] : undefined
}

const cdpEndpoint = optionValue('--cdp') ?? 'http://127.0.0.1:9223'
const targetPath = optionValue('--path') ?? '/usage'
const screenshotPath = optionValue('--screenshot')
const blockRuntimeCss = args.includes('--block-runtime-css')
const sessionFlag = '__ccr_probe_block_apexcharts_css__'

const resolveWebSocketUrl = async (endpoint) => {
  if (endpoint.startsWith('ws://') || endpoint.startsWith('wss://')) {
    return endpoint
  }

  const response = await fetch(`${endpoint.replace(/\/$/, '')}/json/version`)
  if (!response.ok) {
    throw new Error(`CDP version endpoint returned HTTP ${response.status}`)
  }

  const version = await response.json()
  if (!version.webSocketDebuggerUrl) {
    throw new Error('CDP version response has no webSocketDebuggerUrl')
  }

  return version.webSocketDebuggerUrl
}

const findTargetPage = (browser) => {
  const pages = browser.contexts().flatMap((context) => context.pages())
  return pages.find((page) => {
    try {
      return new URL(page.url()).pathname === targetPath
    } catch {
      return false
    }
  })
}

const installRuntimeCssBlocker = async (page) => {
  await page.evaluate((key) => sessionStorage.setItem(key, '1'), sessionFlag)
  await page.addInitScript((key) => {
    if (sessionStorage.getItem(key) !== '1') return

    const originalAppendChild = Node.prototype.appendChild
    Node.prototype.appendChild = function appendChild(node) {
      if (node instanceof HTMLStyleElement && node.id === 'apexcharts-css') {
        window.__ccrBlockedApexStyleCount = (window.__ccrBlockedApexStyleCount ?? 0) + 1
        return node
      }
      return originalAppendChild.call(this, node)
    }
  }, sessionFlag)
}

const inspectPage = (page) => page.evaluate(() => {
  const chart = document.querySelector('.distribution-card__chart')
  const runtimeStyle = document.getElementById('apexcharts-css')
  const markers = Array.from(
    chart?.querySelectorAll('.apexcharts-tooltip-marker svg') ?? [],
  )
  const groups = Array.from(
    chart?.querySelectorAll('.apexcharts-tooltip-series-group') ?? [],
  )
  const tooltip = chart?.querySelector('.apexcharts-tooltip') ?? null

  const rectOf = (element) => {
    if (!element) return null
    const rect = element.getBoundingClientRect()
    return {
      x: Number(rect.x.toFixed(2)),
      y: Number(rect.y.toFixed(2)),
      width: Number(rect.width.toFixed(2)),
      height: Number(rect.height.toFixed(2)),
    }
  }

  const ruleSources = []
  const inaccessibleStyleSheets = []
  for (const sheet of document.styleSheets) {
    try {
      const rules = Array.from(sheet.cssRules)
      if (!rules.some((rule) => rule.cssText.includes('.apexcharts-tooltip-marker'))) {
        continue
      }

      const owner = sheet.ownerNode
      ruleSources.push({
        href: sheet.href,
        ownerTag: owner?.nodeName ?? null,
        ownerId: owner instanceof HTMLElement ? owner.id || null : null,
        viteDevId:
          owner instanceof HTMLElement ? owner.dataset.viteDevId ?? null : null,
        ruleCount: rules.length,
      })
    } catch {
      inaccessibleStyleSheets.push({
        href: sheet.href,
      })
    }
  }

  let runtimeRuleCount = null
  try {
    runtimeRuleCount = runtimeStyle?.sheet?.cssRules.length ?? null
  } catch {
    runtimeRuleCount = null
  }

  const markerDetails = markers.map((marker) => {
    const host = marker.parentElement
    const group = host?.parentElement
    const markerStyle = getComputedStyle(marker)
    const hostStyle = host ? getComputedStyle(host) : null
    const groupStyle = group ? getComputedStyle(group) : null
    return {
      markerRect: rectOf(marker),
      markerWidth: markerStyle.width,
      markerHeight: markerStyle.height,
      markerMaxWidth: markerStyle.maxWidth,
      hostWidth: hostStyle?.width ?? null,
      hostHeight: hostStyle?.height ?? null,
      hostDisplay: hostStyle?.display ?? null,
      hostInlineDisplay: host?.style.display || null,
      groupDisplay: groupStyle?.display ?? null,
      groupInlineDisplay: group?.style.display || null,
    }
  })

  const giantMarkers = markerDetails.filter(({ markerRect }) =>
    markerRect && (markerRect.width > 32 || markerRect.height > 32),
  )
  const groupsHidden = groups.length > 0
    && groups.every((group) => getComputedStyle(group).display === 'none')
  const tooltipStyle = tooltip ? getComputedStyle(tooltip) : null
  const markerHostsSized = markerDetails.length > 0
    && markerDetails.every(({ hostWidth, hostHeight }) =>
      hostWidth === '12px' && hostHeight === '12px',
    )

  const checks = {
    chartMounted: Boolean(chart?.querySelector('.apexcharts-canvas')),
    markersCreated: markers.length > 0,
    noGiantMarkers: giantMarkers.length === 0,
    markerHostsSized,
    tooltipGroupsInitiallyHidden: groupsHidden,
    tooltipIsPositionedOverlay: tooltipStyle?.position === 'absolute',
    criticalCssRuleAvailable: ruleSources.length > 0,
  }

  return {
    url: location.href,
    mode: sessionStorage.getItem('__ccr_probe_block_apexcharts_css__') === '1'
      ? 'block-runtime-css'
      : 'normal',
    runtimeStyle: {
      present: Boolean(runtimeStyle),
      textLength: runtimeStyle?.textContent?.length ?? 0,
      ruleCount: runtimeRuleCount,
      disabled: runtimeStyle?.sheet?.disabled ?? null,
      blockedAppendCount: window.__ccrBlockedApexStyleCount ?? 0,
    },
    ruleSources,
    inaccessibleStyleSheets,
    chartCount: document.querySelectorAll('.apexcharts-canvas').length,
    markerCount: markers.length,
    giantMarkerCount: giantMarkers.length,
    largestMarker: markerDetails.reduce((largest, marker) => {
      if (!marker.markerRect) return largest
      if (!largest || marker.markerRect.width > largest.width) return marker.markerRect
      return largest
    }, null),
    firstMarker: markerDetails[0] ?? null,
    tooltip: {
      rect: rectOf(tooltip),
      display: tooltipStyle?.display ?? null,
      position: tooltipStyle?.position ?? null,
      opacity: tooltipStyle?.opacity ?? null,
    },
    checks,
    healthy: Object.values(checks).every(Boolean),
  }
})

let page
let report
let exitCode = 1

try {
  const websocketUrl = await resolveWebSocketUrl(cdpEndpoint)
  const browser = await chromium.connectOverCDP(websocketUrl, { timeout: 15_000 })
  page = findTargetPage(browser)
  if (!page) {
    const urls = browser.contexts().flatMap((context) =>
      context.pages().map((candidate) => candidate.url()),
    )
    throw new Error(`No CDP page matched ${targetPath}; found: ${urls.join(', ')}`)
  }

  if (blockRuntimeCss) {
    await installRuntimeCssBlocker(page)
  }

  await page.reload({ waitUntil: 'domcontentloaded', timeout: 20_000 })
  await page.waitForFunction(() =>
    Boolean(document.querySelector(
      '.distribution-card__chart .apexcharts-tooltip-marker svg',
    )), null, { timeout: 20_000 })

  report = await inspectPage(page)
  if (screenshotPath) {
    await page.screenshot({ path: resolve(screenshotPath), fullPage: true })
  }
  exitCode = report.healthy ? 0 : 1
} catch (error) {
  report = {
    healthy: false,
    error: error instanceof Error ? error.stack ?? error.message : String(error),
  }
} finally {
  if (page && blockRuntimeCss) {
    try {
      await page.evaluate((key) => sessionStorage.removeItem(key), sessionFlag)
      await page.reload({ waitUntil: 'domcontentloaded', timeout: 20_000 })
    } catch (cleanupError) {
      report.cleanupError = cleanupError instanceof Error
        ? cleanupError.message
        : String(cleanupError)
      exitCode = 1
    }
  }
}

console.log(JSON.stringify(report, null, 2))
console.log(report.healthy ? '[PASS] ApexCharts tooltip CSS contract is healthy' : '[FAIL] ApexCharts tooltip CSS contract is broken')
process.exit(exitCode)
