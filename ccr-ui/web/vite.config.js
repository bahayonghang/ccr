import { defineConfig } from 'vite';

export default defineConfig({
  root: './',
  base: './',
  server: {
    port: 3000,
    open: true,
    proxy: {
      '/api': {
        target: `http://localhost:${process.env.CCR_WEB_PORT || 19527}`,
        changeOrigin: true,
      }
    }
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    assetsDir: '',
    sourcemap: false,
    rollupOptions: {
      output: {
        entryFileNames: 'script.js',
        chunkFileNames: 'script-[hash].js',
        assetFileNames: (assetInfo) => {
          if (assetInfo.name && assetInfo.name.endsWith('.css')) {
            return 'style.css';
          }
          return '[name][extname]';
        }
      }
    }
  }
});
