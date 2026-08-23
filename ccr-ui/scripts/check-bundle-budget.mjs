#!/usr/bin/env node
/* eslint-disable no-console */

// Bundle 体积预算（React 基座）
//
// 取值方法见 .trellis/tasks/08-22-arch-quality-perf/bundle-budget.md：
// - 每项预算 = 实测体积 + 余量；与 Vue 基线（.trellis/tasks/08-22-react-migration/baseline/bundle-budget.txt）一一对应的项以 Vue 基线为参考上限。
// - motion 13.1.1 与 zod 4.4.3 单列两行（design.md §8 R9.1）：记录实际增量与预留值。
//   当前二者均未被应用代码导入（marker 检索确认），实际增量 = 0；预留值来自 rolldown 实测
//   （临时 scratch 入口构建，测后即删，未改动应用源码），见 bundle-budget.md §4。
// - 预留行在对应包被导入后须以真实消耗数更新，见 bundle-budget.md §4 的说明。

import fs from 'node:fs/promises'
import path from 'node:path'
import { gzipSync } from 'node:zlib'

const assetsDir = path.resolve(process.cwd(), 'dist/assets')
const distIndexPath = path.resolve(process.cwd(), 'dist/index.html')

const kib = (bytes) => (bytes / 1024).toFixed(2)

const ensureAssetsDir = async () => {
  try {
    const stat = await fs.stat(assetsDir)
    if (!stat.isDirectory()) {
      throw new Error('not a directory')
    }
  } catch {
    console.error(`[bundle-budget] Missing dist assets directory: ${assetsDir}`)
    process.exit(1)
  }
}

const findChunk = async (files, prefix, extension = '.js') => {
  const candidates = files.filter((file) => file.startsWith(prefix) && file.endsWith(extension))
  if (candidates.length === 0) return null
  let largest = null
  let largestSize = -1

  for (const file of candidates) {
    const stat = await fs.stat(path.join(assetsDir, file))
    if (stat.size > largestSize) {
      largest = file
      largestSize = stat.size
    }
  }

  return largest
}

const getBufferMetrics = async (fileName, absPath) => {
  const content = await fs.readFile(absPath)
  return {
    fileName,
    size: content.byteLength,
    gzipSize: gzipSync(content, { level: 9 }).byteLength,
  }
}

const getFileMetrics = async (fileName) => {
  return getBufferMetrics(fileName, path.join(assetsDir, fileName))
}

const getStartupFontCssMetrics = async () => {
  const html = await fs.readFile(distIndexPath, 'utf8')
  const hrefs = [...html.matchAll(/href="(\/fonts\/maplebright\/[^"]+\/startup\.css)"/g)]
    .map((match) => match[1])

  const uniqueHrefs = [...new Set(hrefs)]
  const buffers = await Promise.all(
    uniqueHrefs.map(async (href) => {
      const absPath = path.resolve(process.cwd(), 'public', href.replace('/fonts/', 'fonts/'))
      return fs.readFile(absPath)
    }),
  )

  const merged = Buffer.concat(buffers)
  return {
    fileName: uniqueHrefs.join(', ') || '(none)',
    size: merged.byteLength,
    gzipSize: gzipSync(merged, { level: 9 }).byteLength,
  }
}

const validateBudget = (name, metrics, budget) => {
  const errors = []
  if (metrics.size > budget.maxBytes) {
    errors.push(`${name} raw ${kib(metrics.size)} KiB > ${kib(budget.maxBytes)} KiB`)
  }
  if (typeof budget.maxGzipBytes === 'number' && metrics.gzipSize > budget.maxGzipBytes) {
    errors.push(`${name} gzip ${kib(metrics.gzipSize)} KiB > ${kib(budget.maxGzipBytes)} KiB`)
  }
  return errors
}

// 已知 vendor 分组前缀（manualChunks 分组名 + rolldown-runtime）。除这些与入口 index
// 之外剩下的 .js chunk 视为懒加载应用 chunk；当前 React 壳层尚无懒加载，此值 = 0，
// 待 08-22-shell-port 引入 React.lazy 后自然生效。
const VENDOR_PREFIXES = [
  'react-vendor-',
  'query-vendor-',
  'ui-vendor-',
  'charts-vendor-',
  'i18n-vendor-',
  'markdown-vendor-',
  'search-vendor-',
  'tauri-vendor-',
  'virtual-vendor-',
  'term-vendor-',
  'motion-vendor-',
  'form-vendor-',
  'rolldown-runtime-',
]

const findLargestLazyChunk = async (files) => {
  const entryPrefix = await findChunk(files, 'index-')
  let largest = null
  let largestSize = -1

  for (const file of files) {
    if (!file.endsWith('.js')) continue
    if (file === entryPrefix) continue
    if (VENDOR_PREFIXES.some((prefix) => file.startsWith(prefix))) continue
    const stat = await fs.stat(path.join(assetsDir, file))
    if (stat.size > largestSize) {
      largest = file
      largestSize = stat.size
    }
  }

  return largest
}

