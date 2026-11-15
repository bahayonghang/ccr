# 快速开始

本页覆盖第一次上手 CCR 所需的最短路径：安装、初始化平台配置库、安全创建首个 Profile、在平台间切换运行时，并理解 `ccr` / `ccr ui` / `ccr-ui` 的关系。

## 环境要求

- Rust 1.90+
- 可选：Node.js 18+ 与 Bun 1.0+（仅在本地开发 `ccr-ui` 时需要）
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

## 初始化平台配置库

CCR 7.x 按平台将 Profile 存储在 SQLite 配置库 `~/.ccr/platforms/<platform>/profiles.db` 中。按需初始化你使用的平台模板：

```bash
ccr claude profile init
ccr codex profile init
ccr grok profile init
```

> 若要在当前代码仓库初始化 `mini-trellis` 或 `trellis` 项目工作流，请运行 `ccr project init`。

初始化后，先确认当前运行时总览与平台注册表：

```bash
ccr current
ccr platform list
```

## 创建并切换第一个 Claude Profile

请始终通过标准输入（`--auth-token-stdin`）传入密钥，避免明文 Token 进入进程参数或 Shell 历史；也可通过 `profile open` 在 `$VISUAL` / `$EDITOR` 中编辑配置库投影：

```bash
printf '%s' "$ANTHROPIC_AUTH_TOKEN" | ccr claude profile create work \
  --base-url https://api.example.com \
  --auth-token-stdin

ccr claude profile list
ccr claude profile switch work
ccr claude profile current
ccr claude profile off
```

## 管理 Codex 与 Grok Profile

```bash
# Codex
ccr codex auth current
ccr codex profile list
ccr codex profile switch <name>
ccr codex profile off

# Grok（支持 --env-key 绑定环境变量名或 --api-key-stdin 传入密钥）
ccr grok profile create xai --env-key XAI_API_KEY
ccr grok profile list
ccr grok profile switch xai
ccr grok profile off
```

## 最小日常闭环

```bash
ccr current
ccr validate
ccr doctor
ccr history --limit 20
```

## 图形入口

```bash
ccr ui -p 15173 --backend-port 38081
```

- `ccr`：主 CLI / 终端 TUI 入口
- `ccr ui`：图形入口（默认前端端口 `15173`，后端端口 `38081`）
- `ccr-ui`：前端开发与 Tauri 运行工程目录

## 升级与迁移提示

CCR 7.x 已统一收敛至平台级显式命令（`ccr <claude|codex|grok> profile <动作>`）与 `profiles.db` SQLite 配置库，不再使用旧的隐式全局平台切换或顶层单文件子命令。完整旧命令与存储迁移对照请参阅 [`迁移指南`](/reference/migration)。

## 接下来读什么

- [`CLI 工作流`](/guide/cli-workflows)
- [`配置模型`](/guide/configuration)
- [`入口选择`](/guide/entrypoints)
- [`命令参考`](/reference/commands/)
