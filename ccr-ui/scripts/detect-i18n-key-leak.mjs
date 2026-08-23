#!/usr/bin/env bun
/* eslint-disable no-console -- 诊断脚本必须用 console 输出，不接 logger */
/**
 * key 原文泄漏检测。由 zh-CN.ts 叶子 key 集合判定，不用手写正则做最终命中。
 *
 * 用法：
 *   bun ./scripts/detect-i18n-key-leak.mjs --self-test
 *   bun ./scripts/detect-i18n-key-leak.mjs --text "common.save 保存 package.json"
 */

import zh from '../src/i18n/locales/zh-CN.ts'
import { findLeakedKeys, leafKeys } from './i18n-utils.mjs'

const keySet = new Set(leafKeys(zh))

function selfTest() {
  const cases = [
    { text: 'checkin.stats.total_accounts', expect: ['checkin.stats.total_accounts'], label: 'underscore key' },
    { text: 'common.save', expect: ['common.save'], label: 'two-segment key' },
    { text: '保存', expect: [], label: 'translated copy' },
    { text: 'package.json example.com', expect: [], label: 'key-shaped non-catalog text' },
  ]
  let failed = 0
  for (const item of cases) {
    const hits = findLeakedKeys(item.text, keySet)
    const ok =
      hits.length === item.expect.length && item.expect.every((key) => hits.includes(key))
    if (ok) {
      console.log(`✅ ${item.label}: ${JSON.stringify(item.text)} → ${JSON.stringify(hits)}`)
    } else {
      failed += 1
      console.error(`❌ ${item.label}: expected ${JSON.stringify(item.expect)}, got ${JSON.stringify(hits)}`)
    }
  }
  if (!keySet.has('checkin.stats.total_accounts') || !keySet.has('common.save')) {
    failed += 1
    console.error('❌ leaf key set missing fixture keys')
  }
  if (keySet.has('package.json') || keySet.has('example.com')) {
    failed += 1
    console.error('❌ leaf key set should not contain package.json / example.com')
  }
  if (failed > 0) {
    process.exit(1)
  }
  console.log(`✅ key leak detector self-test passed（集合 ${keySet.size} 个叶子 key）`)
}

const args = process.argv.slice(2)
if (args.includes('--self-test') || args.length === 0) {
  selfTest()
}

const textIndex = args.indexOf('--text')
if (textIndex >= 0) {
  const text = args[textIndex + 1] ?? ''
  const hits = findLeakedKeys(text, keySet)
  if (hits.length === 0) {
    console.log('✅ no catalog keys in text')
  } else {
    console.error(`❌ leaked keys: ${hits.join(', ')}`)
    process.exit(1)
  }
}
