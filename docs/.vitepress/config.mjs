import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

export default withMermaid(
  defineConfig({
    title: 'CCR 技术文档',
    description: 'Claude Code Configuration Switcher - Rust 实现的配置管理工具',
    
    // 基础配置
    base: '/',
    lang: 'zh-CN',
    
    // 主题配置
    themeConfig: {
      // 网站标题
      siteTitle: 'CCR 技术文档',
      
      // Logo
      logo: '/logo.svg',
      
      // 导航栏
      nav: [
        { text: '首页', link: '/' },
        { text: '快速开始', link: '/quick-start' },
        { text: '架构设计', link: '/architecture/' },
        { text: '安装指南', link: '/installation/' },
        { text: '命令参考', link: '/commands/' },
        { text: 'API 参考', link: '/api/' },
        { text: '开发指南', link: '/development/' },
        { 
          text: 'v0.2.0',
          items: [
            { text: '更新日志', link: '/changelog' },
            { text: '迁移指南', link: '/migration' }
          ]
        },
        { 
          text: '源码',
          link: 'https://github.com/bahayonghang/ccs'
        }
      ],
      
      // 侧边栏
      sidebar: {
        '/architecture/': [
          {
            text: '架构设计',
            items: [
              { text: '整体架构', link: '/architecture/' },
              { text: '核心模块', link: '/architecture/modules' },
              { text: '数据流程', link: '/architecture/data-flow' },
              { text: '设计决策', link: '/architecture/design-decisions' },
              { text: 'CCS 对比', link: '/architecture/ccs-comparison' }
            ]
          },
          {
            text: '技术实现',
            items: [
              { text: '文件锁机制', link: '/architecture/locking' },
              { text: '原子操作', link: '/architecture/atomic-operations' },
              { text: '错误处理', link: '/architecture/error-handling' },
              { text: '审计追踪', link: '/architecture/audit-trail' }
            ]
          }
        ],
        '/installation/': [
          {
            text: '安装指南',
            items: [
              { text: '安装概览', link: '/installation/' },
              { text: '从源码构建', link: '/installation/build-from-source' },
              { text: '配置文件', link: '/installation/configuration' },
              { text: '环境变量', link: '/installation/environment' },
              { text: '故障排除', link: '/installation/troubleshooting' }
            ]
          }
        ],
        '/commands/': [
          {
            text: '命令参考',
            items: [
              { text: '命令概览', link: '/commands/' },
              { text: 'list / ls', link: '/commands/list' },
              { text: 'current / status', link: '/commands/current' },
              { text: 'switch', link: '/commands/switch' },
              { text: 'validate / check', link: '/commands/validate' },
              { text: 'history', link: '/commands/history' },
              { text: 'web', link: '/commands/web' },
              { text: 'version', link: '/commands/version' }
            ]
          }
        ],
        '/api/': [
          {
            text: 'API 参考',
            items: [
              { text: 'API 概览', link: '/api/' },
              { text: '配置管理', link: '/api/config' },
              { text: '设置管理', link: '/api/settings' },
              { text: '历史记录', link: '/api/history' },
              { text: '文件锁', link: '/api/lock' },
              { text: '错误类型', link: '/api/errors' }
            ]
          },
          {
            text: 'Web API',
            items: [
              { text: 'RESTful 接口', link: '/api/web-api' },
              { text: '前端集成', link: '/api/frontend' }
            ]
          }
        ],
        '/development/': [
          {
            text: '开发指南',
            items: [
              { text: '开发概览', link: '/development/' },
              { text: '项目结构', link: '/development/structure' },
              { text: '构建系统', link: '/development/build' },
              { text: '测试指南', link: '/development/testing' },
              { text: '代码规范', link: '/development/code-style' },
              { text: '贡献指南', link: '/development/contributing' }
            ]
          },
          {
            text: '高级主题',
            items: [
              { text: '添加新命令', link: '/development/add-command' },
              { text: '扩展 API', link: '/development/extend-api' },
              { text: '性能优化', link: '/development/performance' },
              { text: '安全考虑', link: '/development/security' }
            ]
          }
        ]
      },
      
      // 社交链接
      socialLinks: [
        { icon: 'github', link: 'https://github.com/bahayonghang/ccs' }
      ],
      
      // 页脚
      footer: {
        message: 'Released under the MIT License.',
        copyright: 'Copyright © 2024-2025 Yonghang Li'
      },
      
      // 搜索
      search: {
        provider: 'local',
        options: {
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
      },
      
      // 编辑链接
      editLink: {
        pattern: 'https://github.com/bahayonghang/ccs/edit/main/docs/:path',
        text: '在 GitHub 上编辑此页'
      },
      
      // 最后更新时间
      lastUpdated: {
        text: '最后更新于',
        formatOptions: {
          dateStyle: 'short',
          timeStyle: 'medium'
        }
      },
      
      // 大纲配置
      outline: {
        level: [2, 3],
        label: '页面导航'
      },
      
      // 文档页脚
      docFooter: {
        prev: '上一页',
        next: '下一页'
      },
      
      // 移动端菜单文本
      darkModeSwitchLabel: '主题',
      sidebarMenuLabel: '菜单',
      returnToTopLabel: '返回顶部',
      langMenuLabel: '多语言'
    },
    
    // Markdown配置
    markdown: {
      lineNumbers: true,
      theme: {
        light: 'catppuccin-latte',
        dark: 'catppuccin-mocha'
      },
      // 代码组支持
      codeTransformers: [
        // 我们可以在这里添加自定义的代码转换器
      ]
    },
    
    // 构建配置
    vite: {
      build: {
        chunkSizeWarningLimit: 1600
      },
      // Mermaid插件需要的优化配置
      optimizeDeps: {
        include: ['@braintree/sanitize-url']
      }
    },
    
    // 忽略死链接检查（用于文档占位链接）
    ignoreDeadLinks: true,
    
    // 头部配置
    head: [
      ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
      ['meta', { name: 'theme-color', content: '#8b5cf6' }],
      ['meta', { name: 'og:type', content: 'website' }],
      ['meta', { name: 'og:locale', content: 'zh_CN' }],
      ['meta', { name: 'og:site_name', content: 'CCR 技术文档' }],
      ['meta', { name: 'og:description', content: 'Claude Code Configuration Switcher - Rust 实现的配置管理工具' }]
    ]
  })
)

