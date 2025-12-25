import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

// 后端端口配置（与 justfile 保持一致）
const backendPort = process.env.BACKEND_PORT || '48081'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
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
        },
      },
    },
    // 分块大小警告阈值
    chunkSizeWarningLimit: 1000,
  },
  // 开发服务器优化
  server: {
    port: 5173,
    hmr: {
      overlay: true,
    },
    proxy: {
      '/api': {
        target: `http://localhost:${backendPort}`,
        changeOrigin: true,
      },
    },
  },
  // 依赖优化
  optimizeDeps: {
    include: ['vue', 'vue-router', 'pinia', 'axios'],
  },
})