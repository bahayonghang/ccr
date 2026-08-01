/// <reference types="vitest/config" />
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'
import path from 'node:path'
import devWarmTargets from './scripts/dev-warm-targets.json'

const dirname =
  typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url))

// https://vitejs.dev/config/
export default defineConfig(({ command }) => {
  const useRuntimeOnlyI18n = command === 'build'

  return {
    plugins: [vue()],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
        // dev 需要 message compiler，否则 locale 字符串会直接回退成 key；
        // build 继续使用 runtime-only，避免桌面壳 CSP 与 runtime compiler 冲突。
        'vue-i18n': useRuntimeOnlyI18n
          ? 'vue-i18n/dist/vue-i18n.runtime.esm-bundler.js'
          : 'vue-i18n/dist/vue-i18n.esm-bundler.js',
      },
    },
    build: {
      outDir: 'dist',
      reportCompressedSize: false,
      rollupOptions: {
        output: {
          manualChunks: {
            'vue-vendor': ['vue', 'vue-router', 'pinia'],
            'ui-vendor': ['@iconify/vue'],
            // 图表库统一走 src/utils/apexChartsCore.ts 的按需入口，并全程 await import()；
            // 这里固定成单一 charts-vendor chunk，避免多个懒加载点各自复制一份 core。
            'charts-vendor': ['apexcharts/core', 'vue3-apexcharts/core'],
            'i18n-vendor': ['vue-i18n'],
            'markdown-vendor': ['dompurify'],
            'search-vendor': ['fuse.js'],
            'tauri-vendor': ['@tauri-apps/api'],
            'virtual-vendor': ['@tanstack/vue-virtual'],
            'term-vendor': ['ansi_up'],
          },
        },
      },
      chunkSizeWarningLimit: 500,
    },
    server: {
      host: '127.0.0.1',
      port: 15173,
      strictPort: true,
      watch: {
        ignored: ['**/src-tauri/target/**', '**/ref/**', '**/logs/**'],
      },
      fs: {
        // providers-catalog.json 位于仓库根 crates/ 下（前后端共享单一数据源），
        // dev server 默认只放行 ccr-ui 根目录，这里显式放行 catalog 数据目录
        allow: [dirname, path.resolve(dirname, '../crates/ccr-checkin/data')],
      },
      warmup: {
        clientFiles: devWarmTargets.clientFiles,
      },
      hmr: {
        overlay: true,
      },
    },
    optimizeDeps: {
      noDiscovery: true,
      include: [
        'vue',
        'vue-router',
        'pinia',
        '@iconify/vue',
        'vue-i18n',
        // 重量级依赖：避免 noDiscovery 下运行时逐个转译，大幅缩短 dev 首屏加载
        'apexcharts',
        'vue3-apexcharts',
        'dompurify',
        'ansi_up',
        '@tauri-apps/api',
        '@tauri-apps/api/core',
        '@tauri-apps/api/app',
        '@tauri-apps/api/event',
        '@tauri-apps/api/window',
        '@tanstack/vue-virtual',
      ],
    },
  }
})
