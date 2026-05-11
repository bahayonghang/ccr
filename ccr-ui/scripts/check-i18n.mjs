#!/usr/bin/env bun
/* eslint-disable no-console -- 诊断脚本必须用 console 输出，不接 logger */
/**
 * scripts/check-i18n.mjs
 *
 * 比较 src/i18n/locales/{en-US,zh-CN}.ts 的 leaf key 集合，单边新增 key 时报错并 exit 1。
 *
 * 触发场景：开发者在一个 locale 加了新 key 忘记同步另一个 locale，
 * vue-i18n 在运行时会回退展示 key 路径而不是真实文案，UI 上很难第一时间发现。
 * 本脚本接到 just frontend-check 链中，CI 能在合并前拦截。
 *
 * 用法：bun run scripts/check-i18n.mjs
 */

import en from '../src/i18n/locales/en-US.ts'
import zh from '../src/i18n/locales/zh-CN.ts'

/**
 * 递归收集 leaf key 路径，跳过 array 与函数。
 * 只把"实际承载文案的叶子节点"算 key。中间命名空间不算。
 */
function* leafKeys(obj, prefix = '') {
  for (const [k, v] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${k}` : k
    if (v && typeof v === 'object' && !Array.isArray(v)) {
      yield* leafKeys(v, path)
    } else {
      yield path
    }
  }
}

const enKeys = new Set(leafKeys(en))
const zhKeys = new Set(leafKeys(zh))

const missingZh = [...enKeys].filter((k) => !zhKeys.has(k)).sort()
const missingEn = [...zhKeys].filter((k) => !enKeys.has(k)).sort()

if (missingZh.length === 0 && missingEn.length === 0) {
  console.log(`✅ i18n key 集合一致（en-US ${enKeys.size} 个 key，zh-CN ${zhKeys.size} 个 key）`)
  process.exit(0)
}

if (missingZh.length > 0) {
  console.error(`❌ zh-CN.ts 缺失 ${missingZh.length} 个 key（en-US 已有）:`)
  for (const key of missingZh) console.error(`  - ${key}`)
}
if (missingEn.length > 0) {
  console.error(`❌ en-US.ts 缺失 ${missingEn.length} 个 key（zh-CN 已有）:`)
  for (const key of missingEn) console.error(`  - ${key}`)
}
console.error('')
console.error(
  '提示：单边新增 i18n key 时，另一端运行时会回退展示 key 路径而非文案。请补齐两端再合并。',
)
process.exit(1)
