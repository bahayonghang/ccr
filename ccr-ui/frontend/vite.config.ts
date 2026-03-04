/// <reference types="vitest/config" />
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { fileURLToPath, URL } from 'node:url';

// 后端端口配置（与 justfile 保持一致）
import path from 'node:path';
import { storybookTest } from '@storybook/addon-vitest/vitest-plugin';
import { playwright } from '@vitest/browser-playwright';
const dirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url));

// More info at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon
const backendPort = process.env.BACKEND_PORT || '48081';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  build: {
    outDir: 'dist',
    // 启用打包分析报告
    reportCompressedSize: true,
    // 代码分割优化
    rollupOptions: {
      output: {
        manualChunks: {
          // 将 Vue 生态单独打包
          'vue-vendor': ['vue', 'vue-router', 'pinia'],
          // 将 UI 库单独打包
          'ui-vendor': ['lucide-vue-next'],
          // 将 i18n 单独打包
          'i18n-vendor': ['vue-i18n']
        }
      }
    },
    // 分块大小警告阈值
    chunkSizeWarningLimit: 1000
  },
  // 开发服务器优化
  server: {
    port: 15173,
    warmup: {
      // 预热关键模块：减少首屏路由与布局首次访问的按需变换开销
      // 当前 API 已统一收敛到 tauri.ts + index.ts（不再使用旧 modules 分层文件）
      clientFiles: [
        './src/main.ts',
        './src/App.vue',
        './src/components/MainLayout.vue',
        './src/views/HomeView.vue',
        './src/api/index.ts',
        './src/api/tauri.ts',
        './src/router/index.ts',
      ],
    },
    hmr: {
      overlay: true
    },
    proxy: {
      '/api': {
        target: `http://127.0.0.1:${backendPort}`,
        changeOrigin: true
      }
    }
  },
  // 依赖优化：预打包高频重量级包，避免浏览器请求时串行变换
  optimizeDeps: {
    // noDiscovery: true → 禁用源码扫描（Vite 5.1+）
    // 效果：跳过扫描 200+ 源文件的过程（节省 3~5 秒），
    //       仅预打包下方 include 列表中的 npm 包
    noDiscovery: true,
    include: [
      'vue',
      'vue-router',
      'pinia',
      'lucide-vue-next', // MainLayout 中使用 13+ 图标，需预打包
      'vue-i18n',        // 多处导入的国际化库
      'marked',          // CJS→ESM 互操作需预打包
      // highlight.js 为 CJS 包，必须预打包以避免 ESM 链接错误
      'highlight.js/lib/core',
      'highlight.js/lib/languages/javascript',
      'highlight.js/lib/languages/typescript',
      'highlight.js/lib/languages/python',
      'highlight.js/lib/languages/bash',
      'highlight.js/lib/languages/json',
      'highlight.js/lib/languages/yaml',
      'highlight.js/lib/languages/xml',
      'highlight.js/lib/languages/css',
      'highlight.js/lib/languages/rust',
      'highlight.js/lib/languages/go',
      'highlight.js/lib/languages/sql',
      'highlight.js/lib/languages/markdown',
      'highlight.js/lib/languages/diff',
    ]
  },
  test: {
    projects: [{
      extends: true,
      plugins: [
        // The plugin will run tests for the stories defined in your Storybook config
        // See options at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon#storybooktest
        storybookTest({
          configDir: path.join(dirname, '.storybook')
        })],
      test: {
        name: 'storybook',
        browser: {
          enabled: true,
          headless: true,
          provider: playwright({}),
          instances: [{
            browser: 'chromium'
          }]
        },
        setupFiles: ['.storybook/vitest.setup.ts']
      }
    }]
  }
});