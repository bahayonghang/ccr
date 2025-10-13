import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "CCR Desktop 文档",
  description: "CCR Tauri 桌面应用 - 完整的技术文档",
  base: '/ccr-tauri/',

  head: [
    ['link', { rel: 'icon', href: '/ccr-tauri/favicon.ico' }]
  ],

  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: '⚡',

    nav: [
      { text: '首页', link: '/' },
      { text: '快速开始', link: '/guide/getting-started' },
      { text: '架构', link: '/architecture/overview' },
      { text: 'API', link: '/api/commands' },
      { text: 'GitHub', link: 'https://github.com/harleyqing/ccr' }
    ],

    sidebar: [
      {
        text: '介绍',
        items: [
          { text: '什么是 CCR Desktop', link: '/introduction' },
          { text: '特性', link: '/features' },
          { text: '技术栈', link: '/tech-stack' }
        ]
      },
      {
        text: '快速开始',
        items: [
          { text: '安装', link: '/guide/installation' },
          { text: '快速开始', link: '/guide/getting-started' },
          { text: '开发环境', link: '/guide/development' },
          { text: '构建发布', link: '/guide/build' }
        ]
      },
      {
        text: '架构设计',
        items: [
          { text: '总体架构', link: '/architecture/overview' },
          { text: '前端架构', link: '/architecture/frontend' },
          { text: '后端架构', link: '/architecture/backend' },
          { text: '通信机制', link: '/architecture/communication' },
          { text: '安全策略', link: '/architecture/security' }
        ]
      },
      {
        text: 'API 参考',
        items: [
          { text: 'Tauri Commands', link: '/api/commands' },
          { text: '前端 API', link: '/api/frontend' },
          { text: '类型定义', link: '/api/types' }
        ]
      },
      {
        text: '开发指南',
        items: [
          { text: '开发入门', link: '/development/getting-started' },
          { text: '项目结构', link: '/development/structure' },
          { text: '添加新功能', link: '/development/add-feature' },
          { text: '调试技巧', link: '/development/debugging' },
          { text: '最佳实践', link: '/development/best-practices' }
        ]
      },
      {
        text: '配置',
        items: [
          { text: 'Tauri 配置', link: '/config/tauri' },
          { text: 'Vite 配置', link: '/config/vite' },
          { text: '文件权限', link: '/config/permissions' }
        ]
      },
      {
        text: '部署',
        items: [
          { text: '打包应用', link: '/deployment/packaging' },
          { text: '代码签名', link: '/deployment/signing' },
          { text: 'GitHub Actions', link: '/deployment/ci-cd' }
        ]
      },
      {
        text: '故障排查',
        items: [
          { text: '常见问题', link: '/troubleshooting/faq' },
          { text: '调试日志', link: '/troubleshooting/logging' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/harleyqing/ccr' }
    ],

    search: {
      provider: 'local'
    },

    footer: {
      message: 'Made with ❤️ by 哈雷酱',
      copyright: 'Copyright © 2024 CCR Desktop'
    },

    editLink: {
      pattern: 'https://github.com/harleyqing/ccr/edit/main/docs_ccr-tauri/:path',
      text: '在 GitHub 上编辑此页'
    },

    lastUpdated: {
      text: '最后更新',
      formatOptions: {
        dateStyle: 'full',
        timeStyle: 'medium'
      }
    }
  },

  markdown: {
    lineNumbers: true,
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    }
  }
})
