#!/usr/bin/env node
/* eslint-disable no-console */
// 组件内样式行数检查（08-22-arch-quality-perf 批次 3）
// 目标：只针对 React 侧 `.tsx` + `.module.css`（.vue 已整体退出管线，阶段 4–5 离开树，不在此检查）。
// 依据：design.md §3.1 样式行数 P90=412（139 个历史 .vue 组件）与父任务 design.md §6 的比例约束
//       「单组件局部样式行数不超过其 JSX 行数」。阈值/比例定义见 thresholds.md。
//
// 计数规则：
//   styleLines  = `.module.css` 的物理行数（split('\n').length，与分布测量口径一致）。
//   jsxLines    = 消费该样式的 `.tsx` 文件的非空、非纯注释源码行数（简单代理 JSX 行数：
//                 解析 JSX 根到闭合的精确计数对大多数组件等价，且对拆分场景更稳——组件拆分后
//                 tsx 行数与样式行数同时收缩；此口径记录在脚本头，作为约定基准）。
//   配对规则     = 同目录同名 `.tsx`（Foo.module.css ↔ Foo.tsx）优先；否则解析目录内所有 `.tsx`
//                的 import 语句，按 './*.module.css' 引用匹配。
// 两个约束：
//   (a) 绝对上限：styleLines <= 412；
//   (b) 比例约束：styleLines <= jsxLines（样式行数不超过消费组件 JSX 行数）。
// 任一违反即输出违规清单并退出码 1。当前树无 `.module.css`，应零违规通过。
import fs from 'node:fs'
import path from 'node:path'

const root = process.cwd()
const SRC = path.join(root, 'src')
const MAX_STYLE_LINES = 412

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

const physicalLines = (p) => fs.readFileSync(p, 'utf8').split('\n').length

// 非空、非纯注释的源码行数（JSX 行数代理）
const jsxProxyLines = (p) =>
  fs.readFileSync(p, 'utf8')
    .split('\n')
    .filter((l) => {
      const t = l.trim()
      if (t === '') return false
      if (t.startsWith('//') || t.startsWith('*') || t.startsWith('/*')) return false
      return true
    }).length

const cssFiles = walk(SRC, ['.module.css'])
const tsxFiles = walk(SRC, ['.tsx'])

const violations = []

for (const cssPath of cssFiles) {
  const dir = path.dirname(cssPath)
  const base = path.basename(cssPath, '.module.css')
  // 配对：同目录同名 .tsx 优先
  let consumer = tsxFiles.find((f) => path.dirname(f) === dir && path.basename(f, '.tsx') === base)
  if (!consumer) {
    // 回退：解析同目录 .tsx 的 import './*.module.css'
    consumer = tsxFiles.find((f) => {
      if (path.dirname(f) !== dir) return false
      const src = fs.readFileSync(f, 'utf8')
      const cssRel = path.relative(dir, cssPath).replaceAll('\\', '/')
      const re = new RegExp(`import[^;]*from\\s+['"]\\./${base}\\.module\\.css['"]`)
      return re.test(src) || src.includes(`'./${cssRel}'`) || src.includes(`"./${cssRel}"`)
    })
  }
  const styleLines = physicalLines(cssPath)
  const jsxLines = consumer ? jsxProxyLines(consumer) : 0
  const rel = path.relative(root, cssPath).replaceAll('\\', '/')

  if (styleLines > MAX_STYLE_LINES) {
    violations.push(
      `[style-lines] ${rel}：样式 ${styleLines} 行 > 绝对上限 ${MAX_STYLE_LINES}`,
    )
  }
  if (consumer && styleLines > jsxLines) {
    violations.push(
      `[style-lines] ${rel}：样式 ${styleLines} 行 > 消费组件 ${path
        .relative(root, consumer)
        .replaceAll('\\', '/')} 的 JSX ${jsxLines} 行（比例约束）`,
    )
  }
  if (!consumer) {
    violations.push(`[style-lines] ${rel}：未找到消费它的 .tsx 组件（同目录同名或 import 引用均无）`)
  }
}

if (violations.length === 0) {
  console.log(`[check:style-lines] PASS：${cssFiles.length} 个 .module.css，全部满足 412 行绝对上限与 JSX 比例约束`)
  process.exit(0)
}
for (const v of violations) console.error(v)
console.error(`[check:style-lines] FAIL：${violations.length} 项违规`)
process.exit(1)
