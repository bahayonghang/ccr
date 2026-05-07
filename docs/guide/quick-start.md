# 快速开始

本页只覆盖第一次上手所需的最短路径：安装、初始化、创建首个 profile，并理解 `ccr` / `ccr ui` / `ccr-ui` 的关系。

## 环境要求
- Rust 1.90+
- 可选：Node.js 18+ 与 Bun 1.0+（仅在开发 `ccr-ui` 时需要）
- 推荐：`just`

## 安装

### 直接安装

```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

### 源码安装

```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path crates/ccr
```

## 初始化 CCR

```bash
ccr init
```

初始化后，先确认当前 runtime 总览：

```bash
ccr current
ccr platform list
```

## 创建并切换第一个 Claude Profile

```bash
ccr add
ccr claude profile list
ccr claude profile switch <name>
ccr claude profile current
ccr claude profile off
```

## 切换第一个 Codex Profile

```bash
ccr codex auth current
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile current
ccr codex profile off
```

## 最小日常闭环

```bash
ccr current
ccr validate
ccr history -l 20
```

## 图形入口

```bash
ccr ui -p 15173 --backend-port 38081
```

- `ccr`：主 CLI / TUI 入口
- `ccr ui`：图形入口
- `ccr-ui`：前端开发与 Tauri 运行工程目录

## 迁移提醒

以下旧命令已经退休：

- `ccr switch <name>`
- `ccr <name>`
- `ccr platform switch <platform>`
- `ccr platform current`

当前推荐路径：

- `ccr claude profile switch <name>`
- `ccr codex profile switch <name>`
- `ccr current`

## 接下来读什么

- [`CLI 工作流`](/guide/cli-workflows)
- [`配置模型`](/guide/configuration)
- [`入口选择`](/guide/entrypoints)
- [`命令参考`](/reference/commands/)
