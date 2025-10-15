import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

export default withMermaid(defineConfig({
  // 站点基础配置
  title: 'CCR UI 文档',
  description: 'CCR UI - 现代化的 CCR 配置管理 Web 应用程序',
  lang: 'zh-CN',
  
  // 基础路径配置
  base: '/',
  
  // 主题外观配置
  appearance: true, // 支持深色模式切换
  
  // 头部配置
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    ['meta', { name: 'viewport', content: 'width=device-width, initial-scale=1.0' }],
    ['meta', { name: 'keywords', content: 'CCR, Claude Code, 配置管理, React, Rust, 全栈应用' }]
  ],
  
  // Markdown 配置
  markdown: {
    lineNumbers: true, // 显示行号
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    }
  },
  
  // 主题配置
  themeConfig: {
    // 站点标题和 Logo
    siteTitle: 'CCR UI',
    logo: '/logo.svg',
    
    // 导航栏配置
    nav: [
      { text: '首页', link: '/' },
      { text: '快速开始', link: '/guide/getting-started' },
      {
        text: '前端文档',
        items: [
          { text: '项目概述', link: '/frontend/overview' },
          { text: '开发指南', link: '/frontend/development' },
          { text: '组件文档', link: '/frontend/components' },
          { text: 'API 接口', link: '/frontend/api' }
        ]
      },
      {
        text: '后端文档',
        items: [
          { text: '架构设计', link: '/backend/architecture' },
          { text: '开发指南', link: '/backend/development' },
          { text: 'API 文档', link: '/backend/api' },
          { text: '部署指南', link: '/backend/deployment' }
        ]
      },
      {
        text: '更多',
        items: [
          { text: '贡献指南', link: '/contributing' },
          { text: '更新日志', link: '/changelog' },
          { text: 'FAQ', link: '/faq' }
        ]
      }
    ],
    
    // 侧边栏配置
    sidebar: {
      '/guide/': [
        {
          text: '指南',
          items: [
            { text: '快速开始', link: '/guide/getting-started' },
            { text: '项目结构', link: '/guide/project-structure' },
            { text: '开发环境', link: '/guide/development-setup' },
            { text: '构建部署', link: '/guide/build-deploy' }
          ]
        }
      ],
      '/frontend/': [
        {
          text: '前端文档',
          items: [
            { text: '项目概述', link: '/frontend/overview' },
            { text: '技术栈', link: '/frontend/tech-stack' },
            { text: '开发指南', link: '/frontend/development' },
            { text: '组件文档', link: '/frontend/components' },
            { text: 'API 接口', link: '/frontend/api' },
            { text: '样式指南', link: '/frontend/styling' },
            { text: '测试', link: '/frontend/testing' }
          ]
        }
      ],
      '/backend/': [
        {
          text: '后端文档',
          items: [
            { text: '架构设计', link: '/backend/architecture' },
            { text: '技术栈', link: '/backend/tech-stack' },
            { text: '开发指南', link: '/backend/development' },
            { text: 'API 文档', link: '/backend/api' },
            { text: '数据模型', link: '/backend/models' },
            { text: '错误处理', link: '/backend/error-handling' },
            { text: '部署指南', link: '/backend/deployment' }
          ]
        }
      ]
    },
    
    // 社交链接
    socialLinks: [
      { icon: 'github', link: 'https://github.com/your-username/ccr' }
    ],
    
    // 页脚配置
    footer: {
      message: '基于 MIT 许可证发布',
      copyright: 'Copyright © 2024 CCR UI Team'
    },
    
    // 搜索配置
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
              navigateText: '切换',
              closeText: '关闭'
            }
          }
        }
      }
    },
    
    // 编辑链接
    editLink: {
      pattern: 'https://github.com/your-username/ccr/edit/main/ccr-ui/docs/:path',
      text: '在 GitHub 上编辑此页面'
    },
    
    // 最后更新时间
    lastUpdated: {
      text: '最后更新于',
      formatOptions: {
        dateStyle: 'short',
        timeStyle: 'medium'
      }
    },
    
    // 文档页脚导航
    docFooter: {
      prev: '上一页',
      next: '下一页'
    },
    
    // 大纲配置
    outline: {
      level: [2, 3],
      label: '页面导航'
    },
    
    // 返回顶部
    returnToTopLabel: '回到顶部',
    
    // 侧边栏菜单标签
    sidebarMenuLabel: '菜单',
    
    // 深色模式切换标签
    darkModeSwitchLabel: '主题',
    lightModeSwitchTitle: '切换到浅色模式',
    darkModeSwitchTitle: '切换到深色模式'
  },
  
  // 构建配置
  build: {
    outDir: '../dist/docs'
  }
}))  // withMermaid 闭合括号