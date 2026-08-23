import type { UnifiedMcpServer } from '@/types/unifiedMcp'
import type { TranslateFunction } from '@/utils/tf'

const TRANSPORT_LABEL_MAX = 40

export type McpStateTone = 'ok' | 'warning' | 'danger' | 'muted'

/** 密钥预览：已掩码的原样返回；短值全掩；长值保留头尾。 */
export function maskSecret(value: string): string {
  if (!value) return ''
  if (value.includes('•')) return value
  if (value.length <= 8) return '••••••'
  return `${value.slice(0, 4)}••••${value.slice(-2)}`
}

export function shortenTransportLabel(label: string): string {
  if (label.length <= TRANSPORT_LABEL_MAX) return label
  return `${label.slice(0, TRANSPORT_LABEL_MAX).trimEnd()}...`
}

export function mcpScopeLabel(scope: string, t: TranslateFunction): string {
  const key = `mcp.manager.scopes.${scope}`
  const label = t(key)
  return label === key ? scope : label
}

export function mcpApprovalLabel(state: string, t: TranslateFunction): string {
  const key = `mcp.manager.approvals.${state}`
  const label = t(key)
  return label === key ? state : label
}

export function formatScopeList(scopes: string[], t: TranslateFunction): string {
  return scopes.map((scope) => mcpScopeLabel(scope, t)).join(' / ')
}

export function mcpStateLabel(server: UnifiedMcpServer, t: TranslateFunction): string {
  if (server.hidden_by) return t('mcp.manager.state.overridden')
  if (server.disabled) return t('mcp.manager.state.disabled')
  if (server.approval_state === 'pending') return t('mcp.manager.state.pending')
  if (server.approval_state === 'disabled') return t('mcp.manager.state.notApproved')
  if (server.effective === false) return t('mcp.manager.state.hidden')
  return t('mcp.manager.state.effective')
}

export function mcpStateTone(server: UnifiedMcpServer): McpStateTone {
  if (server.hidden_by || server.effective === false) return 'muted'
  if (server.disabled || server.approval_state === 'disabled') return 'danger'
  if (server.approval_state === 'pending') return 'warning'
  return 'ok'
}

export const MCP_STATE_TONE_CLASS: Record<McpStateTone, string> = {
  ok: 'border-success/24 bg-success/10 text-success',
  warning: 'border-warning/26 bg-warning/10 text-warning',
  danger: 'border-danger/24 bg-danger/10 text-danger',
  muted: 'border-border-default/34 bg-bg-elevated/44 text-text-muted',
}
