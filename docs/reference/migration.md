# 仓库布局迁移指南

本文用于说明本次 workspace 重构后，旧路径/旧命令与新布局之间的映射关系。

## 路径映射

| 旧路径 | 新路径 | 说明 |
|---|---|---|
| `src/` | `crates/ccr/src/` | 主 CLI、TUI、Web API 与服务层 |
| `tests/` | `crates/ccr/tests/` | CLI 集成测试 |
| `build.rs` | `crates/ccr/build.rs` | 主包构建脚本 |
| `ccr-db/` | `crates/ccr-db/` | 数据库 crate |
| `ccr-types/` | `crates/ccr-types/` | 共享类型 crate |
| `ccr-ui/backend/` | `ccr-ui/src-tauri/` | Tauri 桌面壳 |
| `ccr-ui/frontend/` | `ccr-ui/src/` | Vue 前端源码 |
| 无统一归档目录 | `outputs/` | 仅用于归档最终产物，不改变原生输出位置 |

## 命令映射

| 旧命令 | 新命令 |
|---|---|
| `cargo install --path .` | `cargo install --path crates/ccr` |
| `cargo run -- ...` | `cargo run -p ccr -- ...` |
| `cargo build --release` | `cargo build -p ccr --release` |
| `cd ccr-ui/backend && cargo build --release` | `cd ccr-ui/src-tauri && cargo build --release` |
| `cd ccr-ui/frontend && bun run build` | `cd ccr-ui && bun run build` |

## 构建产物

- CLI 原生产物仍输出到 `target/`。
- UI 前端静态资源仍输出到 `ccr-ui/dist/`。
- Tauri/桌面壳产物仍输出到 `ccr-ui/src-tauri/target/`。
- 使用根目录 `just outputs-collect` 将最终产物汇总到 `outputs/`。
