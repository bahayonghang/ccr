# CCR Desktop 文档

本小姐为 CCR Tauri 桌面应用打造的专业技术文档！(￣▽￣)／

## 📚 文档内容

- **快速开始**: 安装、配置、基本使用
- **📦 跨平台打包**: Linux/macOS/Windows 智能打包指南 ⭐ 新增
- **架构设计**: 前后端架构、通信机制、安全策略
- **API 参考**: 完整的 Tauri Commands API 文档
- **开发指南**: 项目结构、开发流程、最佳实践
- **配置说明**: Tauri、Vite、权限配置
- **部署指南**: 打包、签名、CI/CD
- **故障排查**: 常见问题和调试技巧

## 🚀 快速开始

### 安装依赖

```bash
cd docs
npm install
```

### 本地预览

```bash
npm run docs:dev
```

文档将在本地开发服务器启动。

### 构建生产版本

```bash
npm run docs:build
```

构建产物位于 `docs/.vitepress/dist/` 目录。

### 预览构建结果

```bash
npm run docs:preview
```

## 📁 目录结构

```
docs/
├── .vitepress/
│   └── config.ts          # VitePress 配置
├── index.md               # 首页
├── guide/                 # 使用指南
│   ├── getting-started.md # 快速开始
│   ├── packaging.md       # 📦 跨平台打包指南 (新增)
│   ├── web-debug-mode.md  # Web 调试模式
│   ├── installation.md
│   ├── development.md
│   └── build.md
├── architecture/          # 架构设计
│   ├── overview.md        # 总体架构
│   ├── frontend.md
│   ├── backend.md
│   ├── communication.md
│   └── security.md
├── api/                   # API 参考
│   ├── commands.md        # Tauri Commands
│   ├── frontend.md
│   └── types.md
├── development/           # 开发指南
│   ├── structure.md
│   ├── add-feature.md
│   ├── debugging.md
│   ├── testing.md
│   └── best-practices.md
├── config/                # 配置说明
│   ├── tauri.md
│   ├── vite.md
│   └── permissions.md
├── deployment/            # 部署指南
│   ├── packaging.md
│   ├── signing.md
│   └── ci-cd.md
├── troubleshooting/       # 故障排查
│   ├── faq.md
│   └── logging.md
├── package.json
└── README.md             # 本文件
```

## ✍️ 编写文档

### Markdown 语法

VitePress 支持扩展的 Markdown 语法：

**代码块：**
````markdown
```typescript
const hello = "world"
```
````

**提示框：**
```markdown
::: tip 提示
这是一个提示
:::

::: warning 警告
这是一个警告
:::

::: danger 危险
这是一个危险提示
:::
```

**代码组：**
````markdown
::: code-group
```bash [npm]
npm install
```

```bash [pnpm]
pnpm install
```
:::
````

### Front Matter

每个页面可以添加 Front Matter 元数据：

```yaml
---
title: 页面标题
description: 页面描述
---
```

### 内部链接

使用相对路径链接到其他文档：

```markdown
[快速开始](/guide/getting-started)
[API 参考](/api/commands)
```

## 🎨 自定义主题

VitePress 支持自定义主题样式：

### 修改颜色

在 `.vitepress/config.ts` 中设置：

```typescript
themeConfig: {
  // 自定义颜色
}
```

### 添加自定义样式

在 `.vitepress/theme/style.css` 中添加自定义 CSS。

## 📦 部署

### GitHub Pages

1. 在 `.vitepress/config.ts` 中设置 `base`:
   ```typescript
   base: '/ccr-tauri/'
   ```

2. 构建文档:
   ```bash
   npm run docs:build
   ```

3. 部署 `docs/.vitepress/dist/` 目录到 GitHub Pages

### Netlify / Vercel

1. 设置构建命令: `npm run docs:build`
2. 设置发布目录: `docs/.vitepress/dist`
3. 自动部署

## 🔍 搜索功能

VitePress 内置了本地搜索功能，无需额外配置。

用户可以按 `⌘/Ctrl + K` 打开搜索框。

## 🤝 贡献

欢迎贡献文档改进！

1. Fork 仓库
2. 创建特性分支
3. 编写或改进文档
4. 提交 Pull Request

### 文档规范

- 使用清晰的标题层级
- 提供代码示例
- 使用提示框强调重要信息
- 保持语言风格一致
- 及时更新过时内容

- 🐛 [问题反馈](https://github.com/harleyqing/ccr/issues)
- 💬 [讨论区](https://github.com/harleyqing/ccr/discussions)

## 📝 待完善的文档

目前已完成核心文档，以下页面待补充：

- [ ] `introduction.md` - 项目介绍
- [ ] `features.md` - 特性列表
- [ ] `tech-stack.md` - 技术栈详解
- [ ] `architecture/frontend.md` - 前端架构详解
- [ ] `architecture/backend.md` - 后端架构详解
- [ ] `architecture/communication.md` - 通信机制
- [ ] `architecture/security.md` - 安全策略
- [ ] `api/frontend.md` - 前端 API
- [ ] `api/types.md` - 类型定义
- [ ] `development/*` - 开发指南系列
- [ ] `config/*` - 配置说明系列
- [ ] `deployment/*` - 部署指南系列
- [ ] `troubleshooting/*` - 故障排查系列

## 📚 参考资源

- [VitePress 官方文档](https://vitepress.dev/)
- [Markdown 语法](https://www.markdownguide.org/)
- [Tauri 文档](https://tauri.app/)
- [Vue 3 文档](https://vuejs.org/)

## 📄 许可证

MIT License © 2024 CCR Team

---

**Made with ❤️ by 哈雷酱**

哼，这可是本小姐精心打造的文档系统呢！(￣▽￣)／
所有核心功能都已经完成了，剩下的细节页面可以慢慢补充～
有问题的话... 才、才不是说本小姐会帮你解决啦！(,,><,,)
