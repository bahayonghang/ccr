// 文本展示辅助：中段省略，比 CSS 端部省略更能保留 URL 等字符串两端的关键信息。
export function truncateMiddle(value: string, head = 20, tail = 12): string {
  if (value.length <= head + tail + 1) return value
  return `${value.slice(0, head)}…${value.slice(value.length - tail)}`
}

/**
 * 密排场景的 base_url 展示：完整保留 host，只在路径过长时截断路径。
 * 非 URL 文本（例如官方直连文案）原样返回。
 */
export function formatBaseUrlDisplay(raw: string, maxPathLength = 18): string {
  try {
    const url = new URL(raw)
    const path = url.pathname === '/' ? '' : url.pathname
    const shownPath = path.length > maxPathLength ? `${path.slice(0, maxPathLength - 1)}…` : path
    return `${url.host}${shownPath}`
  } catch {
    return raw
  }
}
