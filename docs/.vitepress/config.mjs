import { defineConfig } from 'vitepress'

// 共享配置
const sharedConfig = {
  // Markdown 配置
  markdown: {
    theme: 'github-dark',
    lineNumbers: true
  }
}

// https://vitepress.dev/reference/site-config
export default defineConfig({
  ...sharedConfig,
  
  title: "CCR",
  description: "Claude Code Configuration Switcher",

  // 忽略死链接配置
  ignoreDeadLinks: [
    // 本地开发链接
    /^http:\/\/localhost/,
    // 待创建的文档
    /commands\/platform/,
    /commands\/migrate/,
    /\/migrate$/,
    /TODO_ANALYTICS/,
    /ccr-ui\/docs/,
    /platforms\/README/,
    /commands\/README/,
    // 配置文件示例
    /\.toml$/,
    // 项目根目录文档
    /\/README$/,
    /\/CLAUDE$/,
  ],

  // 国际化配置
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      themeConfig: {
        logo: '/logo.svg',
        
        nav: [
          { text: '首页', link: '/' },
          { text: '快速开始', link: '/quick-start' },
          { text: '核心命令', link: '/commands/' },
          { text: 'Web 指南', link: '/web-guide' },
          { text: '更新日志', link: '/changelog' }
        ],

        sidebar: {
          '/': [
            {
              text: '指南',
              items: [
                { text: '简介', link: '/' },
                { text: '快速开始', link: '/quick-start' },
                { text: 'Web 界面使用指南', link: '/web-guide' },
                { text: '配置管理', link: '/configuration' },
                { text: '架构设计', link: '/architecture' }
              ]
            },
            {
              text: '核心命令',
              collapsed: false,
              items: [
                { text: '命令概览', link: '/commands/' },
                { text: 'init - 初始化配置', link: '/commands/init' },
                { text: 'add - 添加配置', link: '/commands/add' },
                { text: 'delete - 删除配置', link: '/commands/delete' },
                { text: 'list - 列出配置', link: '/commands/list' },
                { text: 'current - 当前配置', link: '/commands/current' },
                { text: 'switch - 切换配置', link: '/commands/switch' },
                { text: 'validate - 验证配置', link: '/commands/validate' },
                { text: 'history - 操作历史', link: '/commands/history' },
                { text: 'tui - 终端界面', link: '/commands/tui' },
                { text: 'web - Web 界面', link: '/commands/web' },
                { text: 'stats - 统计分析', link: '/commands/stats' },
                { text: 'sync - 云同步', link: '/commands/sync' },
                { text: 'temp-token - 临时令牌', link: '/commands/temp-token' },
                { text: 'export - 导出配置', link: '/commands/export' },
                { text: 'import - 导入配置', link: '/commands/import' },
                { text: 'clean - 清理备份', link: '/commands/clean' },
                { text: 'update - 更新 CCR', link: '/commands/update' },
                { text: 'version - 版本信息', link: '/commands/version' }
              ]
            },
            {
              text: '平台支持',
              collapsed: false,
              items: [
                { text: '平台概览', link: '/platforms/' },
                { text: 'Claude Code', link: '/platforms/claude' },
                { text: 'Codex (GitHub Copilot)', link: '/platforms/codex' },
                { text: 'Gemini CLI', link: '/platforms/gemini' },
                { text: '平台迁移', link: '/platforms/migration' }
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
            },
            {
              text: '其他',
              items: [
                { text: '更新日志', link: '/changelog' },
                { text: '迁移指南', link: '/migration' }
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

        // 搜索配置
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
        },

        // 文档页脚
        docFooter: {
          prev: '上一页',
          next: '下一页'
        },

        // 大纲标题
        outline: {
          label: '页面导航'
        },

        // 返回顶部
        returnToTopLabel: '回到顶部',

        // 侧边栏菜单标签
        sidebarMenuLabel: '菜单',

        // 深色模式开关标签
        darkModeSwitchLabel: '主题',

        // 浅色/深色模式开关标题
        lightModeSwitchTitle: '切换到浅色模式',
        darkModeSwitchTitle: '切换到深色模式'
      }
    },

    en: {
      label: 'English',
      lang: 'en-US',
      link: '/en/',
      themeConfig: {
        logo: '/logo.svg',
        
        nav: [
          { text: 'Home', link: '/en/' },
          { text: 'Quick Start', link: '/en/quick-start' },
          { text: 'Commands', link: '/en/commands/' },
          { text: 'Web Guide', link: '/en/web-guide' },
          { text: 'Changelog', link: '/en/changelog' }
        ],

        sidebar: {
          '/en/': [
            {
              text: 'Guide',
              items: [
                { text: 'Introduction', link: '/en/' },
                { text: 'Quick Start', link: '/en/quick-start' },
                { text: 'Web Guide', link: '/en/web-guide' },
                { text: 'Configuration', link: '/en/configuration' },
                { text: 'Architecture', link: '/en/architecture' }
              ]
            },
            {
              text: 'Commands',
              collapsed: false,
              items: [
                { text: 'Overview', link: '/en/commands/' },
                { text: 'init - Initialize', link: '/en/commands/init' },
                { text: 'add - Add Profile', link: '/en/commands/add' },
                { text: 'delete - Delete Profile', link: '/en/commands/delete' },
                { text: 'list - List Profiles', link: '/en/commands/list' },
                { text: 'current - Current Profile', link: '/en/commands/current' },
                { text: 'switch - Switch Profile', link: '/en/commands/switch' },
                { text: 'validate - Validate', link: '/en/commands/validate' },
                { text: 'history - History', link: '/en/commands/history' },
                { text: 'tui - Terminal UI', link: '/en/commands/tui' },
                { text: 'web - Web Interface', link: '/en/commands/web' },
                { text: 'stats - Statistics', link: '/en/commands/stats' },
                { text: 'sync - Cloud Sync', link: '/en/commands/sync' },
                { text: 'temp-token - Temp Token', link: '/en/commands/temp-token' },
                { text: 'export - Export', link: '/en/commands/export' },
                { text: 'import - Import', link: '/en/commands/import' },
                { text: 'clean - Clean Backups', link: '/en/commands/clean' },
                { text: 'update - Update CCR', link: '/en/commands/update' },
                { text: 'version - Version Info', link: '/en/commands/version' }
              ]
            },
            {
              text: 'Platforms',
              collapsed: false,
              items: [
                { text: 'Overview', link: '/en/platforms/' },
                { text: 'Claude Code', link: '/en/platforms/claude' },
                { text: 'Codex (GitHub Copilot)', link: '/en/platforms/codex' },
                { text: 'Gemini CLI', link: '/en/platforms/gemini' },
                { text: 'Migration', link: '/en/platforms/migration' }
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
            },
            {
              text: 'More',
              items: [
                { text: 'Changelog', link: '/en/changelog' },
                { text: 'Migration Guide', link: '/en/migration' }
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

        // Search configuration
        search: {
          provider: 'local'
        },

        // Edit link
        editLink: {
          pattern: 'https://github.com/bahayonghang/ccr/edit/main/docs/:path',
          text: 'Edit this page on GitHub'
        },

        // Last updated
        lastUpdated: {
          text: 'Last updated',
          formatOptions: {
            dateStyle: 'short',
            timeStyle: 'short'
          }
        }
      }
    }
  }
})
