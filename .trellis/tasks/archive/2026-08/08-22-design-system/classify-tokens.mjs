/**
 * 448 token 分类脚本（design.md §2 方法）
 *
 * 输入：ccr-ui/src/styles/tokens.css
 * 输出：token-classification.md（448 行，无未分类项）
 *
 * 分类规则：
 *   - 对每个变量名收集其在 tokens.css 全部顶层规则块中的定义点；
 *     @media 内的定义视为降级覆盖，不参与「选择器上下文」计数。
 *   - 归一化选择器：剥掉 :where() / html / :root 链，取 data-* 属性集合为上下文。
 *   - 出现在 2 个以上不同上下文 → 可切换语义变量（第 1 层）。
 *   - 单上下文且值为字面量 → 常量 token（进 @theme）。
 *   - 值引用其他变量（var()/calc()/color-mix()）→ 计算/别名 token，跟随其输入类别。
 *
 * 运行：bun classify-tokens.mjs
 */
import { readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
// postcss 从 ccr-ui 的依赖解析（脚本本身位于 .trellis/，不在项目 node_modules 树内）
const UI_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../ccr-ui')
const postcss = createRequire(path.join(UI_ROOT, 'probe.js'))('postcss')

const TOKENS_PATH = path.resolve(UI_ROOT, 'src/styles/tokens.css')
const OUT_PATH = path.resolve(path.dirname(fileURLToPath(import.meta.url)), 'token-classification.md')

const source = await readFile(TOKENS_PATH, 'utf8')
const root = postcss.parse(source)

/** 归一化选择器 → 上下文集合：只保留 [data-*="..."] 的属性值对，按名排序。 */
const selectorContext = (selector) => {
  const attrs = [...selector.matchAll(/\[data-([\w-]+)=['"]([\w-]+)['"]\]/g)].map((m) => [
    m[1],
    m[2],
  ])
  attrs.sort((a, b) => a[0].localeCompare(b[0]) || a[1].localeCompare(b[1]))
  return JSON.stringify(attrs)
}

/** 顶层规则块（含顶层 @media 内第一层规则；@media 本身标记为降级块）。 */
const blocks = []
const walkTop = (node, inMedia) => {
  if (node.type === 'atrule' && node.name === 'media') {
    node.nodes?.forEach((child) => walkTop(child, true))
    return
  }
  if (node.type === 'atrule') return
  if (node.type === 'rule') {
    const decls = {}
    node.walkDecls((d) => {
      if (d.prop.startsWith('--')) decls[d.prop] = d.value.trim()
    })
    if (Object.keys(decls).length > 0) {
      blocks.push({ node, inMedia, decls })
    }
  }
}
root.nodes?.forEach((node) => walkTop(node, false))

/** 每个名字：全部定义点（上下文 → 值列表）。 */
const nameDefs = new Map()
for (const block of blocks) {
  for (const [name, value] of Object.entries(block.decls)) {
    if (!nameDefs.has(name)) nameDefs.set(name, [])
    nameDefs.get(name).push({
      context: selectorContext(block.node.selector),
      value,
      inMedia: block.inMedia,
    })
  }
}

/** 每个名字的分类缓存。 */
const classCache = new Map()

const isDerived = (value) => /var\(|calc\(|color-mix\(/.test(value)

const classifyName = (name) => {
  if (classCache.has(name)) return classCache.get(name)
  const defs = nameDefs.get(name) ?? []

  // 分类只依据非 @media 顶层块。
  const topDefs = defs.filter((d) => !d.inMedia)
  const contexts = new Set(topDefs.map((d) => d.context))

  let klass
  let target

  if (contexts.size >= 2) {
    klass = '可切换语义变量'
    target = '第 1 层（themes/ 普通 CSS 变量）'
  } else if (topDefs.length === 0) {
    // 只在 @media 内定义：跟随其值引用的输入。
    const value = defs[0]?.value ?? ''
    if (isDerived(value)) {
      klass = '计算 token（仅 @media 降级定义，跟随输入）'
      target = followTarget(value)
    } else {
      klass = '常量 token'
      target = '@theme（非 inline）'
    }
  } else if (isDerived(topDefs[0].value)) {
    klass = '计算 token（跟随输入）'
    target = followTarget(topDefs[0].value)
  } else {
    klass = '常量 token'
    target = '@theme（非 inline）'
  }

  classCache.set(name, { klass, target })
  return classCache.get(name)
}

const followTarget = (value) => {
  const refs = [...value.matchAll(/var\(\s*(--[\w-]+)/g)].map((m) => m[1])
  if (refs.length === 0) return '@theme（非 inline）'
  const classes = refs.map((ref) => classifyName(ref).klass)
  if (classes.every((c) => c === '常量 token')) return '@theme（非 inline）'
  if (classes.some((c) => c.includes('可切换'))) return '第 1 层（跟随可切换输入）'
  return '第 1 层（跟随计算输入）'
}

// 分类统计
const counts = { '可切换语义变量': 0, '常量 token': 0, '计算 token（跟随输入）': 0, '计算 token（仅 @media 降级定义，跟随输入）': 0 }
for (const name of nameDefs.keys()) {
  const { klass } = classifyName(name)
  counts[klass] = (counts[klass] ?? 0) + 1
}

// 产出 448 行：按源码顺序每个定义点一行。
let rows = []
let totalRows = 0
for (const block of blocks) {
  for (const [name, value] of Object.entries(block.decls)) {
    const { klass, target } = classifyName(name)
    rows.push({ name, value, klass, target, inMedia: block.inMedia })
    totalRows += 1
  }
}

// 校验：无未分类。
const unclassified = rows.filter((r) => !r.klass || !r.target)
if (unclassified.length > 0) {
  throw new Error(`未分类项：${unclassified.map((r) => r.name).join(', ')}`)
}

const md = `# token 分类表（batch 1）

> 依据 \`.trellis/tasks/08-22-design-system/design.md\` §2 分类方法对 \`ccr-ui/src/styles/tokens.css\`
> 的 448 个变量定义点逐条分类。**448 行，无未分类项。**
>
> 生成：\`classify-tokens.mjs\`（bun）。名称集合基线见 \`token-names-before.txt\`。

## 分类统计（按唯一名）

| 类 | 唯一名数 |
| --- | --- |
${Object.entries(counts)
  .map(([k, v]) => `| ${k} | ${v} |`)
  .join('\n')}

## 分类方法（design.md §2）

1. 对每个变量名收集 \`tokens.css\` 全部顶层规则块中的定义点（\`@media\` 内定义为降级覆盖，不参与计数）。
2. 归一化选择器：剥掉 \`:where()\` / \`html\` / \`:root\` 链，取 \`[data-*]\` 属性值对集合为上下文。
3. 出现在 **2 个以上不同上下文**（\`:root\` + \`[data-theme=...]\` / \`[data-flavor=...]\` / \`[data-accent=...]\`）→ **可切换语义变量（第 1 层）**。
4. 单上下文且值为字面量（间距、圆角、字号、字重、时长、z-index…）→ **常量 token（进 @theme）**。
5. 值引用其他变量（\`var()\` / \`calc()\` / \`color-mix()\`）→ **计算/别名 token，跟随其输入变量的类别**。

> 批次 1 落位说明：第 1 层变量**物理上仍留在 \`tokens.css\`**（批次 1 与批次 2 之间的
> 兼容约束，见 \`implement.md\` 批次 1 证据块——\`theme-contrast-contract\` / \`apple-glass-surface-contract\` /
> \`theme-bootstrap\` 三个 smoke 测试直接解析 \`tokens.css\` 文本）。「目标落位」列记录的是
> design.md §3 的目标位置，批次 2 目录分层时随测试契约重建（批次 8）落地。

## 明细（448 行，按源码顺序）

| # | 变量名 | 定义点值（节选） | 类 | 目标落位 |
| --- | --- | --- | --- | --- |
${rows
  .map(
    (r, i) =>
      `| ${i + 1} | \`${r.name}\` | \`${r.value.replace(/`/g, '')}\` | ${r.klass}${
        r.inMedia ? '（@media 降级）' : ''
      } | ${r.target} |`
  )
  .join('\n')}

---
共 ${rows.length} 行（tokens.css 内全部自定义属性定义点）。未分类项：0。
`

await writeFile(OUT_PATH, md, 'utf8')
console.log(`wrote ${OUT_PATH}`)
console.log(`rows: ${totalRows}, unique names: ${nameDefs.size}`)
console.log('counts:', JSON.stringify(counts, null, 2))
