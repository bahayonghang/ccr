/**
 * useMarkdownRender —— 统一封装 marked + DOMPurify + highlight.js
 *
 * 抽走之前散落在各处 Markdown 区块里的
 * 重复 import（marked + 13 种 hljs 语言 + sanitize），保持行为一致并消除冗余打包。
 *
 * 导出：
 *   - renderMarkdown(raw, options?)：同步返回 sanitize 后的 HTML 字符串；内部做内容级缓存
 *   - highlightCodeBlocks(root)   ：对容器内所有 <pre><code> 执行语法高亮（幂等）
 *   - hljs                        ：共享 hljs 实例，供调用方细粒度操作
 */

import { marked, type MarkedOptions } from 'marked'
import hljs from 'highlight.js/lib/core'
import { registerDefaultLanguages } from '@/utils/highlightLanguages'
import { sanitizeMarkdown } from '@/utils/sanitize'

// 首次加载即注册默认语言（幂等）
registerDefaultLanguages(hljs)

// 渲染结果缓存，按 (raw + optionsKey) 命中，避免大文档重复解析
const RENDER_CACHE_CAPACITY = 64
const renderCache = new Map<string, string>()

const buildCacheKey = (raw: string, options?: MarkedOptions): string => {
  if (!options) return raw
  // options 可枚举字段较少，JSON.stringify 足够且可控
  return `${JSON.stringify(options)} ${raw}`
}

const rememberRender = (key: string, html: string): void => {
  if (renderCache.has(key)) {
    renderCache.delete(key)
  } else if (renderCache.size >= RENDER_CACHE_CAPACITY) {
    const oldest = renderCache.keys().next().value
    if (oldest !== undefined) renderCache.delete(oldest)
  }
  renderCache.set(key, html)
}

/**
 * 渲染 Markdown 为 sanitize 后的 HTML。
 * - 空输入返回空串
 * - 内部使用 marked.parse（同步）+ sanitizeMarkdown
 * - 命中缓存则直接返回
 */
export function renderMarkdown(raw: string, options?: MarkedOptions): string {
  if (!raw) return ''

  const key = buildCacheKey(raw, options)
  const cached = renderCache.get(key)
  if (cached !== undefined) {
    // LRU touch：删除再插入把键挪到最新位置
    renderCache.delete(key)
    renderCache.set(key, cached)
    return cached
  }

  const html = options
    ? (marked.parse(raw, options) as string)
    : (marked.parse(raw) as string)
  const safe = sanitizeMarkdown(html)
  rememberRender(key, safe)
  return safe
}

/**
 * 对容器内所有 <pre><code> 块执行 hljs.highlightElement。
 * 传入 null / undefined 时无操作。幂等（hljs 内部有防重复标记）。
 */
export function highlightCodeBlocks(root: HTMLElement | null | undefined): void {
  if (!root) return
  root.querySelectorAll<HTMLElement>('pre code').forEach((block) => {
    hljs.highlightElement(block)
  })
}

/** 允许外部在需要时拿到 hljs 实例（如直接 highlight 单块） */
export { hljs }
