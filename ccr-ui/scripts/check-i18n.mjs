#!/usr/bin/env bun
/* eslint-disable no-console -- 诊断脚本必须用 console 输出，不接 logger */
/**
 * 比较两个 locale 的叶子 key 集合，并扫描调用点的缺失 / 未使用 key。
 *
 * 用法：bun run scripts/check-i18n.mjs
 */

import { readdirSync, readFileSync, statSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import en from '../src/i18n/locales/en-US.ts'
import zh from '../src/i18n/locales/zh-CN.ts'
import whitelist from './i18n-key-whitelist.json'
import { leafKeys } from './i18n-utils.mjs'

const EXPECTED_LEAF_COUNT = 4166
const here = dirname(fileURLToPath(import.meta.url))
const srcRoot = join(here, '../src')

const enKeySet = new Set(leafKeys(en))
const zhKeySet = new Set(leafKeys(zh))

const missingZh = [...enKeySet].filter((key) => !zhKeySet.has(key)).sort()
const missingEn = [...zhKeySet].filter((key) => !enKeySet.has(key)).sort()

let failed = false

if (enKeySet.size !== EXPECTED_LEAF_COUNT || zhKeySet.size !== EXPECTED_LEAF_COUNT) {
  console.error(
    `❌ 叶子 key 数量不是 ${EXPECTED_LEAF_COUNT}（en-US ${enKeySet.size}，zh-CN ${zhKeySet.size}）`,
  )
  failed = true
}

if (missingZh.length === 0 && missingEn.length === 0) {
  console.log(`✅ i18n key 集合一致（en-US ${enKeySet.size} 个 key，zh-CN ${zhKeySet.size} 个 key）`)
} else {
  failed = true
  if (missingZh.length > 0) {
    console.error(`❌ zh-CN.ts 缺失 ${missingZh.length} 个 key（en-US 已有）:`)
    for (const key of missingZh) console.error(`  - ${key}`)
  }
  if (missingEn.length > 0) {
    console.error(`❌ en-US.ts 缺失 ${missingEn.length} 个 key（zh-CN 已有）:`)
    for (const key of missingEn) console.error(`  - ${key}`)
  }
}

const sourceFiles = []
const walk = (dir) => {
  for (const name of readdirSync(dir)) {
    const full = join(dir, name)
    const stat = statSync(full)
    if (stat.isDirectory()) {
      if (name === 'locales') continue
      walk(full)
      continue
    }
    if (/\.(ts|tsx)$/.test(name)) sourceFiles.push(full)
  }
}
walk(srcRoot)

const CALL_RE = /(?<![A-Za-z0-9_])(?:i18n\.)?t\(\s*(['"])([^'"]+)\1/g
const TF_RE = /(?<![A-Za-z0-9_])tf\(\s*(['"])([^'"]+)\1/g
const QUOTED_KEY_RE = /['"]([A-Za-z][A-Za-z0-9_]*(?:\.[A-Za-z0-9_]+)+)['"]/g
const PREFIX_TEMPLATE_RE = /`([A-Za-z][A-Za-z0-9_.]*)\$\{[^}]+\}([A-Za-z0-9_.]*)`/g
const SUFFIX_TEMPLATE_RE = /`\$\{[^}]+\}((?:\.[A-Za-z0-9_]+)+)`/g

const usedExact = new Set()
const usedPrefixes = new Set()
const usedSuffixes = new Set()
const callKeys = new Set()

for (const file of sourceFiles) {
  const content = readFileSync(file, 'utf8')
  let match
  const call = new RegExp(CALL_RE.source, 'g')
  while ((match = call.exec(content))) {
    usedExact.add(match[2])
    callKeys.add(match[2])
  }
  const tf = new RegExp(TF_RE.source, 'g')
  while ((match = tf.exec(content))) {
    usedExact.add(match[2])
  }
  const quoted = new RegExp(QUOTED_KEY_RE.source, 'g')
  while ((match = quoted.exec(content))) {
    usedExact.add(match[1])
  }
  const prefixTpl = new RegExp(PREFIX_TEMPLATE_RE.source, 'g')
  while ((match = prefixTpl.exec(content))) {
    if (match[1]) usedPrefixes.add(match[1])
    if (match[2]) usedSuffixes.add(match[2].startsWith('.') ? match[2] : `.${match[2]}`)
  }
  const suffixTpl = new RegExp(SUFFIX_TEMPLATE_RE.source, 'g')
  while ((match = suffixTpl.exec(content))) {
    usedSuffixes.add(match[1])
  }
}

const whitelistMissing = new Set(
  (whitelist.missing ?? []).map((entry) => entry.key).filter((key) => !key.endsWith('.*')),
)
const whitelistMissingPrefixes = (whitelist.missing ?? [])
  .map((entry) => entry.key)
  .filter((key) => key.endsWith('.*'))
  .map((key) => key.slice(0, -1))
const whitelistUnused = new Set((whitelist.unused ?? []).map((entry) => entry.key))

const missingCalls = [...callKeys]
  .filter((key) => !zhKeySet.has(key))
  .filter((key) => !whitelistMissing.has(key))
  .filter((key) => !whitelistMissingPrefixes.some((prefix) => key.startsWith(prefix)))
  .sort()

const isUsed = (key) => {
  if (usedExact.has(key) || whitelistUnused.has(key)) return true
  for (const prefix of usedPrefixes) {
    if (key.startsWith(prefix)) return true
  }
  for (const suffix of usedSuffixes) {
    if (key.endsWith(suffix)) return true
  }
  return false
}

const unusedKeys = [...zhKeySet].filter((key) => !isUsed(key)).sort()

if (missingCalls.length === 0) {
  console.log(`✅ 字面量 t() 调用的 key 均存在于词条（扫描 ${callKeys.size} 个）`)
} else {
  console.log(`⚠ 字面量 t() 调用了词条中没有的 key ${missingCalls.length} 个（多为 tf 兜底或历史 key；不阻断）:`)
  for (const key of missingCalls.slice(0, 20)) console.log(`  - ${key}`)
  if (missingCalls.length > 20) console.log(`  ... 另有 ${missingCalls.length - 20} 个`)
}

if (unusedKeys.length === 0) {
  console.log('✅ 词条中无未使用 key')
} else {
  console.log(`⚠ 未使用 key ${unusedKeys.length} 个（动态拼接 / 配置表可能未静态抽出）`)
  for (const key of unusedKeys.slice(0, 20)) console.log(`  - ${key}`)
  if (unusedKeys.length > 20) console.log(`  ... 另有 ${unusedKeys.length - 20} 个`)
}

if (failed) {
  console.error('')
  console.error('提示：单边新增 i18n key 时，另一端运行时会回退展示 key 路径而非文案。请补齐两端再合并。')
  process.exit(1)
}

process.exit(0)
