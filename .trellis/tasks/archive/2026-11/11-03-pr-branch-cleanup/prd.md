# 清理全部 PR 与多余分支

## Goal

把 24 个打开的 PR 尽量合并进它们各自的目标分支，修掉阻断合并的检查失败，并把 Dependabot 从每周改为每月一次。处理完成后，远端长期分支只留 `main` 和 `dev`。

## Background

本次规划时，`bahayonghang/ccr` 有 24 个打开的 PR，全部 `MERGEABLE` 且 `BLOCKED`。`#114` 由仓库所有者打开，`ci/faster-checks` → `dev`。`#90`–`#113`（没有 `#94`）由 Dependabot 打开，目标都是 `main`。抽样失败多数来自目标分支上已经存在的检查，不是升级 diff 本身。

`origin/main` 的 workspace members 没有 `crates/ccr-app`。`origin/dev` 有。因此文档审计里的 “crate map missing workspace member: crates/ccr-app” 属于 `dev` / `#114`，不能当成 `main` 上的同一缺陷。

远端除 `main` 和 `dev` 外还有 23 个 `dependabot/**`、`ci/faster-checks`，以及没有打开 PR 的 `audit-remediation-hosted-20260727`。本地分支是 `ci/faster-checks`、`dev`、`main`。当前检出的是 `ci/faster-checks`（`3a44b53`），工作区有大量未提交的桌面端、GPUI 和 Trellis 改动。

`.github/dependabot.yml` 有 6 条更新，`interval` 全是 `weekly`。`scripts/ci/check_workflow_governance.py` 的 `dependabot_failures()` 只检查生态、目录、锁文件和重复映射，不锁定 `weekly`。GitHub 接受 `monthly`。Dependabot 读取默认分支 `main`。

## Requirements

1. 尽量合并全部 24 个 PR。Dependabot 升级按正常升级处理：保留 PR 里的锁文件 bump，只补编译、类型、治理 SHA 或调用点需要的最小修复。
2. 先修目标分支上阻断 required check 的失败，再合并。已经在 `#114` 上确认、实施时必须先在干净的 `origin/dev` 或 `origin/main` 上复现的失败：
   - Clippy `clippy::needless_borrows_for_generic_args`：`crates/ccr-cli/src/commands/lifecycle/clean_storage.rs:766` 和 `:769` 的 `symlink(&outside_file, &linked_file.join(...))`。
   - 文档审计：`docs/scripts/audit-docs.mjs:278` 要求中英文 crate map 包含 workspace member。`dev` 缺 `crates/ccr-app`（`docs/reference/internals/crate-map.md` 与 `docs/en/reference/internals/crate-map.md`）。
   - `cargo audit` 仍报告建议，包括 `rustls-webpki` 0.103.14 `RUSTSEC-2026-0382`。按依赖治理规格，在现有范围内升级到已修补版本，不忽略建议，不删掉 `cargo audit`。
   - macOS 与 Windows 的 `concurrent_grok_profile_init_is_lossless_and_idempotent`（`crates/ccr/tests/commands/grok_profile.rs:241`）经 `assert_success`（`:103`）失败，子进程退出码 90。
   - macOS 的 `clean_preview_fields_suggestion_and_empty_result_are_neutral`（`crates/ccr/tests/commands/output_presentation.rs:564`）缺少 `扫描目录`。
   - 覆盖率 `no_op_mutations_enforce_secret_mode_without_replacing_the_file`（`crates/ccr-config/src/managers/config/repository_tests.rs:190`）权限位断言失败。
3. `#113`（`@types/vscode` 1.110.0 → 1.144.0）、`#111`（`@types/node` 20 → 26）、`#103`（`dirs` 6 → 7）按升级修复调用点。扩展的 `engines.vscode` 目前是 `^1.85.0`；只有代码真正用到新 API 时才提高 engines。
4. `#90`–`#92` 若已经把 `uses:` 写成 40 位 commit SHA，就不要手改 pin。只有可变 tag 或治理脚本拒绝时才改 SHA。
5. 六条 Dependabot `schedule.interval` 改为 `monthly`。生态、目录、标签保持不变。`main` 和 `dev` 都要有这份配置。`.trellis/spec/ccr/backend/dependency-governance.md:363` 的 “Preserve the schedules” 改为六条间隔都是 `monthly`，避免后续代理改回 `weekly`。
6. 合并完成后删除已合并 PR 的远端头分支。`#114` 合并后删除远端 `ci/faster-checks`。没有打开 PR 的 `audit-remediation-hosted-20260727` 也删除。
7. 本地删除 `ci/faster-checks` 时不得丢弃未提交的桌面端改动。当前分支删不掉；只有脏文件能无冲突地跟着切到 `dev` 或 `main` 时才删除本地分支。

## Technical Notes

- 稳定 required check 名称是 `Root Workspace Required`、`Vue and Docs Required`、`Tauri Linux Required`、`VS Code Required`。不跳过、不改名、不用管理员权限合并红检查。
- 质量工作流不能加 `pull_request.paths`。路径过滤仍由 relevance job 负责。
- 同一锁文件的 Dependabot PR 要分组、一次合并一个，再 rebase 同组其余 PR。分组见 `research/pr-inventory.md`。
- `#114` 的检查跑的是 “`dev` + CI 提交” 的合并结果。先把修复合并进 `dev`，再重跑 `#114`，不必把修复提交塞进脏的 `ci/faster-checks` 检出。
- `dev` 合并 `#114` 之后，同文件 `:358` 仍写着质量工作流只在 `pull_request` 触发。这一句只在 `dev` 上改成与已合并工作流一致：`pull_request` 指向 `main` / `develop` / `dev`，并且 `push` 只指向 `main`。`main` 在收到这套 CI 改动之前保持原句。
- 本地 Windows 上 spawn `bash` 的测试会因 WSL `ext4.vhdx` 挂载失败而红。那不是本次要修的产品缺陷。

## Out of Scope

- 不提交、不暂存、不还原当前工作区里未提交的桌面端、GPUI、锁文件和既有 Trellis 任务文件。
- 不手写 Dependabot 已经提交的锁文件 bump。
- 不减少 `just ci` 或 GitHub Actions 的测试项，不把失败检查改成跳过。
- 不强制合并仍红的 required check，也不把无法安全修好的 PR 直接关闭。
- 不把 `#114` 的 CI 提速合并进 `main`。它的目标分支是 `dev`。

## Acceptance Criteria

- [ ] 24 个 PR 都已合并；若某个 PR 在修复尝试后 required check 仍失败，任务记录写明 PR 号、失败检查和停下来的原因，并且该 PR 保持打开。
- [ ] `git ls-remote --heads origin` 只剩 `main`、`dev`，以及上一条仍打开的 PR 头分支。
- [ ] `#114` 合并后，远端不存在 `ci/faster-checks`。远端不存在 `audit-remediation-hosted-20260727`。
- [ ] 本地 `ci/faster-checks` 已删除，或者任务记录写明脏工作区无法无冲突切换，因此本地分支保留。未提交的桌面端文件仍在。
- [ ] `origin/main` 和 `origin/dev` 的 `.github/dependabot.yml` 六条 `interval` 都是 `monthly`，生态、目录、标签不变。
- [ ] 两条分支上 `python scripts/ci/check_workflow_governance.py` 通过。
- [ ] `dependency-governance.md` 写明六条 Dependabot 间隔为 `monthly`，不再要求保留 `weekly`。
- [ ] 本文件 Requirements 2 里、在对应干净分支上复现过的检查变为通过。覆盖率、clippy、审计和平台测试都没有被跳过。
