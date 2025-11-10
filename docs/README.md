# CCR Documentation

本目录包含 CCR (Claude Code Configuration Switcher) 的完整文档，使用 VitePress 构建。

## 🌍 多语言支持

文档提供多语言版本：

- **🇨🇳 简体中文（默认）** - 根目录下的完整中文文档
- **🇺🇸 English** - `/en` 目录下的完整英文文档

在线浏览文档：[GitHub Repository](https://github.com/bahayonghang/ccr)

## Quick Start

### Using Just (Recommended)

If you have [just](https://github.com/casey/just) installed:

```bash
# 查看所有可用命令
just

# 安装依赖
just install

# 启动开发服务器
just dev

# 构建生产版本
just build

# 预览生产版本
just preview
```

### Using npm

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build

# 预览生产版本
npm run preview
```

## Available Just Commands

| Command | Description |
|---------|-------------|
| `just install` | 安装依赖 |
| `just dev` | 启动开发服务器 |
| `just build` | 构建生产版本 |
| `just preview` | 预览生产构建 |
| `just clean` | 清理构建文件和缓存 |
| `just clean-all` | 完全清理(包括 node_modules) |
| `just reinstall` | 重新安装依赖 |
| `just audit` | 检查安全漏洞 |
| `just audit-fix` | 修复安全漏洞 |
| `just update` | 更新依赖 |
| `just outdated` | 检查过期依赖 |
| `just rebuild` | 快速重建 |
| `just verify` | 验证构建 |
| `just setup` | 开发环境完整设置 |
| `just deploy` | 生产部署准备 |

## 📁 文档结构

```
docs/
├── .vitepress/
│   └── config.mjs          # VitePress 国际化配置
├── guide/                  # 📖 用户指南（中文，默认）
│   ├── quick-start.md          # 快速开始
│   ├── configuration.md        # 配置管理
│   └── web-guide.md            # Web 指南
├── reference/              # 📚 技术参考（中文）
│   ├── architecture.md         # 架构设计
│   ├── changelog.md            # 更新日志
│   ├── commands/               # 命令参考
│   └── platforms/              # 平台支持
├── examples/               # 💡 示例（中文）
│   ├── index.md
│   ├── multi-platform-setup.md
│   └── troubleshooting.md
├── en/                     # 🇺🇸 English Documentation
│   ├── index.md                # English homepage
│   ├── guide/                  # 📖 User Guide
│   │   ├── quick-start.md
│   │   ├── configuration.md
│   │   └── web-guide.md
│   ├── reference/              # 📚 Reference
│   │   ├── architecture.md
│   │   ├── changelog.md
│   │   ├── migration.md
│   │   ├── commands/
│   │   └── platforms/
│   └── examples/               # 💡 Examples
├── index.md                # 中文首页
├── public/
│   └── logo.svg            # 项目 Logo
├── package.json            # Node.js 依赖
└── justfile                # 构建自动化脚本（带依赖自检）
```

## Contributing

When adding new documentation:

1. Create a new `.md` file in the `docs/` directory
2. Update `.vitepress/config.mjs` to include the new page in navigation/sidebar
3. Use VitePress markdown features for enhanced documentation

## VitePress Features

- **Markdown Extensions**: Enhanced markdown with syntax highlighting, code groups, and more
- **Vue Components**: Use Vue components in markdown
- **Search**: Built-in local search functionality
- **Theme Customization**: Customizable default theme
- **Internationalization**: Multi-language support (if needed)

For more information, visit [VitePress Documentation](https://vitepress.dev/).
