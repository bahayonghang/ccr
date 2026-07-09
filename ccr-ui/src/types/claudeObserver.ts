/**
 * Claude Observer 类型 re-export shim
 *
 * 类型真身是 ts-rs 生成绑定（src/types/generated/claude_observer/，由
 * `just tauri-bindings` 从 Rust DTO 生成并入库）。本文件只保留旧导入路径
 * `@/types/claudeObserver` 的兼容名，禁止再新增手写 Rust 镜像接口。
 */

export type { BreakdownRow } from '@/types/generated/claude_observer/BreakdownRow'
export type { CacheStatsDto } from '@/types/generated/claude_observer/CacheStatsDto'
export type { DailyPoint } from '@/types/generated/claude_observer/DailyPoint'
export type { HeatmapCell } from '@/types/generated/claude_observer/HeatmapCell'
export type { InsightDto } from '@/types/generated/claude_observer/InsightDto'
export type { SessionRow } from '@/types/generated/claude_observer/SessionRow'
export type { SubscriptionDto } from '@/types/generated/claude_observer/SubscriptionDto'
export type { TopToolRow } from '@/types/generated/claude_observer/TopToolRow'
