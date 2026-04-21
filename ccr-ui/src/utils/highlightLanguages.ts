/**
 * 集中注册 highlight.js 语言，避免在多个组件里重复 import 相同的 13 种语言模块
 *
 * 使用方式：
 *   import hljs from 'highlight.js/lib/core'
 *   import { registerDefaultLanguages } from '@/utils/highlightLanguages'
 *   registerDefaultLanguages(hljs)
 *
 * 幂等：同一 hljs 实例重复调用安全，内部通过 WeakSet 去重。
 */

import type { HLJSApi } from 'highlight.js'
import bash from 'highlight.js/lib/languages/bash'
import css from 'highlight.js/lib/languages/css'
import diff from 'highlight.js/lib/languages/diff'
import go from 'highlight.js/lib/languages/go'
import javascript from 'highlight.js/lib/languages/javascript'
import json from 'highlight.js/lib/languages/json'
import markdown from 'highlight.js/lib/languages/markdown'
import python from 'highlight.js/lib/languages/python'
import rust from 'highlight.js/lib/languages/rust'
import sql from 'highlight.js/lib/languages/sql'
import typescript from 'highlight.js/lib/languages/typescript'
import xml from 'highlight.js/lib/languages/xml'
import yaml from 'highlight.js/lib/languages/yaml'

// 已注册过的 hljs 实例，避免重复调用带来的内部表重建开销
const registered = new WeakSet<HLJSApi>()

/** 默认语言 + 常见别名对照 */
const ALIASES: Array<[string, unknown]> = [
  ['javascript', javascript],
  ['js', javascript],
  ['typescript', typescript],
  ['ts', typescript],
  ['python', python],
  ['py', python],
  ['bash', bash],
  ['sh', bash],
  ['shell', bash],
  ['json', json],
  ['yaml', yaml],
  ['yml', yaml],
  ['xml', xml],
  ['html', xml],
  ['css', css],
  ['rust', rust],
  ['rs', rust],
  ['go', go],
  ['golang', go],
  ['sql', sql],
  ['markdown', markdown],
  ['md', markdown],
  ['diff', diff],
]

/**
 * 在给定的 hljs 实例上注册默认语言。幂等。
 */
export function registerDefaultLanguages(hljs: HLJSApi): void {
  if (registered.has(hljs)) return
  for (const [name, lang] of ALIASES) {
    hljs.registerLanguage(name, lang as never)
  }
  registered.add(hljs)
}
