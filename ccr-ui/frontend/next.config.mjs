/** @type {import('next').NextConfig} */
const nextConfig = {
  // 启用 Turbopack（Next.js 16 默认打包器，2-5x 构建速度提升）
  experimental: {
    // 启用文件系统缓存（Beta）
    turbopackFileSystemCacheForDev: true,
  },

  // Turbopack 配置 - 设置 workspace root 消除警告
  turbopack: {
    root: process.cwd(),
  },

  // 生产构建优化
  compiler: {
    removeConsole: process.env.NODE_ENV === 'production' ? {
      exclude: ['error', 'warn'],
    } : false,
  },

  // 配置 API 代理（开发环境）
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://localhost:8081/api/:path*',
      },
    ];
  },

  // 性能优化配置
  poweredByHeader: false,
  compress: true,

  // 图片优化
  images: {
    formats: ['image/avif', 'image/webp'],
    deviceSizes: [640, 750, 828, 1080, 1200, 1920, 2048, 3840],
    imageSizes: [16, 32, 48, 64, 96, 128, 256, 384],
  },
};

export default nextConfig;

