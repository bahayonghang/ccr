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
          // 将 axios 单独打包
          'http-vendor': ['axios'],
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
      // 预热关键模块：服务器就绪后立即变换，浏览器请求前已完成
      // 包含 client.ts 的全部 16 个 barrel re-export 模块，
      // 避免浏览器首次请求时触发 16 次串行 on-demand 变换
      clientFiles: [
        './src/main.ts',
        './src/App.vue',
        './src/components/MainLayout.vue',
        './src/views/HomeView.vue',
        // API 层：core + client barrel + 所有子模块（共 16 个）
        './src/api/core.ts',
        './src/api/client.ts',
        './src/api/modules/stats.ts',
        './src/api/modules/config.ts',
        './src/api/modules/mcp.ts',
        './src/api/modules/agents.ts',
        './src/api/modules/slashCommands.ts',
        './src/api/modules/plugins.ts',
        './src/api/modules/hooks.ts',
        './src/api/modules/skills.ts',
        './src/api/modules/checkin.ts',
        './src/api/modules/outputStyles.ts',
        './src/api/modules/statusline.ts',
        './src/api/modules/converter.ts',
        './src/api/modules/sync.ts',
        './src/api/modules/codex.ts',
        './src/api/modules/gemini.ts',
        './src/api/modules/qwen.ts',
        './src/api/modules/iflow.ts',
        './src/api/modules/droid.ts',
        './src/api/modules/usageV2.ts',
        './src/api/modules/skillHub.ts',
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
      'axios',
      'lucide-vue-next', // MainLayout 中使用 13+ 图标，需预打包
      'vue-i18n',        // 多处导入的国际化库
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