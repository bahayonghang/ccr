/**
 * claude-observer 各 Tab 共享的展示格式化函数。
 *
 * 仅收口在多个 Tab 间逐字重复的实现；带不同阈值/精度的局部变体
 * （如 UsageInsightPanel 使用更高阈值的 formatUsd）仍保留在各自组件内，
 * 避免改变既有展示口径。
 */

/** 紧凑 token 计数：B / M / k 三档，保留既有精度与小写 k。 */
export function formatTokens(value: number): string {
  if (!Number.isFinite(value) || value <= 0) return '0'
  if (value >= 1e9) return `${(value / 1e9).toFixed(2)}B`
  if (value >= 1e6) return `${(value / 1e6).toFixed(2)}M`
  if (value >= 1e3) return `${(value / 1e3).toFixed(1)}k`
  return value.toLocaleString()
}

/** 美元金额：≥100 取整、≥1 两位小数、否则四位小数。 */
export function formatUsd(value: number): string {
  if (!Number.isFinite(value)) return '$0.00'
  if (value >= 100) return `$${value.toFixed(0)}`
  if (value >= 1) return `$${value.toFixed(2)}`
  return `$${value.toFixed(4)}`
}
