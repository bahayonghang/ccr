import { fileURLToPath, URL } from 'node:url'
import path from 'node:path'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import devWarmTargets from './scripts/dev-warm-targets.json'

const dirname =
  typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url))

// https://vite.dev/config/
// React 基座配置：映射见 .trellis/tasks/08-22-react-foundation/design.md §3。
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  build: {
    outDir: 'dist',
    reportCompressedSize: false,
    rollupOptions: {
      output: {
        // vite 8 底层为 rolldown，不再支持对象形式的 manualChunks，改为函数形式；
        // 分组与 design.md §3 的清单一一对应
        manualChunks: (id) => {
          const groups: Array<[string, RegExp]> = [
            ['react-vendor', /[\\/]node_modules[\\/](react|react-dom|react-router)[\\/]/],
            // query-vendor：react-query 在 main.tsx/shell/queryClient.ts 被真实导入，
            // 08-22-arch-quality-perf 批次 8 测量后加入。必须同时匹配 @tanstack/query-core：
            // react-query 的查询引擎在独立的 query-core 包中，仅匹配 react-query 会把它留在 index。
            // 加入后 index 从 167.15 kB 降至 142.80 kB，query-vendor 32.26 kB（原始 rolldown 实测）。
            // form-vendor / motion-vendor 未加入：react-hook-form 与 motion 当前无导入点，
            // 空分组不产出 chunk，待 state-logic-port / design-system 实际导入时再补。
            ['query-vendor', /[\\/]node_modules[\\/]@tanstack[\\/](react-query|query-core)[\\/]/],
            ['motion-vendor', /[\\/]node_modules[\\/]motion[\\/]/],
            ['ui-vendor', /[\\/]node_modules[\\/]@iconify[\\/]react[\\/]/],
            // 图表库统一走 src/utils/apexChartsCore.ts 的按需入口，并全程 await import()；
            // 这里固定成单一 charts-vendor chunk，避免多个懒加载点各自复制一份 core。
            ['charts-vendor', /[\\/]node_modules[\\/](apexcharts|react-apexcharts)[\\/]/],
            ['i18n-vendor', /[\\/]node_modules[\\/](i18next|react-i18next)[\\/]/],
            ['markdown-vendor', /[\\/]node_modules[\\/]dompurify[\\/]/],
            ['search-vendor', /[\\/]node_modules[\\/]fuse\.js[\\/]/],
            ['tauri-vendor', /[\\/]node_modules[\\/]@tauri-apps[\\/]api[\\/]/],
            ['virtual-vendor', /[\\/]node_modules[\\/]@tanstack[\\/]react-virtual[\\/]/],
            ['term-vendor', /[\\/]node_modules[\\/]ansi_up[\\/]/],
          ]
          for (const [name, pattern] of groups) {
            if (pattern.test(id)) return name
          }
          return undefined
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
      'react',
      'react-dom',
      'react-dom/client',
      'react-router',
      '@tanstack/react-query',
      '@iconify/react',
      // 重量级依赖：避免 noDiscovery 下运行时逐个转译，大幅缩短 dev 首屏加载
      'apexcharts',
      'react-apexcharts',
      'dompurify',
      'fuse.js',
      'ansi_up',
      '@tauri-apps/api',
      '@tauri-apps/api/core',
      '@tauri-apps/api/app',
      '@tauri-apps/api/event',
      '@tauri-apps/api/window',
      '@tanstack/react-virtual',
    ],
  },
})
