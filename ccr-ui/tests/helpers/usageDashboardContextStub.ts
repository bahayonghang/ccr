/**
 * Shared usage dashboard context stub for smoke tests.
 * usage 域 tab 组件改为 inject useUsageDashboardContext() 后不再接受 props；
 * 这里用一层 provide 包装模拟 UsageDashboardView 的 provide 作用域，测试只需给目标
 * 组件实际读取的字段，无需构建完整的 useUsageDashboardState() 返回值。
 */
import { defineComponent, h, provide, type Component } from 'vue'
import {
  usageDashboardContextKey,
  type UsageDashboardContext,
} from '@/views/usage/usageDashboardContext'

/**
 * context 故意放宽成 Record<string, unknown>：测试只搭建目标组件实际读取的字段，
 * 图表 options 等精确类型（来自 chartOptions.ts 工厂）在测试桩里没必要严格匹配。
 */
export const withUsageDashboardContext = (
  target: Component,
  context: Record<string, unknown>
) =>
  defineComponent({
    name: 'UsageDashboardContextHarness',
    setup() {
      provide(usageDashboardContextKey, context as UsageDashboardContext)
      return () => h(target)
    },
  })
