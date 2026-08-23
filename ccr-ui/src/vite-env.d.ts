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
