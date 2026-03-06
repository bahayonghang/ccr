# 快速开始

本页只做第一次上手必需的事情：安装、初始化、创建首个 profile，并明确 `ccr ui` 与 `ccr web` 的定位。

## 环境要求
- Rust 1.90+
- 可选：Node.js 18+ 与 Bun 1.0+（仅在开发 `ccr-ui` 时需要）
- 建议：`just`

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

工作区说明：
- 可安装 CLI crate 位于 `crates/ccr`
- `crates/ccr-db` 与 `crates/ccr-types` 提供配套服务和共享类型
- `docs/`、`scripts/`、`examples/` 保持仓库根目录

## 初始化

CCR 默认使用 Unified Mode：

```bash
ccr init
```

初始化后的核心结构：

```text
~/.ccr/
├── config.toml
├── platforms/
│   ├── claude/
│   ├── codex/
│   ├── gemini/
│   ├── qwen/
│   ├── iflow/
│   └── droid/
├── history/
└── backups/
```

如果必须继续使用 Legacy 单文件模式：

```bash
export CCR_LEGACY_MODE=1
ccr init
```

## 创建首个 profile

```bash
ccr platform list
ccr add
ccr list
ccr switch <name>
```

日常最小闭环：

```bash
ccr current
ccr validate
ccr history -l 20
```

## 浏览器与桌面入口

```bash
ccr ui -p 15173 --backend-port 38081
ccr web --host 127.0.0.1 --port 19527 --no-browser
```

- `ccr ui`：推荐浏览器入口，适合日常可视化管理
- `ccr web`：Legacy 轻量 API，适合脚本、CI、兼容场景

## 接下来读什么
- 日常命令组织：[`CLI 工作流`](/guide/cli-workflows)
- 图形界面运行方式：[`UI 概览`](/guide/ui-overview)
- `ccr ui` vs `ccr web`：[`Web 指南`](/guide/web-guide)
- 全量命令：[`命令参考`](/reference/commands/)
