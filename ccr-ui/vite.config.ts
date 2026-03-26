/// <reference types="vitest/config" />
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { fileURLToPath, URL } from 'node:url';
import path from 'node:path';
import { storybookTest } from '@storybook/addon-vitest/vitest-plugin';
import { playwright } from '@vitest/browser-playwright';

const dirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url));

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
    reportCompressedSize: true,
    rollupOptions: {
      output: {
        manualChunks: {
          'vue-vendor': ['vue', 'vue-router', 'pinia'],
          'ui-vendor': ['@iconify/vue'],
          'charts-vendor': ['apexcharts', 'vue3-apexcharts'],
          'i18n-vendor': ['vue-i18n']
        }
      }
    },
    chunkSizeWarningLimit: 1000
  },
  server: {
    port: 15173,
    strictPort: true,
    warmup: {
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
      'marked',
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
