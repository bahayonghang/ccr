/**
 * 跨视图共享的运行时常量。
 *
 * 仅收口真正在多个文件间重复的字面量；模块内一次性使用的常量仍保留在各自文件中。
 */

/** 数据刷新去抖 TTL：距上次加载不足此值则跳过强制刷新（Codex Profiles/MCP/Auth 共用）。 */
export const REFRESH_TTL_MS = 30_000

/** 模态打开后聚焦首个可交互元素的延迟，给入场过渡留出渲染时间。 */
export const MODAL_FOCUS_DELAY_MS = 100