// 预留包检查：marker 检索 + 专用 chunk 归属。
// 返回 { actualBytes, note }；actualBytes 为可归因体积（专用 chunk）或 0（无 marker）。
const checkReservedPackage = async (files, { name, marker, dedicatedPrefix }) => {
  const dedicated = dedicatedPrefix
    ? await findChunk(files, dedicatedPrefix)
    : null

  if (dedicated) {
    const metrics = await getFileMetrics(dedicated)
    return {
      actualBytes: metrics.size,
      actualGzip: metrics.gzipSize,
      fileName: dedicated,
      note: `专用 chunk ${dedicated}`,
    }
  }

  // 无专用 chunk：检查 marker 是否出现在任何应用 chunk 中
  const hits = []
  for (const file of files) {
    if (!file.endsWith('.js')) continue
    const content = await fs.readFile(path.join(assetsDir, file), 'utf8')
    if (content.includes(marker)) {
      hits.push(file)
    }
  }

  if (hits.length > 0) {
    return {
      actualBytes: null,
      actualGzip: null,
      fileName: hits.join(', '),
      note: `marker「${marker}」出现于 ${hits.length} 个 chunk，无法精确归因；` +
        `请在 ${name} 导入后以真实消耗数更新本行（bundle-budget.md §4）`,
    }
  }

  return { actualBytes: 0, actualGzip: 0, fileName: '(none)', note: '未导入（marker 检索确认）' }
}

await ensureAssetsDir()
const files = await fs.readdir(assetsDir)

const entryChunk = await findChunk(files, 'index-')
if (!entryChunk) {
  console.error('[bundle-budget] Missing index chunk in dist/assets')
  process.exit(1)
}

const entryCssChunk = await findChunk(files, 'index-', '.css')
if (!entryCssChunk) {
  console.error('[bundle-budget] Missing index CSS chunk in dist/assets')
  process.exit(1)
}

const largestLazyChunk = await findLargestLazyChunk(files)

const entryMetrics = await getFileMetrics(entryChunk)
const entryCssMetrics = await getFileMetrics(entryCssChunk)
const shellIconMetrics = await getBufferMetrics(
  'solarShellIconSubset.ts',
  path.resolve(process.cwd(), 'src/config/solarShellIconSubset.ts'),
)
const startupFontCssMetrics = await getStartupFontCssMetrics()

const queryVendorChunk = await findChunk(files, 'query-vendor-')
const reactVendorChunk = await findChunk(files, 'react-vendor-')

const lazyMetrics = largestLazyChunk ? await getFileMetrics(largestLazyChunk) : null
const queryVendorMetrics = queryVendorChunk ? await getFileMetrics(queryVendorChunk) : null
const reactVendorMetrics = reactVendorChunk ? await getFileMetrics(reactVendorChunk) : null

// 预算表（raw / gzip 均以 KiB 为单位的整数阈值）
//
// | 项 | Vue 基线 raw/gz | React 实测 raw/gz | 预算 raw/gz | 依据 |
// | index | 243.69 / 45.41 | 139.46 / 9.70 | 256 / 48 | 与 Vue index 一一对应，以其为参考上限 |
// | react-vendor | — | 264.51 / 82.79 | 320 / 96 | 实测 ×1.21；react-dom 稳定，增长空间小 |
// | query-vendor | — | 31.51 / 9.67 | 64 / 20 | 实测 ×2；视图迁移后 query 使用面会扩大 |
// | 最大懒加载 chunk | 93.40 / 26.51 | 0（无懒加载） | 128 / 40 | 以 Vue UsageDashboardView 为参考；尚无则放行 |
// | core.css | 123.13 / 19.35 | 198.20 / 28.22 | 240 / 36 | 实测 +21%；v4 首屏 CSS 增长已记录（见 code-splitting.md §3） |
// | shell-icons | 24.19 / 7.73 | 24.19 / 7.73 | 40 / 12 | 文件未变，沿用旧预算 |
// | startup-font-css | 0.00 / 0.02 | 0.00 / 0.02 | 150 / — | 字体声明仍为惰性加载，沿用旧预算 |
const BUDGETS = {
  index: { maxBytes: 256 * 1024, maxGzipBytes: 48 * 1024 },
  'react-vendor': { maxBytes: 320 * 1024, maxGzipBytes: 96 * 1024 },
  'query-vendor': { maxBytes: 64 * 1024, maxGzipBytes: 20 * 1024 },
  'largest-lazy': { maxBytes: 128 * 1024, maxGzipBytes: 40 * 1024 },
  'core.css': { maxBytes: 240 * 1024, maxGzipBytes: 36 * 1024 },
  'shell-icons': { maxBytes: 40 * 1024, maxGzipBytes: 12 * 1024 },
  'startup-font-css': { maxBytes: 150 * 1024 },
}

