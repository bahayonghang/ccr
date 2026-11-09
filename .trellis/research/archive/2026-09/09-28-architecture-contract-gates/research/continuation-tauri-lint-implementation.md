# T10 后续测试 lint 与验收缺口实施记录

日期：2026-09-28。执行 owner：`/root/implement_t09`。基线 commit：`34d8a85e0e48b793733835e0304c8ed33940fcee`。本记录只覆盖 root 分配的后续切片，不改变父任务或子任务状态，也不替代完整 CI 和非作者复核。

## 来源与原始证据

1. T09 `check-report.md` 的未修复项及 `research/backend-clippy.log`：Tauri all-targets Clippy 有 4 项既有测试诊断。原日志中的位置分别为 `main.rs:428`、`state.rs:815`、`commands/claude.rs:725`、`commands/codex.rs:2282`。原日志保留，未以新结果覆写。
2. root 的独立验收复核：T08 AC1 需要 model-only patch 后的磁盘 reread；既有读取和内存 merge 测试不能单独证明完整持久化序列。
3. root 的独立验收复核：Claude Settings 的 subtitle 引用不存在；`codex_process_service.rs` 两个测试导入只在 Windows 使用，旧 Linux 检查记录报 unused imports。
4. root 的真实浏览器合成状态观察：Codex Settings 显示 `codex.settings.save` 与 4 个无命名空间的推理档位键。来源为 `continuation-web-fixture-v2.log`。

开始前复用 `trellis-before-dev` 上下文，并使用 `ccr-gate-recovery` 的窄检查流程。源码和文档读取均针对当前任务与指定路径。

## 修复机制与行为边界

### 四项 Tauri 测试 lint

- `src-tauri/src/main.rs`、`state.rs`：将原测试模块整体移至所有生产 item 之后。模块内容与生产函数内容保持不变。
- `src-tauri/src/commands/claude.rs`：将 `assert_eq!(predicate, true)` 改为 `assert!(predicate)`，保持同一谓词。
- `src-tauri/src/commands/codex.rs`：把受影响测试改为同步 `#[test]`，在显式 current-thread Tokio runtime 的 `block_on` 中执行原异步体。全局环境 mutex 仍在外层持有，覆盖设置环境、异步完成、恢复环境和结果断言；未提前释放。未新增全局串行、延迟或 lint allow。

### Codex 实际持久化回归

新增私有 `update_codex_settings_at_path`，仅提取 `codex_update_settings` 原来的连续 `read → apply → write → message` 序列。生产 handler 仍在原 `spawn_blocking` 中解析原路径，保持参数转换、错误映射、返回值及 cache invalidation 时序。

新增 `commands::codex::settings::tests::model_only_update_preserves_notifications_and_untouched_fields_on_disk`。测试将合成 TOML 写入 `TempDir`，调用同一个生产持久化函数，再从磁盘读取；断言 model 实际变化，notifications 数组的内容与顺序不变，并将完整 TOML 与只更改 model 的期望值比较。fixture 包含 TUI 未知字段、顶层未知数组、history、profiles 和 MCP 扩展字段。未删除原有读取和 merge 测试。

Windows 的 `codex_config_path` 和 AppState preferences 使用 `dirs::home_dir`，其 Windows 实现读取 `FOLDERID_Profile`，不能通过本任务的 HOME/CCR_CODEX_DIR fixture 安全隔离。因此经 root 明确选择私有持久化 helper 方案，未构造真实 AppState、未调用真实 desktop startup、未访问用户 home，也未使用 unsafe 构造 State。未添加 `tauri/test` dev feature；未编辑 manifest 或 lockfile。

该回归证明真实后端磁盘 roundtrip；不覆盖 State 构造、dispatcher、cache invalidation 或原生 WebView E2E。前端 payload 和 dirty-field 契约仍由 T08/T09 的独立回归证明。

### 独立复核确认的两项小修复

