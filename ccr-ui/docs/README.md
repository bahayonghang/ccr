# CCR UI Docs

CCR UI 的多语言文档（默认中文，含英文），基于 VitePress。

## 目录结构
```
docs/
|-- .vitepress/          # VitePress 配置
|   `-- config.ts        # 多语言导航/侧边栏
|-- guide/               # 中文用户指南
|   |-- getting-started.md
|   |-- project-structure.md
|   |-- stats.md
|   `-- tauri.md
|-- reference/           # 中文技术参考
|   |-- frontend/
|   |   |-- overview.md
|   |   |-- tech-stack.md
|   |   |-- development.md
|   |   |-- components.md
|   |   |-- api.md
|   |   |-- styling.md
|   |   `-- testing.md
|   `-- backend/
|       |-- architecture.md
|       |-- tech-stack.md
|       |-- development.md
|       |-- api.md
|       |-- error-handling.md
|       `-- deployment.md
|-- contributing.md      # 贡献指南
|-- faq.md               # 常见问题
|-- en/                  # 英文文档
|   |-- index.md
|   |-- guide/
|   |-- reference/
|   |   |-- frontend/
|   |   `-- backend/
|   |-- contributing.md
|   `-- faq.md
|-- public/              # 静态资源
|   |-- logo.svg
|   `-- favicon.ico
|-- index.md             # 中文首页
|-- package.json         # 文档依赖
`-- justfile             # 文档站任务
```

## 开发
```bash
npm install             # 安装依赖
npm run docs:dev        # 本地预览 http://localhost:5174
npm run docs:build      # 生成 .vitepress/dist/
npm run docs:preview    # 预览构建产物
```

## 新增/修改页面
- 中文页：放在根级 `guide/` 或 `reference/`，在 `.vitepress/config.ts` 的 zh 导航/侧栏注册。
- 英文页：放在 `en/` 下，对应更新 en 导航/侧栏。
- 内链用绝对路径，如 `[快速开始](/guide/getting-started)`、`[Getting Started](/en/guide/getting-started)`。
- 外链示例：`[GitHub](https://github.com/bahayonghang/ccr)`.

## VitePress 片段示例
Frontmatter:
```yaml
---
title: Page Title
description: Page description
---
```
提示/警告：
```markdown
::: tip
提示
:::

::: warning
注意
:::
```
代码块：
````markdown
```rust
fn main() {}
```
````

## 贡献流程
1. 编辑或新增页面，语言与现有文档保持一致。
2. 更新 `.vitepress/config.ts` 导航/侧栏。
3. 本地预览：`npm run docs:dev`。
4. 提交 PR 到 https://github.com/bahayonghang/ccr。
