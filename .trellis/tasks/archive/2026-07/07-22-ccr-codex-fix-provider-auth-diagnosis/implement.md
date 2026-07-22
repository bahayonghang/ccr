# Implement - `ccr codex fix` Provider auth diagnosis

> 用户于 2026-07-22 选择推荐方案并明确要求“开始实现”；以下勾选与验证记录反映实际交付结果。

## 1. Planning Gate

- [x] 用户评审收敛后的 PRD/design/implement；实施前任务保持 `planning`。
- [x] 获得明确实施授权后，运行 `task.py start` 进入 Phase 2。

## 2. Add Domain Diagnostic Contract

- [x] 阅读 `.trellis/spec/ccr-codex/backend/backend-guidelines.md` 与 `codex-app-server-cleanup.md`。
- [x] 在 `ccr-codex` 增加 `CodexRuntimeDiagnostic`、match status、issue 类型和 provider-validity 类型；所有公开字段可安全序列化/Debug。
- [x] 增加只读 inspection 入口，先读取 raw registry/profiles pointers，避免调用会清理 pointer 的 `stable_current_profile()`。
- [x] 复用现有 `build_switch_spec` expectation，避免复制 auth-mode、auto-promote、credential-store 规则。
- [x] 实现 route comparison 与 credential comparison；比较值只在内存存在，并检查 `CODEX_API_KEY` / `OPENAI_API_KEY`、实际 runtime 与目标 profile 的 provider `env_key` 存在性。
- [x] 单测覆盖 match/missing/mismatch/not-applicable/unsupported、pointer ambiguity、无当前 profile 和无 secret 泄漏。
- [x] 验证：`cargo test -p ccr-codex runtime_diagnostic -- --test-threads=1`（10 passed）。
- 回滚点：新增 model/inspection 可整体移除，不影响现有 apply path。

## 3. Integrate CLI Rendering

- [x] 在 `crates/ccr-cli/src/commands/codex/fix.rs` 中按 cleanup -> inspect -> doctor 顺序编排。
- [x] 渲染 profile、provider、base URL、wire API、credential store/source 与三层 summary。
- [x] doctor 区块标注本次 snapshot profile；若 inspection 前后 profile/runtime 变化，报告竞态并拒绝宣称一致。
- [x] 增加 local-drift 非零退出码，定义与 2/127 同时出现时的固定优先级。
- [x] 为纯渲染/脱敏函数加入回归测试，并用 sentinel 断言 JSON、文本、URL userinfo/query 无 secret 泄漏。
- [x] 验证：`cargo test -p ccr-cli --lib fix -- --test-threads=1`（10 passed）。

## 4. Add Explicit Runtime Repair

- [x] 在 `CodexAction::Fix` 增加 `repair_runtime: bool` 与帮助文案；裸命令不得进入 runtime 写入分支。
- [x] 仅对 `repairable=true` 的快照调用 `CodexPlatform::apply_profile`；pointer ambiguous、profile/secret missing 和 unsupported store 均拒绝猜测。
- [x] `--dry-run --repair-runtime` 只展示动作，不发送信号、不写 runtime 或 doctor 临时报告。
- [x] 修复后重新 inspection；只有 route + credential 均 Match 才报告修复成功。
- [x] 单元/集成测试覆盖 auth missing/mismatch、route mismatch、修复成功和 dry-run 受管文件 byte-for-byte 不变；修复继续委托既有 `CodexRuntimeCommitPlan` 原子写/回滚路径，没有新增直接写路径。
- [x] 验证：`cargo test -p ccr --test commands codex_fix -- --test-threads=1`（2 passed，含 help）。
- 回滚点：flag/repair branch 可独立撤回，诊断仍保留。

## 5. Documentation

- [x] 更新命令/platform Codex 文档与英文镜像，说明先切换目标 profile 再运行 fix。
- [x] 明确 file auth 与环境变量的区别，以及 local match 不等于 Provider key valid。
- [x] 记录 `--repair-runtime`、dry-run 组合和新退出码。
- [x] 给出 `INVALID_API_KEY` 处置结论，不要求用户打印或粘贴 `auth.json`。

## 6. Validation And Review

- [x] `just fmt-check`
- [x] `cargo test -p ccr-codex -- --test-threads=1`（223 passed，3 ignored）
- [x] `cargo test -p ccr-cli --lib -- --test-threads=1`（204 passed，1 ignored）
- [x] `cargo test -p ccr-cli --lib fix -- --test-threads=1`（10 passed）
- [x] `cargo test -p ccr --test commands -- --test-threads=1`（55 passed）
- [x] `just lint-strict`
- [x] `just docs-check`
- [x] 已评估 full gate：`just version-check` 的版本同步通过，但随后因任务外既有 `ccr-ui/README.md` 仍写 `version-6.5.1` 而失败；该文件无本任务 diff，因此未运行会在同一点失败的 `just ci`。
- [x] 最终 diff 只涉及 ccr-codex、ccr-cli、对应 tests/docs/spec 与本任务文件，不夹带其他工作树改动。
- [x] 人工安全审查：检查 sentinel、`OPENAI_API_KEY` / `CODEX_API_KEY` 输出、Debug/JSON、doctor 文本与临时报告路径，未发现 secret 值、片段或 fingerprint 输出。