// 预留行（R9.1）：motion 13.1.1 / zod 4.4.3。当前未导入，实际增量 = 0；
// 预留值来自 rolldown 实测典型使用面（motion 121.89 / 39.19，zod 62.50 / 16.62）取整，
// zod 另与 zod-pilot 实测 +59.4/15.6 交叉验证（bundle-budget.md §4）。
const RESERVED = [
  {
    name: 'motion',
    marker: 'AnimatePresence',
    dedicatedPrefix: 'motion-vendor-',
    maxBytes: 128 * 1024,
    maxGzipBytes: 44 * 1024,
    note: '预留值 128 / 44 KiB：rolldown 实测 motion/react + AnimatePresence 典型使用面 121.89 raw / 39.19 gzip 取整',
  },
  {
    name: 'zod',
    marker: 'ZodError',
    dedicatedPrefix: 'form-vendor-',
    maxBytes: 64 * 1024,
    maxGzipBytes: 20 * 1024,
    note: '预留值 64 / 20 KiB：zod-pilot 实测 +59.4 raw / +15.6 gzip（真实构建）与 rolldown 实测 62.50 / 16.62 交叉验证后取整',
  },
]

const failures = []
const report = []

const addFailure = (name, metrics, budget) => {
  const errs = validateBudget(name, metrics, budget)
  if (errs.length > 0) {
    failures.push(...errs)
  }
}

// 主条目：按名查 chunk，缺失即失败（该分组应存在）
const mandatoryEntries = [
  ['index', entryMetrics, BUDGETS.index],
]
if (reactVendorMetrics) {
  mandatoryEntries.push(['react-vendor', reactVendorMetrics, BUDGETS['react-vendor']])
} else {
  failures.push('react-vendor chunk 缺失（react/react-dom/react-router 未归入 vendor 分组）')
}
if (queryVendorMetrics) {
  mandatoryEntries.push(['query-vendor', queryVendorMetrics, BUDGETS['query-vendor']])
} else {
  failures.push('query-vendor chunk 缺失（@tanstack/react-query/query-core 未归入 vendor 分组）')
}

for (const [name, metrics, budget] of mandatoryEntries) {
  addFailure(name, metrics, budget)
  report.push(`${name}: ${metrics.fileName} raw=${kib(metrics.size)} KiB gzip=${kib(metrics.gzipSize)} KiB`)
}

// 最大懒加载 chunk：当前无懒加载则放行（report 标注）
if (lazyMetrics) {
  addFailure('largest-lazy', lazyMetrics, BUDGETS['largest-lazy'])
  report.push(`largest-lazy: ${lazyMetrics.fileName} raw=${kib(lazyMetrics.size)} KiB gzip=${kib(lazyMetrics.gzipSize)} KiB`)
} else {
  report.push('largest-lazy: (none) 当前 React 壳层无懒加载 chunk，放行')
}

addFailure('core.css', entryCssMetrics, BUDGETS['core.css'])
addFailure('shell-icons', shellIconMetrics, BUDGETS['shell-icons'])
addFailure('startup-font-css', startupFontCssMetrics, BUDGETS['startup-font-css'])
report.push(`core.css: ${entryCssMetrics.fileName} raw=${kib(entryCssMetrics.size)} KiB gzip=${kib(entryCssMetrics.gzipSize)} KiB`)
report.push(`shell-icons: ${shellIconMetrics.fileName} raw=${kib(shellIconMetrics.size)} KiB gzip=${kib(shellIconMetrics.gzipSize)} KiB`)
report.push(`startup-font-css: ${startupFontCssMetrics.fileName} raw=${kib(startupFontCssMetrics.size)} KiB gzip=${kib(startupFontCssMetrics.gzipSize)} KiB`)

// 预留行（R9.1）
for (const reserved of RESERVED) {
  const result = await checkReservedPackage(files, reserved)
  if (result.actualBytes === null) {
    // marker 出现但无法归因 → 必须更新真实消耗数，否则视为超预算
    failures.push(
      `${reserved.name}: 已导入但无法归因体积（${result.note}），请以真实消耗数更新预留行`,
    )
    report.push(
      `${reserved.name}: (inlined) ${result.note}；预留 ${kib(reserved.maxBytes)} / ${kib(reserved.maxGzipBytes)} KiB`,
    )
  } else {
    const errs = []
    if (result.actualBytes > reserved.maxBytes) {
      errs.push(
        `${reserved.name} raw ${kib(result.actualBytes)} KiB > 预留 ${kib(reserved.maxBytes)} KiB`,
      )
    }
    if (result.actualGzip > reserved.maxGzipBytes) {
      errs.push(
        `${reserved.name} gzip ${kib(result.actualGzip)} KiB > 预留 ${kib(reserved.maxGzipBytes)} KiB`,
      )
    }
    failures.push(...errs)
    report.push(
      `${reserved.name}: ${result.fileName} actual=${kib(result.actualBytes)} KiB / ${kib(result.actualGzip)} KiB ` +
        `reserved=${kib(reserved.maxBytes)} KiB / ${kib(reserved.maxGzipBytes)} KiB（${reserved.note}）`,
    )
  }
}

for (const line of report) {
  console.log(`[bundle-budget] ${line}`)
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`[bundle-budget] FAIL ${failure}`)
  }
  process.exit(1)
}

console.log('[bundle-budget] PASS all budgets satisfied')
