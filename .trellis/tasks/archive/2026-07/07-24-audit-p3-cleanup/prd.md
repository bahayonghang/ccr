# P3 轻量清理

> 父任务：`07-24-audit-remediation` ｜ 覆盖：P3-01、P3-02、P3-05 ｜ 报告 §1.3

## Goal

处理不抢占 P1/P2 资源的长期可维护性问题：facade 治理、超大文件拆分规划、编码/格式化门禁。

## 背景 / 证据（已核实）

- `crates/ccr/src/lib.rs` — 兼容 facade 大量 root-level re-export，文档承认用于兼容（P3-01）
- 超大权威文件：`command_exec.rs`（~1700 行）、`migrations.rs`（~1800 行）、`codex_auth.rs`、`sync.rs`，混合 policy/IO/DTO/serialization/OS 分支（P3-02）
- `crates/ccr-db/src/database/migrations.rs:562-654` — 注释 mojibake（GBK 乱码，如"鍥炲～鐜版湁璁板綍"）；`tauri.conf.json` 关键 JSON 压成单行（P3-05）

## Requirements

- [x] facade：标记 deprecated path；新代码 lint 禁止依赖 umbrella crate；按 major version 清理路线（P3-01，持续项）
- [x] 超大文件：按 policy/adapter/runtime/store/types 拆分规划（不做纯机械切文件）；本任务先出拆分方案，实际拆分可随对应 gateway 整改进行（P3-02）
- [x] 修复 `migrations.rs` 注释 mojibake 为正确 UTF-8 中文（P3-05）
- [x] 加 Prettier/JSON format check 门禁；避免在 generated 以外单行压缩 JSON（P3-05，与 ci-governance 协调）

## Acceptance Criteria

- [x] `migrations.rs` 注释乱码清零，`just fmt-check` 通过
- [x] facade deprecation 标记落地，新代码 lint 规则生效
- [x] 超大文件拆分方案文档化（design/note）
- [x] JSON format check 纳入门禁

## Out of Scope

- 不在同一 7.x 周期直接删除全部兼容 facade
- 不按行数机械拆分模块或重写无关业务逻辑
- 不格式化 lockfile、generated binding、第三方资产或空白有语义的 fixture

## Notes

- 优先级最低，收尾阶段处理；注释乱码修复是纯 P3-05 quick fix，可随手先做
- 拆分是"按职责"而非"按行数"，避免制造无意义 module
- 内部实现注释保持中文（CLAUDE.md 规则）

## Verification Evidence (2026-07-27)

- `cargo test -p ccr --test public_api_compat -- --nocapture`: 3/3；legacy
  paths、stable prelude 与 root surface snapshot 均通过。
- `cargo test -p ccr --doc`: 9 个普通 doctest + 1 个 deprecated
  compile-fail doctest 通过。
- `python -m unittest scripts/test_check_dependency_drift.py
  scripts/test_check_json_format.py`: 7/7；真实 dependency/JSON validators
  通过。
- `just fmt-check`、`cargo test -p ccr -- --test-threads=1`、
  `cargo test -p ccr-db migration -- --test-threads=1`、`just lint-strict`、
  `just test` 全部通过；migration focused 16/16。
- mojibake marker 搜索为 0；11 个 JSON inventory 均 canonical。相对 HEAD
  仅 `ccr-ui/package.json`、`ccr-ui/src-tauri/tauri.conf.json`、
  `ccr-vscode/package.json` 含并行 `7.0.0` 语义变化，提交时必须保留在
  unstaged worktree，只提交 HEAD 语义的 canonical JSON。
- `just version-check` 的 version-sync 部分通过，随后被并行版本链阻塞：
  `ccr-ui/README.md` 缺少 `version-7.0.0`。该 README 不属于 P3，也不得为
  通过本子任务而吸收并行版本元数据。
