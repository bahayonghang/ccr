import { inject, markRaw, provide, reactive, toRef, type Component, type InjectionKey } from 'vue'
import type { useUsageDashboardState } from './useUsageDashboardState'

type UsageDashboardState = ReturnType<typeof useUsageDashboardState>

/**
 * usage 域内的 provide/inject 上下文工厂。
 *
 * 由 UsageDashboardView 构建一次后 provide，7 个 tab 子组件 inject 取用，替代逐 tab
 * 8~20 个显式 props 透传（design.md §1）。上下文仅在 usage 域内使用，不上升为全局模式。
 *
 * - `...state`：组合根返回的 ref/computed，经 reactive 解包后 tab 直接读值。
 * - store 派生字段（summary/trends/…）用 tab 侧惯用名扁平化，保持子组件与旧 props 命名一致。
 * - `chartComponent` 用 markRaw 包裹，避免异步组件被 reactive 代理。
 */
export const createUsageDashboardContext = (
  state: UsageDashboardState,
  extras: {
    chartComponent: Component
    updateSelectedPlatform: (platform: string) => void
  }
) => {
  const { store } = state

  return reactive({
    ...state,
    chartComponent: markRaw(extras.chartComponent),
    updateSelectedPlatform: extras.updateSelectedPlatform,
    summary: toRef(store, 'summary'),
    trends: toRef(store, 'trends'),
    modelStats: toRef(store, 'modelStats'),
    projectStats: toRef(store, 'projectStats'),
    logsLoading: toRef(store, 'logsLoading'),
    logsPage: toRef(store, 'logsPage'),
    logsTotalPages: toRef(store, 'logsTotalPages'),
    canPrevLogs: toRef(store, 'canPrevLogs'),
    canNextLogs: toRef(store, 'canNextLogs'),
    hasLogsTotal: toRef(store, 'hasLogsTotal'),
    showLogsPager: toRef(store, 'showLogsPager'),
  })
}

export type UsageDashboardContext = ReturnType<typeof createUsageDashboardContext>

export const usageDashboardContextKey: InjectionKey<UsageDashboardContext> =
  Symbol('usage-dashboard-context')

export const provideUsageDashboardContext = (context: UsageDashboardContext) => {
  provide(usageDashboardContextKey, context)
}

export const useUsageDashboardContext = (): UsageDashboardContext => {
  const context = inject(usageDashboardContextKey)
  if (!context) {
    throw new Error('useUsageDashboardContext 必须在 UsageDashboardView 的 provide 作用域内调用')
  }
  return context
}
