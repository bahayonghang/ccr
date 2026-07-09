# 收拢 ccr facade

## Goal

`dispatch.rs`（748 行，ccr crate 唯一真实实现）迁回 ccr-cli，ccr 收缩为薄 main.rs；删除 ccr 的 4 个死依赖；把两个外部消费方（ccr 二进制、ccr-ui/src-tauri）实际使用的面收进 curated prelude。审查候选 6（Worth exploring）。

## Requirements

### 现状（探索报告定位）

- `crates/ccr`：main.rs(42) + cli/dispatch.rs(748，唯一真逻辑，无直接测试) + lib.rs(315，~90% re-export) + cli/mod.rs(8，纯 re-export)。
- `crates/ccr/Cargo.toml` 4 个死依赖：ccr_config、ccr_skills、ccr_codex、ccr_sync 在 ccr 源码 0 处引用（经 ccr-cli 传递）。
- re-export 墙：`ccr-cli/src/{models,managers,services}/mod.rs` 大量 `#[allow(unused_imports)]` 转发（managers/mod.rs:29-70、services/mod.rs:33-52），`crate::models::Platform` 实为 `ccr_config::Platform`——每个类型 3 条可达路径，定位定义要跨 3 层弹跳；`ccr-tui/src/lib.rs:3` 再 re-export 一层。
- 命令协调逻辑（*_command 如何编排 managers+services+platforms）只被 ccr 层集成测试间接覆盖，locality 弱。

### 要做的

1. dispatch.rs 迁入 ccr-cli（与其路由的命令同 crate），并为路由逻辑补直接测试。
2. ccr 收缩为薄 main.rs + 必要的兼容 re-export 桥（桥保留，见约束）；删除 4 个死依赖。
3. 为两个外部消费方盘点实际 import 面（`rg 'use ccr::' + rg 'use ccr_cli::'` in ccr-ui/src-tauri），把该面收进 ccr-cli 的 curated prelude/明确模块；re-export 墙按盘点结果瘦身——只留真被消费的条目，删除纯"未来可能用"的 `#[allow(unused_imports)]` 行。
4. ccr-tui/src/lib.rs 的纯别名 re-export 一并清理（TUI 内部改直接 import）。

### 约束（重要：spec 冻结）

- **`public-api-boundary.md` 冻结 legacy 根 re-export**：`ccr::application/commands/managers/models/services/sync/sessions` 等路径在 6.x 必须继续可用，不加 `#[deprecated]`。本任务只能做"内部搬家 + 死依赖清理 + 墙瘦身"，**删除 ccr 的兼容桥属于 next-major breaking-change list，超出本任务范围**。
- 根 `pub use`/`pub mod` 变化必须走 `crates/ccr/tests/public_api_compat.rs` 快照有意更新。
- ccr-ui/src-tauri 对 ccr-cli 的依赖不得断裂：每步迁移后 `cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml` 必须通过。

## Acceptance Criteria

- [ ] dispatch 逻辑位于 ccr-cli 且有直接单元/集成测试（路由正确性不再只靠 ccr 层黑盒）。
- [ ] `crates/ccr/Cargo.toml` 无死依赖（cargo-udeps 或人工 rg 验证 0 引用）。
- [ ] re-export 墙中无消费方的条目删除；保留条目均可指出消费方（注释或盘点文档）。
- [ ] `cargo test -p ccr --test public_api_compat` 通过（快照如变化，均为有意且有说明）。
- [ ] `crates/ccr/tests/commands/` 24 个集成测试全绿；`cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml` 通过。
- [ ] `just lint-strict`、`just test` 通过。
- [ ] `public-api-boundary.md` 相应更新；若断定桥接层删除值得排期，在 spec 中登记 breaking-change 候选清单（trellis-update-spec）。

## Notes

- 复杂任务：`task.py start` 前需补 design.md（迁移顺序、prelude 形状、消费面盘点结果）与 implement.md。
- 依赖：建议在 07-03-arch-ccr-error 评估结论之后动手（错误类型归属影响 prelude 形状）；非硬阻塞。
