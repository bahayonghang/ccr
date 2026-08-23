#!/usr/bin/env node
/* eslint-disable no-console -- 测量脚本，与 check-bundle-budget.mjs 同一豁免模式 */
// 批次1 分布测量（08-22-arch-quality-perf）
// 统计 src/**/*.{ts,tsx}（排除 src/types/generated）与历史 .vue 的：
// 行数、圈复杂度、最大嵌套深度、最大参数个数。
// 只做测量，不改任何规则配置；结果输出为 JSON，供 distribution.md 取数。
import fs from 'node:fs'
import path from 'node:path'
import { execFileSync } from 'node:child_process'

const root = process.cwd()
const SRC = path.join(root, 'src')

const walk = (dir, exts, out = []) => {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name)
    if (e.isDirectory()) {
      if (e.name === 'generated' && p.includes('types')) continue
      walk(p, exts, out)
    } else if (exts.some((x) => e.name.endsWith(x))) out.push(p)
  }
  return out
}

const lineCount = (p) => fs.readFileSync(p, 'utf8').split('\n').length

// 圈复杂度/嵌套深度/参数个数：用临时 ESLint 配置以 warning 跑一遍取数据
const tmpConfig = path.join(root, '.eslint.distribution.mjs')
fs.writeFileSync(
  tmpConfig,
  `import base from './eslint.config.js'
export default [
  ...base,
  {
    rules: {
      complexity: ['warn', { max: 0 }],
      'max-depth': ['warn', { max: 0 }],
      'max-params': ['warn', { max: 0 }],
    },
  },
]
`,
)
let lintData = {}
try {
  const files = [...walk(SRC, ['.ts', '.tsx'])]
  const json = execFileSync(
    'bunx',
    ['eslint', '--config', tmpConfig, '--format', 'json', ...files],
    { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 },
  )
  for (const entry of JSON.parse(json)) {
    const m = { complexity: [], depth: [], params: [] }
    for (const msg of entry.messages) {
      if (msg.ruleId === 'complexity') m.complexity.push(Number(msg.message.match(/complexity of (\d+)/)?.[1] ?? 0))
      if (msg.ruleId === 'max-depth') m.depth.push(Number(msg.message.match(/\((\d+)\)\. Maximum/)?.[1] ?? 1))
      if (msg.ruleId === 'max-params') m.params.push(Number(msg.message.match(/too many parameters \((\d+)\)/)?.[1] ?? 0))
    }
    lintData[path.relative(root, entry.filePath).replaceAll('\\', '/')] = m
  }
} finally {
  fs.rmSync(tmpConfig, { force: true })
}

const quantiles = (arr, qs = [0.5, 0.75, 0.9, 0.95]) => {
  if (arr.length === 0) return {}
  const s = [...arr].sort((a, b) => a - b)
  const out = {}
  for (const q of qs) out[`P${q * 100}`] = s[Math.min(s.length - 1, Math.floor(q * s.length))]
  out.max = s[s.length - 1]
  out.mean = Number((s.reduce((a, b) => a + b, 0) / s.length).toFixed(1))
  return out
}

const buildSet = (prefixes) => {
  const rows = []
  for (const [f, m] of Object.entries(lintData)) {
    if (!prefixes.some((p) => f.startsWith(p))) continue
    rows.push({
      file: f,
      lines: lineCount(path.join(root, f)),
      maxComplexity: Math.max(0, ...m.complexity),
      maxDepth: Math.max(0, ...m.depth),
      maxParams: Math.max(0, ...m.params),
    })
  }
  return rows
}

const UNIFY_EXCLUDED = [
  // 18 收敛文件（platform-unify 批次1 清单）+ 3 个 views/generic base 本体
  'ClaudeCodeSettingsView', 'CodexSettingsView', 'GrokSettingsView', 'OpenCodeSettingsView',
  'ClaudeCodeProfilesView', 'CodexProfilesView', 'GrokProfilesView',
  'ClaudeAuthView', 'CodexAuthView', 'GrokAuthView',
  'CommandsView', 'OpenCodeCommandsView', 'CodexMcpView', 'OpenCodeMcpView',
  'CodexAgentsView', 'OpenCodeAgentsView', 'PluginsView', 'OpenCodePluginsView',
  'views/generic/AgentsView', 'views/generic/PlatformMcpView', 'views/generic/PlatformPluginsView',
]

const tsRows = buildSet(['src/'])
const liveRows = tsRows.filter((r) => !UNIFY_EXCLUDED.some((u) => r.file.includes(u)))
const vueFiles = walk(SRC, ['.vue'])
const vueLines = vueFiles.map(lineCount)

const report = {
  generatedAt: new Date().toISOString(),
  liveTsTs: {
    note: 'src/**/*.{ts,tsx} 排除 src/types/generated 与 21 个 platform-unify 文件后的暂定分布',
    fileCount: liveRows.length,
    lines: quantiles(liveRows.map((r) => r.lines)),
    complexity: quantiles(liveRows.map((r) => r.maxComplexity).filter((v) => v > 0)),
    depth: quantiles(liveRows.map((r) => r.maxDepth).filter((v) => v > 0)),
    params: quantiles(liveRows.map((r) => r.maxParams).filter((v) => v > 0)),
  },
  fullTsTsBeforeExclusion: {
    note: '排除前的全量 .ts/.tsx（仍不含 generated）',
    fileCount: tsRows.length,
    lines: quantiles(tsRows.map((r) => r.lines)),
    complexity: quantiles(tsRows.map((r) => r.maxComplexity).filter((v) => v > 0)),
    depth: quantiles(tsRows.map((r) => r.maxDepth).filter((v) => v > 0)),
    params: quantiles(tsRows.map((r) => r.maxParams).filter((v) => v > 0)),
  },
  vueHistorical: {
    note: '.vue 历史分布（已整体退出 lint 管线），仅作上下文',
    fileCount: vueFiles.length,
    lines: quantiles(vueLines),
  },
  top20ByLines: [...liveRows].sort((a, b) => b.lines - a.lines).slice(0, 20),
}
console.log(JSON.stringify(report, null, 2))
