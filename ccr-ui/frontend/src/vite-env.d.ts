/// <reference types="vite/client" />

// Tauri v2 运行时注入的全局变量
interface Window {
    __TAURI_INTERNALS__?: Record<string, unknown>
}

interface ImportMetaEnv {
    readonly VITE_API_BASE_URL: string
    readonly VITE_USAGE_DASHBOARD_AGGREGATED_API?: string
    readonly VITE_USAGE_LOGS_CURSOR_PAGING?: string
    // more env variables...
}

interface ImportMeta {
    readonly env: ImportMetaEnv
}
