/// <reference types="vite/client" />

interface ImportMetaEnv {
    readonly VITE_API_BASE_URL: string
    readonly VITE_USAGE_DASHBOARD_AGGREGATED_API?: string
    readonly VITE_USAGE_LOGS_CURSOR_PAGING?: string
    // more env variables...
}

interface ImportMeta {
    readonly env: ImportMetaEnv
}
