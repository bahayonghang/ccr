/**
 * MCP Manager 页面类型定义
 *
 * 用于 master-detail MCP 管理页面的 UI 状态类型。
 */

import type { UnifiedMcpServer } from './unifiedMcp'

/** 按名称聚合的 MCP 服务器组 — 同名服务器跨平台聚合 */
export interface McpGroup {
  /** 组标识 (取自服务器名称) */
  name: string
  /** 传输类型: 'stdio' | 'http' */
  transportType: 'stdio' | 'http'
  /** 主要显示信息: command (stdio) 或 url (http) */
  transportLabel: string
  /** 此名称下所有平台的服务器实例 */
  items: UnifiedMcpServer[]
  /** 关联的平台 ID 列表 */
  platforms: string[]
}

/** 右侧面板状态 */
export type McpPanelMode =
  | { type: 'empty' }
  | { type: 'detail'; groupName: string }
  | { type: 'create' }
  | { type: 'edit'; groupName: string }
  | { type: 'import' }
