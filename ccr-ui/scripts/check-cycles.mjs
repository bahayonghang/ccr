#!/usr/bin/env node
/* eslint-disable no-console */
// 循环依赖检查（08-22-arch-quality-perf 批次 2，AC3）
// 默认：扫描 src/**/*.{ts,tsx}（排除 types/generated），发现循环即退出码 1。
// --self-check：定向扫描 tests/fixtures/arch-violations 的 cycle-a/cycle-b 夹具，
//   断言恰好检出 1 个循环（夹具本身构成 a→b→a）。
import fs from 'node:fs'
import path from 'node:path'
import { parseDependencyTree, parseCircular } from 'dpdm'

const root = process.cwd()
const selfCheck = process.argv.includes('--self-check')

const walk = (dir, out = []) => {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name)
    if (e.isDirectory()) {
      if (e.name === 'generated' && p.includes('types')) continue
      walk(p, out)
    } else if (/\.(ts|tsx)$/.test(e.name)) {
      out.push(p)
    }
  }
  return out
}

const entries = selfCheck
  ? [
      path.join(root, 'tests/fixtures/arch-violations/cycle-a.ts'),
      path.join(root, 'tests/fixtures/arch-violations/cycle-b.ts'),
    ]
  : walk(path.join(root, 'src'))

// dpdm 4.x：parseDependencyTree 返回 DependencyTree（Record<string, Dependency[]>），
// 循环清单需用 parseCircular(tree, skipDynamicImports) 计算（skipDynamicImports=true 跳过动态导入边，
// 等价 CLI 的 --skip-dynamic-imports circular 语义）。
const tree = await parseDependencyTree(entries, {
  tsconfig: path.join(root, 'tsconfig.json'),
  transform: true,
  skipDynamicImports: false,
})

const circulars = parseCircular(tree, true)
if (!selfCheck && circulars.length === 0) {
  console.log(`[check-cycles] PASS：${entries.length} 个文件，无循环依赖`)
  process.exit(0)
}
if (!selfCheck) {
  for (const c of circulars) console.error(`[check-cycles] 循环依赖：${c.join(' -> ')}`)
  console.error(`[check-cycles] FAIL：检出 ${circulars.length} 个循环依赖`)
  process.exit(1)
}

// 自检模式：夹具必须恰好产生一个循环
const ok =
  circulars.length === 1 &&
  circulars[0].length === 2 &&
  circulars[0].some((f) => f.endsWith('cycle-a.ts')) &&
  circulars[0].some((f) => f.endsWith('cycle-b.ts'))
if (!ok) {
  console.error(`[check-cycles] 自检失败：期望夹具循环 cycle-a <-> cycle-b，实际 ${JSON.stringify(circulars)}`)
  process.exit(1)
}
console.log('[check-cycles] 自检通过：循环检测器对已知循环夹具正确报错')
