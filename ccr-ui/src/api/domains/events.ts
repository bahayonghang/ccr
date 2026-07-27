/**
 * Events Domain —— 事件总线与运行时指标 API
 *
 * 对应后端 commands::events::* 命令。
 * 真迁移自 tauri.ts 第 18 分组。
 */

export { getRecentEvents, getRuntimeMetrics } from '../generated/events'
