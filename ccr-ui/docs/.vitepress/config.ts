import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

// 共享的导航和侧边栏配置
const zhNav = [
  { text: '首页', link: '/' },
  { 
    text: '指南',
    items: [
      { text: '快速开始', link: '/guide/getting-started' },
      { text: '项目结构', link: '/guide/project-structure' },
      { text: '统计功能', link: '/guide/stats' }
    ]
  },
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
  },
  {
    text: '后端文档',
    items: [
      { text: '架构设计', link: '/backend/architecture' },
      { text: '技术栈', link: '/backend/tech-stack' },
      { text: '开发指南', link: '/backend/development' },
      { text: 'API 文档', link: '/backend/api' },
      { text: '错误处理', link: '/backend/error-handling' },
      { text: '部署指南', link: '/backend/deployment' }
    ]
  },
  { text: '贡献指南', link: '/contributing' },
  { text: 'FAQ', link: '/faq' }
]

const enNav = [
  { text: 'Home', link: '/en/' },
  { 
    text: 'Guide',
    items: [
      { text: 'Getting Started', link: '/en/guide/getting-started' },
      { text: 'Project Structure', link: '/en/guide/project-structure' },
      { text: 'Statistics', link: '/en/guide/stats' }
    ]
  },
  {
    text: 'Frontend',
    items: [
      { text: 'Overview', link: '/en/frontend/overview' },
      { text: 'Tech Stack', link: '/en/frontend/tech-stack' },
      { text: 'Development', link: '/en/frontend/development' },
      { text: 'Components', link: '/en/frontend/components' },
      { text: 'API Reference', link: '/en/frontend/api' },
      { text: 'Styling', link: '/en/frontend/styling' },
      { text: 'Testing', link: '/en/frontend/testing' }
    ]
  },
  {
    text: 'Backend',
    items: [
      { text: 'Architecture', link: '/en/backend/architecture' },
      { text: 'Tech Stack', link: '/en/backend/tech-stack' },
      { text: 'Development', link: '/en/backend/development' },
      { text: 'API Reference', link: '/en/backend/api' },
      { text: 'Error Handling', link: '/en/backend/error-handling' },
      { text: 'Deployment', link: '/en/backend/deployment' }
    ]
  },
  { text: 'Contributing', link: '/en/contributing' },
  { text: 'FAQ', link: '/en/faq' }
]

export default withMermaid(defineConfig({
  // 站点基础配置
  title: 'CCR UI',
  description: 'CCR UI - Modern Configuration Management Platform',
  
  // 基础路径配置
  base: '/',
  
  // 忽略死链接（用于外部链接和动态路由）
  ignoreDeadLinks: true,
  
  // 主题外观配置
  appearance: true, // 支持深色模式切换
  
  // 多语言配置
  locales: {
    root: {
      label: '简体中文',
      lang: 'zh-CN',
      title: 'CCR UI 文档',
      description: 'CCR UI - 现代化的 CCR 配置管理 Web 应用程序'
    },
    en: {
      label: 'English',
      lang: 'en-US',
      link: '/en/',
      title: 'CCR UI Documentation',
      description: 'CCR UI - Modern Configuration Management Web Application'
    }
  },
  
  // 头部配置
  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    ['meta', { name: 'viewport', content: 'width=device-width, initial-scale=1.0' }],
    ['meta', { name: 'keywords', content: 'CCR, Claude Code, 配置管理, Vue, Rust, 全栈应用' }]
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
    
    // 社交链接
    socialLinks: [
      { icon: 'github', link: 'https://github.com/your-username/ccr' }
    ],
    
    // 默认语言（中文）导航栏 - 直接在 themeConfig 下
    nav: zhNav,
    
    // 默认语言（中文）侧边栏 - 直接在 themeConfig 下
    sidebar: {
      '/guide/': [
        {
          text: '指南',
          items: [
            { text: '快速开始', link: '/guide/getting-started' },
            { text: '项目结构', link: '/guide/project-structure' },
            { text: '统计功能', link: '/guide/stats' }
          ]
        }
      ],
      '/frontend/': [
        {
          text: '前端文档',
          items: [
            { text: '项目概述', link: '/frontend/overview' },
            { text: '页面架构', link: '/frontend/page-architecture' },
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
            { text: '错误处理', link: '/backend/error-handling' },
            { text: '部署指南', link: '/backend/deployment' }
          ]
        }
      ]
    },
    
    // 默认语言（中文）页脚配置
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
    darkModeSwitchTitle: '切换到深色模式',
    
    // 其他语言配置
    locales: {
      en: {
        label: 'English',
        lang: 'en-US',
        
        // 导航栏
        nav: enNav,
        
        // 侧边栏
        sidebar: {
          '/en/guide/': [
            {
              text: 'Guide',
              items: [
                { text: 'Getting Started', link: '/en/guide/getting-started' },
                { text: 'Project Structure', link: '/en/guide/project-structure' },
                { text: 'Statistics', link: '/en/guide/stats' }
              ]
            }
          ],
          '/en/frontend/': [
            {
              text: 'Frontend',
              items: [
                { text: 'Overview', link: '/en/frontend/overview' },
                { text: 'Page Architecture', link: '/en/frontend/page-architecture' },
                { text: 'Tech Stack', link: '/en/frontend/tech-stack' },
                { text: 'Development', link: '/en/frontend/development' },
                { text: 'Components', link: '/en/frontend/components' },
                { text: 'API Reference', link: '/en/frontend/api' },
                { text: 'Styling', link: '/en/frontend/styling' },
                { text: 'Testing', link: '/en/frontend/testing' }
              ]
            }
          ],
          '/en/backend/': [
            {
              text: 'Backend',
              items: [
                { text: 'Architecture', link: '/en/backend/architecture' },
                { text: 'Tech Stack', link: '/en/backend/tech-stack' },
                { text: 'Development', link: '/en/backend/development' },
                { text: 'API Reference', link: '/en/backend/api' },
                { text: 'Error Handling', link: '/en/backend/error-handling' },
                { text: 'Deployment', link: '/en/backend/deployment' }
              ]
            }
          ]
        },
        
        // 页脚配置
        footer: {
          message: 'Released under the MIT License',
          copyright: 'Copyright © 2024 CCR UI Team'
        },
        
        // 编辑链接
        editLink: {
          pattern: 'https://github.com/your-username/ccr/edit/main/ccr-ui/docs/:path',
          text: 'Edit this page on GitHub'
        },
        
        // 最后更新时间
        lastUpdated: {
          text: 'Last updated',
          formatOptions: {
            dateStyle: 'short',
            timeStyle: 'medium'
          }
        },
        
        // 文档页脚导航
        docFooter: {
          prev: 'Previous',
          next: 'Next'
        },
        
        // 大纲配置
        outline: {
          level: [2, 3],
          label: 'On this page'
        },
        
        // 返回顶部
        returnToTopLabel: 'Return to top',
        
        // 侧边栏菜单标签
        sidebarMenuLabel: 'Menu',
        
        // 深色模式切换标签
        darkModeSwitchLabel: 'Appearance',
        lightModeSwitchTitle: 'Switch to light theme',
        darkModeSwitchTitle: 'Switch to dark theme'
      }
    }
  },
  
  // Vite 配置
  vite: {
    build: {
      outDir: '../dist/docs'
    }
  }
}))  // withMermaid 闭合括号