# CCR CLI facade

[根目录](../../../CLAUDE.md) > **crates/ccr**

## Change Log

- **2026-09-06**: 按当前源码改为 facade 导航；领域逻辑在其他 crate，不再把本包写成单体 `src/` 服务树。
- **2026-01-11**: 补充 cli/, sessions/, storage/, sync/, platforms/, models/ 模块详细描述（当时仍按单体树记录，现已过时）
- **2025-12-17**: 激进精简到 300 行以内，只保留核心架构和技术栈
- **2025-12-16**: 按标准模板重新组织文档结构
- **2025-10-22 00:04:36 CST**: 初始核心模块文档创建

---

## 模块职责

`crates/ccr` 是可安装的 CLI/TUI **入口包**（workspace `default-members`），不是领域实现所在地。

二进制 `src/main.rs` 解析 Clap 参数、初始化日志，并把 TUI 启动器注入 `CommandDispatcher`。命令定义与分发在 `ccr-cli`（本包 `src/cli/mod.rs` 再导出）。TUI 实现在 `ccr-tui`。配置、存储、同步、用量等在对应 crate。

公开 Rust API 通过 `src/lib.rs` 再导出各域 crate，作为 7.x 兼容面；新代码优先 `ccr::prelude` 或直接依赖拥有该逻辑的 crate。不要在本包新增领域服务/管理器。导航以 `crates/code_map.md` 为准，不要按本文件历史单体树去搜 `crates/ccr/src/services/`。

### 入口与委派

| 入口 | 路径 | 实际所有者 |
|------|------|------------|
| CLI 二进制 | `crates/ccr/src/main.rs` | 参数解析、日志、注入 TUI launcher，然后 `CommandDispatcher::dispatch` |
| CLI 再导出 | `crates/ccr/src/cli/mod.rs` | `ccr-cli` 的 `Cli` / `CommandDispatcher` / `build_cli_command` |
| 库入口 | `crates/ccr/src/lib.rs` | 7.x 兼容再导出；新代码走 `prelude` 或域 crate |
| 兼容测试 | `crates/ccr/tests/public_api_compat.rs` | 公开 API 兼容契约 |
| CLI 命令 | `crates/ccr-cli/src/commands/` | 命令实现 |
| TUI | `crates/ccr-tui/` | Ratatui 界面；由二进制注入，避免 `ccr-cli`↔`ccr-tui` 循环依赖 |
| 配置 | `crates/ccr-config`, `crates/ccr-codex` | 平台/profile/Codex |
| 持久化 | `crates/ccr-db`, `crates/ccr-store` | SQLite、会话、定价 |
| 用量投影 | `crates/ccr-usage` | 只读 llmusage SQL；不要在本包写用量查询 |
| 基础设施 | `crates/ccr-core`, `crates/ccr-types` | 锁、原子写、错误类型、跨 crate DTO |

### 本包源码

```
crates/ccr/src/
├── main.rs      # 可安装二进制入口
├── lib.rs       # 库再导出 / 7.x 兼容面
└── cli/mod.rs   # 再导出 ccr-cli
```

集成测试在 `crates/ccr/tests/`（commands / workflows / platforms / managers / public_api_compat）。

---

## 代码风格

- **Edition**: 2024（需要 Rust 1.88+）
- **格式化**: `cargo fmt`
- **检查**: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used`
- **错误处理**: 生产路径不要新增 `unwrap` / `expect`
- **文档**: 内部逻辑中文注释，公开 API 英文
- **测试**: 测试模块可用 `#[allow(clippy::unwrap_used)]`

从仓库根运行 workspace 命令（`Cargo.toml` 与 `justfile` 在根目录），不要假设 cwd 是 `crates/ccr`。

```bash
just build
just test
just lint-strict
just ci
cargo test -p ccr --test commands
cargo test -p ccr --test public_api_compat
```

直接跑 `cargo test` 时带 `-- --test-threads=1`（现有 flake 规避，不是新的串行恢复引擎）。

---

## Git 与文档

分支：`main` / `dev` / `feature/*` / `bugfix/*`。提交用 Conventional Commits。

- 根说明：`/CLAUDE.md`、`/AGENTS.md`
- crate 导航：`crates/code_map.md`、`crates/AGENTS.md`
- 桌面 UI：`ccr-ui/CLAUDE.md`（视觉规则以 `ccr-ui/AGENTS.md` 与 `DESIGN.md` 为准）
