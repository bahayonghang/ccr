import { defineConfig } from 'vitepress'

const sharedConfig = {
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    ['link', { rel: 'alternate icon', type: 'image/png', href: '/favicon.png' }]
  ],
  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    },
    lineNumbers: true
  },
  appearance: true
}

const zhTheme = {
  logo: '/logo.svg',
  nav: [
    { text: '首页', link: '/' },
    { text: '快速开始', link: '/guide/quick-start' },
    { text: 'CLI', link: '/reference/commands/' },
    { text: 'UI', link: '/guide/ui-overview' },
    { text: '架构', link: '/reference/architecture' },
    { text: '更新日志', link: '/reference/changelog' }
  ],
  sidebar: {
    '/': [
      {
        text: '概览',
        collapsed: false,
        items: [
          { text: '项目简介', link: '/' },
          { text: '快速开始', link: '/guide/quick-start' },
          { text: '配置模型', link: '/guide/configuration' },
          { text: 'CLI 工作流', link: '/guide/cli-workflows' },
          { text: 'GitHub Copilot 工作区', link: '/guide/github-copilot-workspace' }
        ]
      },
      {
        text: '入口与 UI',
        collapsed: false,
        items: [
          { text: '入口选择：CLI / TUI / CCR UI', link: '/guide/entrypoints' },
          { text: 'UI 概览', link: '/guide/ui-overview' },
          { text: 'UI 模块地图', link: '/guide/ui-modules' }
        ]
      },
      {
        text: 'CLI 参考',
        collapsed: false,
        items: [
          { text: '命令总览', link: '/reference/commands/' },
          {
            text: '平台与初始化',
            collapsed: true,
            items: [
              { text: 'platform', link: '/reference/commands/platform' },
              { text: 'claude', link: '/reference/commands/claude' },
              { text: 'codex', link: '/reference/commands/codex' },
              { text: 'opencode', link: '/reference/commands/opencode' },
              { text: 'init', link: '/reference/commands/init' }
            ]
          },
          {
            text: 'Profile 与临时覆盖',
            collapsed: true,
            items: [
              { text: 'add', link: '/reference/commands/add' },
              { text: 'delete', link: '/reference/commands/delete' },
              { text: 'list', link: '/reference/commands/list' },
              { text: 'current', link: '/reference/commands/current' },
              { text: 'switch', link: '/reference/commands/switch' },
              { text: 'temp', link: '/reference/commands/temp' },
              { text: 'temp-token', link: '/reference/commands/temp-token' },
              { text: 'validate', link: '/reference/commands/validate' },
              { text: 'enable', link: '/reference/commands/enable' },
              { text: 'disable', link: '/reference/commands/disable' },
              { text: 'clear', link: '/reference/commands/clear' },
              { text: 'optimize', link: '/reference/commands/optimize' }
            ]
          },
          {
            text: '数据、同步与诊断',
            collapsed: true,
            items: [
              { text: 'history', link: '/reference/commands/history' },
              { text: 'export', link: '/reference/commands/export' },
              { text: 'import', link: '/reference/commands/import' },
              { text: 'clean', link: '/reference/commands/clean' },
              { text: 'sync', link: '/reference/commands/sync' },
              { text: 'sessions', link: '/reference/commands/sessions' },
              { text: 'provider', link: '/reference/commands/provider' },
              { text: 'check', link: '/reference/commands/check' }
            ]
          },
          {
            text: '界面、成本与维护',
            collapsed: true,
            items: [
              { text: 'ui', link: '/reference/commands/ui' },
              { text: 'tui', link: '/reference/commands/tui' },
              { text: 'stats', link: '/reference/commands/stats' },
              { text: 'budget', link: '/reference/commands/budget' },
              { text: 'pricing', link: '/reference/commands/pricing' },
              { text: 'skills', link: '/reference/commands/skills' },
              { text: 'prompts', link: '/reference/commands/prompts' },
              { text: 'update', link: '/reference/commands/update' },
              { text: 'version', link: '/reference/commands/version' }
            ]
          }
        ]
      },
      {
        text: '参考资料',
        collapsed: false,
        items: [
          { text: '架构设计', link: '/reference/architecture' },
          { text: 'Crate 地图', link: '/reference/internals/crate-map' },
          { text: '运行时流程', link: '/reference/internals/runtime-flows' },
          { text: '平台支持', link: '/reference/platforms/' },
          { text: '迁移指南', link: '/reference/migration' },
          { text: 'Release 签名验证', link: '/reference/release-verification' },
          { text: '更新日志', link: '/reference/changelog' }
        ]
      },
      {
        text: '示例',
        collapsed: true,
        items: [
          { text: '示例概览', link: '/examples/' },
          { text: '多平台设置', link: '/examples/multi-platform-setup' },
          { text: '故障排除', link: '/examples/troubleshooting' }
        ]
      }
    ]
  },
  socialLinks: [
    { icon: 'github', link: 'https://github.com/bahayonghang/ccr' }
  ],
  footer: {
    message: '基于 MIT 许可发布',
    copyright: 'Copyright © 2025-present'
  },
  search: {
    provider: 'local',
    options: {
      locales: {
        root: {
          translations: {
            button: {
              buttonText: '搜索文档',
              buttonAriaLabel: '搜索文档'
            },
            modal: {
              noResultsText: '无法找到相关结果',
              resetButtonTitle: '清除查询条件',
              footer: {
                selectText: '选择',
                navigateText: '切换'
              }
            }
          }
        }
      }
    }
  },
  editLink: {
    pattern: 'https://github.com/bahayonghang/ccr/edit/main/docs/:path',
    text: '在 GitHub 上编辑此页'
  },
  lastUpdated: {
    text: '最后更新',
    formatOptions: {
      dateStyle: 'short',
      timeStyle: 'short'
    }
  },
  docFooter: {
    prev: '上一页',
    next: '下一页'
  },
  outline: {
    level: [2, 3],
    label: '页面导航'
  },
  returnToTopLabel: '回到顶部',
  sidebarMenuLabel: '菜单',
  darkModeSwitchLabel: '主题',
  lightModeSwitchTitle: '切换到浅色模式',
  darkModeSwitchTitle: '切换到深色模式'
}

