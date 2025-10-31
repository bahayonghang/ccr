import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "CCR",
  description: "Claude Code Configuration Switcher - 配置管理工具",

  // 主题配置
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    logo: '/logo.svg',

    nav: [
      { text: '首页', link: '/' },
      { text: '快速开始', link: '/quick-start' },
      { text: '核心命令', link: '/commands/' },
      { text: 'Web 指南', link: '/web-guide' },
      { text: '更新日志', link: '/changelog' }
    ],

    sidebar: [
      {
        text: '指南',
        items: [
          { text: '简介', link: '/' },
          { text: '快速开始', link: '/quick-start' },
          { text: 'Web 界面使用指南', link: '/web-guide' },
          { text: '配置管理', link: '/configuration' }
        ]
      },
      {
        text: '核心命令',
        collapsed: false,
        items: [
          { text: '命令概览', link: '/commands/' },
          { text: 'init - 初始化配置', link: '/commands/init' },
          { text: 'list - 列出配置', link: '/commands/list' },
          { text: 'current - 当前配置', link: '/commands/current' },
          { text: 'switch - 切换配置', link: '/commands/switch' },
          { text: 'validate - 验证配置', link: '/commands/validate' },
          { text: 'history - 操作历史', link: '/commands/history' },
          { text: 'web - Web 界面', link: '/commands/web' },
          { text: 'export - 导出配置', link: '/commands/export' },
          { text: 'import - 导入配置', link: '/commands/import' },
          { text: 'clean - 清理备份', link: '/commands/clean' },
          { text: 'update - 更新 CCR', link: '/commands/update' },
          { text: 'version - 版本信息', link: '/commands/version' }
        ]
      },
      {
        text: '其他',
        items: [
          { text: '更新日志', link: '/changelog' },
          { text: '迁移指南', link: '/migration' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/bahayonghang/ccr' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2025-present'
    },

    // 搜索配置
    search: {
      provider: 'local'
    },

    // 编辑链接
    editLink: {
      pattern: 'https://github.com/bahayonghang/ccr/edit/main/docs/:path',
      text: '在 GitHub 上编辑此页'
    },

    // 最后更新时间
    lastUpdated: {
      text: '最后更新',
      formatOptions: {
        dateStyle: 'short',
        timeStyle: 'short'
      }
    }
  },

  // Markdown 配置
  markdown: {
    theme: 'github-dark',
    lineNumbers: true
  },

  // 语言配置
  lang: 'zh-CN'
})
