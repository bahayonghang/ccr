# implement：B 落法执行清单

前置：design.md 已定案（否决 A，B = 冻结 + 守卫 + ADR）。全程在 dev 分支，不动 `ccr-vite-dev-server-usage-handoff.md`（未跟踪，另行处置约定）。

## 步骤

1. [x] 否决式调研（research/ 两文件，2026-07-05）
2. [x] error.rs 冻结注释 + variant 快照单测（穷尽 match `variant_name()` + `FROZEN_VARIANTS` 25 名单）
       → verify: `cargo test -p ccr-core -- --test-threads=1` 69 passed；红绿证据：注释掉 ExternalCommandError 臂 → `error[E0004]: non-exhaustive patterns`（error.rs:447），还原后 error 模块 5 测试绿
3. [x] 修 2 处幽灵注释 `CcrError::ConfigNotFound`（实际 variant 为 ConfigSectionNotFound，ccs_config.rs:42 证实）
       → verify: `rg "ConfigNotFound" crates/` 0 命中
4. [x] ADR：新增 `.trellis/spec/ccr-core/backend/ccr-error-freeze.md` + 登记 index.md（表格行 + pre-dev checklist）
       → verify: index 条目与文件名一致；prettier 格式化后表格转义竖线完好
5. [x] spec 措辞核对与修正：实改 4 处 = 已知 3 处（ccr-sync:27 / ccr-store:31 / ccr-codex:34）+ ccr-core:63（"Add a new variant only when…"直接授权加 variant，与冻结冲突，必须重写）；其余 5 处核对为描述性/仍正确，不动（ccr-config:41 只泛指 CcrError 不涉新 variant、ccr-cli:37、ccr:27、ccr-types:49、atomic-writer:113、ccr-core:7）
       → verify: diff 逐行自查通过
6. [x] 全量验证：`just version-check` ✅ → `just fmt-check` ✅ → `just lint-strict` ✅ → `cargo test -p ccr-core -p ccr-cli -- --test-threads=1` 264 passed ✅ → `cargo test -p ccr --test public_api_compat` 3 passed ✅（快照零变化，符合"无公开面变更"预期）
7. [ ] 提交（按 concern 拆分，Conventional Commits + [AI]）：
   - a) `test(core)`: 冻结注释 + 快照单测 + 幽灵注释修复
   - b) `docs(spec)`: ADR + 措辞修正
   - c) 归档提交：task.py archive + journal
8. [ ] `task.py archive 07-03-arch-ccr-error`，父任务进度 5/8 → 6/8，journal 追加 Session 记录

## 回滚点

- 每步独立提交，`git revert <hash>` 即可；无迁移、无接口变更、无数据风险。

## 复查门

- 快照单测的红绿验证证据链（步骤 2 verify）记入本文件完成态备注。
- spec diff 由主会话逐行自查（改动小，不派 trellis-check；若步骤 2-5 出现意外扩面则升级为独立复核）。