const enTheme = {
  logo: '/logo.svg',
  nav: [
    { text: 'Home', link: '/en/' },
    { text: 'Quick Start', link: '/en/guide/quick-start' },
    { text: 'CLI', link: '/en/reference/commands/' },
    { text: 'UI', link: '/en/guide/ui-overview' },
    { text: 'Architecture', link: '/en/reference/architecture' },
    { text: 'Changelog', link: '/en/reference/changelog' }
  ],
  sidebar: {
    '/en/': [
      {
        text: 'Overview',
        collapsed: false,
        items: [
          { text: 'Project Overview', link: '/en/' },
          { text: 'Quick Start', link: '/en/guide/quick-start' },
          { text: 'Configuration Model', link: '/en/guide/configuration' },
          { text: 'CLI Workflows', link: '/en/guide/cli-workflows' },
          { text: 'GitHub Copilot Workspace', link: '/en/guide/github-copilot-workspace' }
        ]
      },
      {
        text: 'Entrypoints and UI',
        collapsed: false,
        items: [
          { text: 'Choosing CLI / TUI / CCR UI', link: '/en/guide/entrypoints' },
          { text: 'UI Overview', link: '/en/guide/ui-overview' },
          { text: 'UI Module Map', link: '/en/guide/ui-modules' }
        ]
      },
      {
        text: 'CLI Reference',
        collapsed: false,
        items: [
          { text: 'Command Overview', link: '/en/reference/commands/' },
          {
            text: 'Platform and Init',
            collapsed: true,
            items: [
              { text: 'platform', link: '/en/reference/commands/platform' },
              { text: 'claude', link: '/en/reference/commands/claude' },
              { text: 'codex', link: '/en/reference/commands/codex' },
              { text: 'opencode', link: '/en/reference/commands/opencode' },
              { text: 'init', link: '/en/reference/commands/init' }
            ]
          },
          {
            text: 'Profiles and Overrides',
            collapsed: true,
            items: [
              { text: 'add', link: '/en/reference/commands/add' },
              { text: 'delete', link: '/en/reference/commands/delete' },
              { text: 'list', link: '/en/reference/commands/list' },
              { text: 'current', link: '/en/reference/commands/current' },
              { text: 'switch', link: '/en/reference/commands/switch' },
              { text: 'temp', link: '/en/reference/commands/temp' },
              { text: 'temp-token', link: '/en/reference/commands/temp-token' },
              { text: 'validate', link: '/en/reference/commands/validate' },
              { text: 'enable', link: '/en/reference/commands/enable' },
              { text: 'disable', link: '/en/reference/commands/disable' },
              { text: 'clear', link: '/en/reference/commands/clear' },
              { text: 'optimize', link: '/en/reference/commands/optimize' }
            ]
          },
          {
            text: 'Data, Sync, and Diagnostics',
            collapsed: true,
            items: [
              { text: 'history', link: '/en/reference/commands/history' },
              { text: 'export', link: '/en/reference/commands/export' },
              { text: 'import', link: '/en/reference/commands/import' },
              { text: 'clean', link: '/en/reference/commands/clean' },
              { text: 'sync', link: '/en/reference/commands/sync' },
              { text: 'sessions', link: '/en/reference/commands/sessions' },
              { text: 'provider', link: '/en/reference/commands/provider' },
              { text: 'check', link: '/en/reference/commands/check' }
            ]
          },
          {
            text: 'Interfaces, Cost, and Maintenance',
            collapsed: true,
            items: [
              { text: 'ui', link: '/en/reference/commands/ui' },
              { text: 'tui', link: '/en/reference/commands/tui' },
              { text: 'stats', link: '/en/reference/commands/stats' },
              { text: 'budget', link: '/en/reference/commands/budget' },
              { text: 'pricing', link: '/en/reference/commands/pricing' },
              { text: 'skills', link: '/en/reference/commands/skills' },
              { text: 'prompts', link: '/en/reference/commands/prompts' },
              { text: 'update', link: '/en/reference/commands/update' },
              { text: 'version', link: '/en/reference/commands/version' }
            ]
          }
        ]
      },
      {
        text: 'Reference',
        collapsed: false,
        items: [
          { text: 'Architecture', link: '/en/reference/architecture' },
          { text: 'Crate Map', link: '/en/reference/internals/crate-map' },
          { text: 'Runtime Flows', link: '/en/reference/internals/runtime-flows' },
          { text: 'Platforms', link: '/en/reference/platforms/' },
          { text: 'Migration Guide', link: '/en/reference/migration' },
          { text: 'Release Verification', link: '/en/reference/release-verification' },
          { text: 'Changelog', link: '/en/reference/changelog' }
        ]
      },
      {
        text: 'Examples',
        collapsed: true,
        items: [
          { text: 'Overview', link: '/en/examples/' },
          { text: 'Multi-Platform Setup', link: '/en/examples/multi-platform-setup' },
          { text: 'Troubleshooting', link: '/en/examples/troubleshooting' }
        ]
      }
    ]
  },
  socialLinks: [
    { icon: 'github', link: 'https://github.com/bahayonghang/ccr' }
  ],
  footer: {
    message: 'Released under the MIT License',
    copyright: 'Copyright © 2025-present'
  },
  search: {
    provider: 'local'
  },
  editLink: {
    pattern: 'https://github.com/bahayonghang/ccr/edit/main/docs/:path',
    text: 'Edit this page on GitHub'
  },
  lastUpdated: {
    text: 'Last updated',
    formatOptions: {
      dateStyle: 'short',
      timeStyle: 'short'
    }
  },
  docFooter: {
    prev: 'Previous page',
    next: 'Next page'
  },
  outline: {
    level: [2, 3],
    label: 'On this page'
  },
  returnToTopLabel: 'Return to top',
  sidebarMenuLabel: 'Menu',
  darkModeSwitchLabel: 'Theme',
  lightModeSwitchTitle: 'Switch to light mode',
  darkModeSwitchTitle: 'Switch to dark mode'
}

export default defineConfig({
  ...sharedConfig,
  title: 'CCR',
  description: 'Unified configuration registry for modern AI CLIs',
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      themeConfig: zhTheme
    },
    en: {
      label: 'English',
      lang: 'en-US',
      link: '/en/',
      themeConfig: enTheme
    }
  }
})
