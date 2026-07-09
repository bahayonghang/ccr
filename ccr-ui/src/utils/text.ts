// 文本展示辅助：中段省略，比 CSS 端部省略更能保留 URL 等字符串两端的关键信息。
export function truncateMiddle(value: string, head = 20, tail = 12): string {
  if (value.length <= head + tail + 1) return value
  return `${value.slice(0, head)}…${value.slice(value.length - tail)}`
}
