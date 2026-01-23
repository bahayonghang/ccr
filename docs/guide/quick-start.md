# 快速开始

面向 CCR v3.4.1 的安装、初始化与日常使用指引。

## 环境要求
- Rust 1.85+（含 Cargo）
- 可选：Node.js 18+（仅当开发 CCR UI 前端时）
- 建议：`just` 任务工具（`cargo install just`）

## 安装
```bash
# 推荐：直接安装
cargo install --git https://github.com/bahayonghang/ccr ccr

# 或源码安装
git clone https://github.com/bahayonghang/ccr.git
cd ccr
cargo install --path .
```

构建特性：`--no-default-features`（仅 CLI）、`--features web`（API/同步/UI 入口）、`--features tui`（TUI）、`--all-features`。

## 初始化与模式
默认使用 Unified Mode（多平台）：
```bash
ccr init
```
生成：
```
~/.ccr/
|-- config.toml          # 平台注册
|-- platforms/
|   |-- claude/  # profiles/history/backups
|   |-- codex/
|   |-- gemini/
|   |-- qwen/
|   `-- iflow/
|-- history/             # 全局历史
`-- backups/             # 全局备份
```

需继续使用单文件模式时，在运行前设置 `CCR_LEGACY_MODE=1`，使用 `~/.ccs_config.toml`。

## 常用操作
```bash
ccr platform list                    # 查看平台
ccr add                              # 引导创建配置
ccr list && ccr switch <name>        # 查看/切换；可直接 ccr <name>
ccr enable <name> | ccr disable <name> [--force]
ccr validate                         # 校验配置与 settings
ccr history -l 50                    # 查看历史
ccr optimize                         # 排序与清理
```

导入/导出与备份：
```bash
ccr export -o configs.toml --no-secrets
ccr import configs.toml --merge --backup
ccr clean --days 30 --dry-run
```

## WebDAV 多目录同步（web 特性）
```bash
ccr sync config                           # 配置 WebDAV
ccr sync folder add claude ~/.claude -r /ccr-sync/claude
ccr sync folder enable claude
ccr sync claude push                      # 单目录
ccr sync all status
ccr sync all pull --force                 # 批量
```

## 界面与服务
```bash
ccr ui -p 3000 --backend-port 8081   # 完整 CCR UI（Vue 3 + Axum + Tauri）
ccr tui                              # 需开启 tui 特性
ccr web --host 0.0.0.0 -p 19527 --no-browser    # 轻量 API/兼容场景
```

## 日常调试
- 日志级别：`export CCR_LOG_LEVEL=debug`（trace/debug/info/warn/error）
- 日志文件：`~/.ccr/logs/ccr.YYYY-MM-DD.log`（按天轮转，保留14天）
- 检查：`cargo fmt --all --check`，`cargo clippy --workspace --all-targets --all-features -- -D warnings`
- 测试：`cargo test --workspace`

## 目录与配置提示
- Unified Mode 配置在 `~/.ccr/`，平台 profiles/history/backups 分目录存放。
- Legacy 模式为单文件 `~/.ccs_config.toml`，与 CCS 兼容。
- CLI 与 CCR UI 共用同一配置与日志/备份体系。
