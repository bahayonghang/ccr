/// <reference types="vitest/config" />
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { fileURLToPath, URL } from 'node:url';
import path from 'node:path';
import { storybookTest } from '@storybook/addon-vitest/vitest-plugin';
import { playwright } from '@vitest/browser-playwright';

const dirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url));

// https://vitejs.dev/config/
export default defineConfig(({ command }) => {
  const useRuntimeOnlyI18n = command === 'build';

  return {
    plugins: [vue()],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
        // dev 需要 message compiler，否则 locale 字符串会直接回退成 key；
        // build 继续使用 runtime-only，避免桌面壳 CSP 与 runtime compiler 冲突。
        'vue-i18n': useRuntimeOnlyI18n
          ? 'vue-i18n/dist/vue-i18n.runtime.esm-bundler.js'
          : 'vue-i18n/dist/vue-i18n.esm-bundler.js'
      }
    },
    build: {
      outDir: 'dist',
      reportCompressedSize: false,
      rollupOptions: {
        output: {
          manualChunks: {
            'vue-vendor': ['vue', 'vue-router', 'pinia'],
            'ui-vendor': ['@iconify/vue'],
            'charts-vendor': ['apexcharts', 'vue3-apexcharts'],
            'i18n-vendor': ['vue-i18n'],
            'markdown-vendor': ['marked', 'dompurify', 'highlight.js'],
            'search-vendor': ['fuse.js'],
            'tauri-vendor': ['@tauri-apps/api'],
            'virtual-vendor': ['@tanstack/vue-virtual'],
            'term-vendor': ['ansi_up']
          }
        }
      },
      chunkSizeWarningLimit: 500
    },
    server: {
      host: '127.0.0.1',
      port: 15173,
      strictPort: true,
      warmup: {
        clientFiles: [
          './src/main.ts',
          './src/App.vue',
          './src/components/MainLayout.vue',
          './src/views/DashboardView.vue',
          './src/views/dashboard/dashboardPresentation.ts',
          './src/api/index.ts',
          './src/api/tauri.ts',
          './src/router/index.ts',
          './src/stores/usage.ts',
          './src/stores/usageDashboardPayload.ts',
          './src/stores/usageImportNormalization.ts',
          './src/views/UsageDashboardView.vue',
          './src/views/usage/useUsageDashboardState.ts',
          './src/views/usage/usageChartOptions.ts',
          './src/views/usage/usageDiagnostics.ts',
          './src/views/usage/usageOverviewInsights.ts',
          './src/views/usage/usageSummaryCards.ts',
          './src/views/CodexAuthView.vue',
          './src/views/codex/codexAuthAccounts.ts',
          './src/components/usage/UsageOverviewTab.vue',
          './src/components/usage/UsageMetricCard.vue',
        ],
      },
      hmr: {
        overlay: true
      }
    },
    optimizeDeps: {
      noDiscovery: true,
      include: [
        'vue',
        'vue-router',
        'pinia',
        '@iconify/vue',
        'vue-i18n',
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
        // 重量级依赖：避免 noDiscovery 下运行时逐个转译，大幅缩短 dev 首屏加载
        'apexcharts',
        'vue3-apexcharts',
        'marked',
        'dompurify',
        'ansi_up',
        '@tauri-apps/api',
        '@tauri-apps/api/core',
        '@tauri-apps/api/app',
        '@tauri-apps/api/event',
        '@tauri-apps/api/window',
        '@tanstack/vue-virtual',
      ]
    },
    test: {
      projects: [{
        extends: true,
        plugins: [
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
        },
      }]
    }
  };
});