- 两种 locale 只补 `claudeSettings.subtitle`。已有 Claude home 描述的英文限定 local sessions，与 typed Settings 的环境能力不完全相同，因此未复用该文案。没有改变 descriptor、表单或环境策略。
- `crates/ccr-codex/src/services/codex_process_service.rs` 的 `SysinfoProcessBackend`、`process_refresh_kind` 测试导入改为 `#[cfg(windows)]`；其全部测试使用点本来就在 Windows 专用测试内。生产实现、测试体和断言保持不变。Linux 编译验收由 root 协调，本 owner 未运行 WSL。

### 浏览器确认的 Codex 文案键

- 按既有 Settings prefix 契约补 `codex.settings.save`。
- 4 个档位 metadata 引用改为完整 `codex.settings.model.reasoningEffortOptions.*`，并添加两语文案；没有增加全局 `low` 等含义不明确的键。未修改 option value 或未知合法值的显示逻辑。
- 新增真实双语 translator 和实际 `CodexSettingsView` 的渲染回归，在同一挂载实例中检查保存按钮、4 个已知 option 和未知 `future-effort`；locale 切换只读取一次 settings。该测试先实际失败，5 个 label 全部返回原键，修复后通过。
- 两种 locale 的最终叶子 key 数量为 4523，原 i18n 数量断言同步。

## 本 owner 执行的验证

| 检查 | 结果 | 日志 |
| --- | --- | --- |
| Tauri all-targets strict Clippy | exit 0，40.69 秒 | `continuation-tauri-all-targets-clippy.log` |
| `commands::codex::` 行为测试 | 56/56，含新磁盘回归 | `continuation-codex-tests.log` |
| `state::tests::` | 3/3 | `continuation-state-tests.log` |
| `tests::close_action` | 2/2 | `continuation-close-action-tests.log` |
| Claude 受影响读取测试 | 1/1 | `continuation-claude-read-tests.log` |
| Codex 双语文案原反例 | 1 failed，证实 5 个原键泄漏 | `continuation-settings-i18n-red.log` |
| Settings 相关 smoke | 4 文件，45/45 | `continuation-settings-smoke.log` |
| i18n 与 key-leak self-test | 24/24，4523 keys | `continuation-i18n-final.log` |
| 前端 type-check | exit 0 | `continuation-type-check.log` |
| 本切片范围 ESLint | exit 0 | `continuation-scoped-eslint.log` |
| 6 个 Rust 文件的只读 rustfmt | exit 0 | `continuation-rustfmt-final.log` |
| 范围内 `git diff --check` | exit 0，保留行尾提示 | `continuation-diff-check.log` |

精确 Cargo 命令、时长和匹配通过数量见 `continuation-tauri-tests.json`。Clippy 命令为：

```text
cargo --config .cargo/tauri-ci.toml clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --all-targets -- -D warnings
```

Cargo 测试保留默认并行执行，使用 `--skip export_bindings`；未运行 bindings 生成或完整 CI。原始 rustfmt 中间诊断和首次 i18n 4518-key 结果保留，最终结果以表格中的 final 文件为准。

范围 ESLint 不代表正式全量 frontend lint 已通过。本 owner 未修改、移动或忽略两份 `.tmp` 脚本，未重新定义原有 lint 失败；其后续授权与修复由 root 独立处理。

## 文件、冻结与交接

本切片修改 11 个源码或测试文件。完整路径、修改前后 SHA256、原始日志 hash 和逐项验证见 `continuation-tauri-lint-implementation.json`。前置证据为 `continuation-tauri-lint-before.json`、`continuation-review-before.json`、`continuation-persistence-before.json`、`continuation-settings-labels-before.json`。hash 覆盖共享工作区的完整文件，不表示其他 owner 的既有改动属于本切片。每项执行开始时未单独采集 hash；本次 after hash 在执行后采集，依据冻结后的无改动记录及 root 冻结文件一致性关联执行版本，标记为 post_run_confirmed_unchanged。

Rust 和前端均已冻结，本 owner 无活动测试或构建进程。已通知 root 释放 Cargo 窗口并重新执行浏览器观察。非作者复核、Linux 检查、完整 aggregate 和原生验收仍由 root 协调。未修改任务状态、父汇总或历史证据；未提交、推送或归档。
