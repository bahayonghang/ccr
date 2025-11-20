# CCR - Claude Code Configuration Switcher

Rust 编写的多平台配置管理工具（Claude、Codex/GitHub Copilot、Gemini CLI、Qwen、讯飞等），版本 3.4.1。以 CLI 为核心，可选 TUI、Web API，以及基于 Vue 3 + Axum + Tauri 的 CCR UI。

## 功能亮点
- 设置安全写入：原子写入、文件锁、审计日志与自动备份，直接管理 `settings.json` 与配置文件。
- 统一多平台模式：默认使用 `~/.ccr/`（按平台分别存放 profiles/history/backups），兼容传统 `~/.ccs_config.toml`。
- 配置全生命周期：`init`、`add`、`list/current/switch`、`enable/disable`、`validate`、`history`、`optimize`，支持 `--yes` 跳过确认。
- 导入/导出/清理：导出可选择去除敏感信息，导入支持合并/替换并自动备份，清理旧备份。
- WebDAV 多目录同步（`web` 特性）：目录注册、启用/禁用、单目录或全量 push/pull/status，支持交互式选择。
- 平台管理：`ccr platform list/switch/current/info/init`，可输出 JSON 便于脚本。
- 临时凭据与更新：`ccr temp-token set/show/clear`，`ccr update --check` 检查或更新版本。
- 可观测性：完整历史记录、JSON 输出选项、成本统计命令（`ccr stats ...`，`web` 特性）。
- 多种界面：CLI 通用，TUI（`--features tui`），兼容型 `ccr web` 轻量 API，现代 CCR UI（`ccr ui`，Vue 3 + Axum 后端，可打包 Tauri 桌面版）。

## 安装
依赖：Rust 1.85+、Cargo。开发 CCR UI 需要 Node.js 18+（npm），建议安装 `just` 以使用快捷命令。

一行安装：
```bash
cargo install --git https://github.com/bahayonghang/ccr ccr
```

源码安装：
```bash
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

构建选项：
- `cargo build --no-default-features`（仅 CLI，最快）
- `cargo build --features web`（CLI + Web API + 同步 + UI 入口）
- `cargo build --features tui`（CLI + TUI）
- `cargo build --all-features`
- 工作区构建/测试：`cargo build --workspace`，`cargo test --workspace`，`cargo clippy --workspace --all-targets --all-features -- -D warnings`

## 工作区结构
```
ccr/
|-- src/                # CLI + 库（platforms、services、sync、web、tui）
|-- ccr-ui/
|   |-- backend/        # Axum 服务（workspace 成员）
|   `-- frontend/       # Vue 3 + Vite + Pinia + Tauri
|-- tests/              # 集成测试
|-- docs/               # 文档站
|-- examples/           # 示例与演示
`-- justfile            # 常用开发任务
```

## CLI 快速上手
1) 初始化统一模式：
```bash
ccr init
```
生成 `~/.ccr/config.toml` 以及 `~/.ccr/platforms/` 下的各平台目录。若要继续使用传统单文件模式，在运行前设置 `CCR_LEGACY_MODE=1`。

2) 查看并切换平台：
```bash
ccr platform list
ccr platform switch claude   # 或 codex/gemini/qwen/iflow
```

3) 创建与管理配置：
```bash
ccr add                      # 交互式创建
ccr list
ccr current
ccr switch <name>            # 也可直接 ccr <name>
ccr enable <name> | ccr disable <name> [--force]
ccr validate
ccr history -l 50
ccr optimize
```

4) 导入 / 导出 / 清理：
```bash
ccr export -o configs.toml --no-secrets
ccr import configs.toml --merge --backup
ccr clean --days 30 --dry-run
```

5) WebDAV 多目录同步（`web` 特性）：
```bash
ccr sync config                    # 配置 WebDAV 端点
ccr sync folder add claude ~/.claude -r /ccr-sync/claude
ccr sync folder enable claude
ccr sync all status
ccr sync all push --force
ccr sync claude pull               # external_subcommand 形式
```

6) 界面与服务：
```bash
ccr ui -p 3000 --backend-port 8081   # 完整 UI（Vue 3 + Axum），优先检测 workspace -> ~/.ccr/ccr-ui -> GitHub 下载
ccr tui                              # TUI（需要启用 tui 特性）
ccr web -p 8080                      # 兼容型轻量 API server
```

其他常用命令：
- `ccr temp-token set sk-xxx [--base-url ... --model ...]`
- `ccr update --check`
- `ccr stats cost --today`（`web` 特性）
- `ccr version`

## CCR UI（Vue 3 + Axum + Tauri）
功能：
- 可视化管理、验证、历史与备份。
- 覆盖全部 CLI 命令的可视化执行，实时输出。
- WebDAV 多目录同步面板（新增/启用/禁用/推送/拉取/状态/批量）。
- 系统信息、平台概览、健康检查。
- 同时支持 Web 模式（HTTP API）与桌面模式（Tauri invoke）。

通过 CLI 启动（推荐）：
```bash
ccr ui                     # 自动检测 workspace -> ~/.ccr/ccr-ui -> GitHub 下载
# 默认端口：前端 3000，后端 8081，可用 -p/--backend-port 覆盖
```

从仓库开发：
```bash
cd ccr-ui
just s                     # 前后端开发模式一键启动
just quick-start           # 首次使用：依赖检查 + 安装 + 启动
```

手动开发（显式命令）：
```bash
# 后端（workspace 成员）
cd ccr-ui/backend
cargo run -- --port 8081

# 前端
cd ../frontend
npm install
npm run dev                # 默认 http://localhost:5173
```

生产构建：
```bash
cd ccr-ui
just build                 # 构建后端 + 前端
just run-prod              # 启动编译后的后端并服务前端产物
```

桌面版（Tauri）：
```bash
cd ccr-ui
just tauri-dev             # 桌面开发窗口
just tauri-build           # 打包桌面安装包
```

## 开发流程
- 快速检查：`cargo check`，`cargo fmt --all --check`，`cargo clippy --workspace --all-targets --all-features -- -D warnings`
- 测试：`cargo test --workspace`，`cargo test --test platform_integration_tests -- --test-threads=1`
- just 辅助：`just dev`，`just watch`，`just ci`，`just build`，`just release`
- 同步相关操作需开启 `web` 特性。

## 故障排查
- 开启日志：`export CCR_LOG_LEVEL=debug`（还可用 trace/info/warn/error）后重试。
- 仍用传统配置：设置 `CCR_LEGACY_MODE=1` 后再 `ccr init`。
- 同步冲突：根据提示使用 `--force` 或 `--interactive`。
- 权限问题：确保配置与 settings 文件对当前用户可写。

## 许可证
MIT
