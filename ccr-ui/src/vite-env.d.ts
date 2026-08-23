/// <reference types="vite/client" />

// Tauri v2 运行时注入的全局变量
interface Window {
    __TAURI__?: Record<string, unknown>
    __TAURI_INTERNALS__?: Record<string, unknown>
}

interface ImportMetaEnv {
    readonly VITE_USAGE_DASHBOARD_AGGREGATED_API?: string
    readonly VITE_USAGE_LOGS_CURSOR_PAGING?: string
    readonly VITE_PERF_HEATMAP_LAZY_LOAD?: string
    // more env variables...
}

interface ImportMeta {
    readonly env: ImportMetaEnv
}

// 迁移过渡期垫片：未迁移的 src/**/*.ts 仍导入 .vue 单文件组件，
// 该声明使其在纯 tsc（无 vue-tsc）下继续通过类型检查。
declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>
  export default component
  // 过渡期具名类型导出：StatTile/PillToggle 仍被未迁移的 ui barrel 引用。
  export type StatTileTone = 'neutral' | 'success' | 'warning' | 'danger' | 'accent'
  export type PillToggleOption<TValue extends string | number = string> = {
    value: TValue
    label: string
    disabled?: boolean
  }
 }
