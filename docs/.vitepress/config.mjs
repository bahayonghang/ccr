import { defineConfig } from 'vitepress'

// 共享配置
const sharedConfig = {
  // Markdown 配置
  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    },
    lineNumbers: true
  },
  // 外观配置 - 支持深浅主题切换
  appearance: true
}

// https://vitepress.dev/reference/site-config
export default defineConfig({
  ...sharedConfig,

  title: "CCR",
  description: "Claude Code Configuration Switcher",

  // 忽略死链接 - 文档仍在持续补充中
  ignoreDeadLinks: true,

  // 国际化配置
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      themeConfig: {
        logo: '/logo.svg',

        nav: [
          { text: '首页', link: '/' },
          { text: '快速开始', link: '/guide/quick-start' },
          { text: '核心命令', link: '/reference/commands/' },
          { text: 'Web 指南', link: '/guide/web-guide' },
          { text: '更新日志', link: '/reference/changelog' }
        ],

        sidebar: {
          '/': [
            {
              text: '📚 入门指南',
              collapsed: false,
              items: [
                { text: '简介', link: '/' },
                { text: '快速开始', link: '/guide/quick-start' },
                { text: 'Web 界面指南', link: '/guide/web-guide' },
                { text: '配置管理', link: '/guide/configuration' }
              ]
            },
            {
              text: '🏗️ 技术参考',
              collapsed: false,
              items: [
                { text: '架构设计', link: '/reference/architecture' },
                { text: 'Web API 参考', link: '/reference/api' },
                { text: '更新日志', link: '/reference/changelog' }
              ]
            },
            {
              text: '⌨️ 核心命令',
              collapsed: false,
              items: [
                { text: '命令概览', link: '/reference/commands/' },
                {
                  text: '平台与初始化',
                  collapsed: true,
                  items: [
                    { text: 'platform - 平台管理', link: '/reference/commands/platform' },
                    { text: 'migrate - 配置迁移', link: '/reference/commands/migrate' },
                    { text: 'init - 初始化', link: '/reference/commands/init' }
                  ]
                },
                {
                  text: '配置操作',
                  collapsed: true,
                  items: [
                    { text: 'add - 添加配置', link: '/reference/commands/add' },
                    { text: 'delete - 删除配置', link: '/reference/commands/delete' },
                    { text: 'list - 列出配置', link: '/reference/commands/list' },
                    { text: 'current - 当前配置', link: '/reference/commands/current' },
                    { text: 'switch - 切换配置', link: '/reference/commands/switch' },
                    { text: 'run - 临时运行', link: '/reference/commands/run' },
                    { text: 'validate - 验证配置', link: '/reference/commands/validate' },
                    { text: 'enable - 启用配置', link: '/reference/commands/enable' },
                    { text: 'disable - 禁用配置', link: '/reference/commands/disable' },
                    { text: 'clear - 清理配置', link: '/reference/commands/clear' },
                    { text: 'optimize - 优化配置', link: '/reference/commands/optimize' }
                  ]
                },
                {
                  text: '数据管理',
                  collapsed: true,
                  items: [
                    { text: 'history - 操作历史', link: '/reference/commands/history' },
                    { text: 'export - 导出配置', link: '/reference/commands/export' },
                    { text: 'import - 导入配置', link: '/reference/commands/import' },
                    { text: 'clean - 清理备份', link: '/reference/commands/clean' }
                  ]
                },
                {
                  text: '界面与服务',
                  collapsed: true,
                  items: [
                    { text: 'tui - 终端界面', link: '/reference/commands/tui' },
                    { text: 'web - Web 界面', link: '/reference/commands/web' },
                    { text: 'ui - CCR UI', link: '/reference/commands/ui' }
                  ]
                },
                {
                  text: '高级功能',
                  collapsed: true,
                  items: [
                    { text: 'check - 冲突检测', link: '/reference/commands/check' },
                    { text: 'stats - 统计分析', link: '/reference/commands/stats' },
                    { text: 'budget - 预算管理', link: '/reference/commands/budget' },
                    { text: 'pricing - 模型定价', link: '/reference/commands/pricing' },
                    { text: 'sync - 云同步', link: '/reference/commands/sync' },
                    { text: 'skills - 技能管理', link: '/reference/commands/skills' },
                    { text: 'prompts - 提示词', link: '/reference/commands/prompts' },
                    { text: 'temp-token - 临时令牌', link: '/reference/commands/temp-token' },
                    { text: 'provider - 健康检查', link: '/reference/commands/provider' },
                    { text: 'sessions - 会话管理', link: '/reference/commands/sessions' },
                    { text: 'update - 更新', link: '/reference/commands/update' },
                    { text: 'version - 版本', link: '/reference/commands/version' }
                  ]
                }
              ]
            },
            {
              text: '🔌 平台支持',
              collapsed: false,
              items: [
                { text: '平台概览', link: '/reference/platforms/' },
                { text: 'Claude Code', link: '/reference/platforms/claude' },
                { text: 'Codex (Copilot)', link: '/reference/platforms/codex' },
                { text: 'Gemini CLI', link: '/reference/platforms/gemini' },
                { text: 'Factory Droid', link: '/reference/platforms/droid' },
                { text: '平台迁移', link: '/reference/platforms/migration' }
              ]
            },
            {
              text: '📝 示例',
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

        // 大纲标题 - 显示 h2 和 h3
        outline: {
          level: [2, 3],
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
          { text: 'Quick Start', link: '/en/guide/quick-start' },
          { text: 'Commands', link: '/en/reference/commands/' },
          { text: 'Web Guide', link: '/en/guide/web-guide' },
          { text: 'Changelog', link: '/en/reference/changelog' }
        ],

        sidebar: {
          '/en/': [
            {
              text: 'Guide',
              items: [
                { text: 'Introduction', link: '/en/' },
                { text: 'Quick Start', link: '/en/guide/quick-start' },
                { text: 'Web Guide', link: '/en/guide/web-guide' },
                { text: 'Configuration', link: '/en/guide/configuration' }
              ]
            },
            {
              text: 'Reference',
              collapsed: false,
              items: [
                { text: 'Architecture', link: '/en/reference/architecture' },
                { text: 'Web API Reference', link: '/en/reference/api' },
                { text: 'Changelog', link: '/en/reference/changelog' },
                { text: 'Migration Guide', link: '/en/reference/migration' }
              ]
            },
            {
              text: 'Commands',
              collapsed: false,
              items: [
                { text: 'Overview', link: '/en/reference/commands/' },
                {
                  text: 'Platform & Init',
                  collapsed: true,
                  items: [
                    { text: 'platform - Platform Registry', link: '/en/reference/commands/platform' },
                    { text: 'migrate - Migration', link: '/en/reference/commands/migrate' },
                    { text: 'init - Initialize', link: '/en/reference/commands/init' }
                  ]
                },
                {
                  text: 'Profile Operations',
                  collapsed: true,
                  items: [
                    { text: 'add - Add Profile', link: '/en/reference/commands/add' },
                    { text: 'delete - Delete Profile', link: '/en/reference/commands/delete' },
                    { text: 'list - List Profiles', link: '/en/reference/commands/list' },
                    { text: 'current - Current Profile', link: '/en/reference/commands/current' },
                    { text: 'switch - Switch Profile', link: '/en/reference/commands/switch' },
                    { text: 'run - Temp Run Profile', link: '/en/reference/commands/run' },
                    { text: 'validate - Validate', link: '/en/reference/commands/validate' },
                    { text: 'enable - Enable Profile', link: '/en/reference/commands/enable' },
                    { text: 'disable - Disable Profile', link: '/en/reference/commands/disable' },
                    { text: 'clear - Clear Config', link: '/en/reference/commands/clear' },
                    { text: 'optimize - Optimize Config', link: '/en/reference/commands/optimize' }
                  ]
                },
                {
                  text: 'Data Management',
                  collapsed: true,
                  items: [
                    { text: 'history - History', link: '/en/reference/commands/history' },
                    { text: 'export - Export', link: '/en/reference/commands/export' },
                    { text: 'import - Import', link: '/en/reference/commands/import' },
                    { text: 'clean - Clean Backups', link: '/en/reference/commands/clean' }
                  ]
                },
                {
                  text: 'Interfaces',
                  collapsed: true,
                  items: [
                    { text: 'tui - Terminal UI', link: '/en/reference/commands/tui' },
                    { text: 'web - Web Interface', link: '/en/reference/commands/web' },
                    { text: 'ui - CCR UI', link: '/en/reference/commands/ui' }
                  ]
                },
                {
                  text: 'Advanced',
                  collapsed: true,
                  items: [
                    { text: 'check - Conflict Detection', link: '/en/reference/commands/check' },
                    { text: 'stats - Statistics', link: '/en/reference/commands/stats' },
                    { text: 'budget - Budgeting', link: '/en/reference/commands/budget' },
                    { text: 'pricing - Model Pricing', link: '/en/reference/commands/pricing' },
                    { text: 'sync - Cloud Sync', link: '/en/reference/commands/sync' },
                    { text: 'skills - Skills Management', link: '/en/reference/commands/skills' },
                    { text: 'prompts - Prompt Presets', link: '/en/reference/commands/prompts' },
                    { text: 'temp-token - Temp Token', link: '/en/reference/commands/temp-token' },
                    { text: 'provider - Health Check', link: '/en/reference/commands/provider' },
                    { text: 'sessions - Session Management', link: '/en/reference/commands/sessions' },
                    { text: 'update - Update CCR', link: '/en/reference/commands/update' },
                    { text: 'version - Version Info', link: '/en/reference/commands/version' }
                  ]
                }
              ]
            },
            {
              text: 'Platforms',
              collapsed: false,
              items: [
                { text: 'Overview', link: '/en/reference/platforms/' },
                { text: 'Claude Code', link: '/en/reference/platforms/claude' },
                { text: 'Codex (GitHub Copilot)', link: '/en/reference/platforms/codex' },
                { text: 'Gemini CLI', link: '/en/reference/platforms/gemini' },
                { text: 'Factory Droid', link: '/en/reference/platforms/droid' },
                { text: 'Platform Migration', link: '/en/reference/platforms/migration' }
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
