# 实施顺序

当前检出是脏的。下面的命令都在新的干净克隆里执行，除非某一步明确写了当前检出。

## 清单

1. 克隆一份干净仓库到临时目录。确认 `origin/main` 与 `origin/dev`。记录将删除的每个远程头 SHA。
2. 从 `origin/main` 建 `fix/required-checks-main`。用最窄命令复现根检查：`cargo clippy` 相关包、`cargo test -p ccr --test commands`、`cargo test -p ccr-config --lib no_op_mutations_enforce_secret_mode_without_replacing_the_file`、`cargo audit`。只修复现出来的失败。
3. 在该分支把 `.github/dependabot.yml` 六条 `interval` 改为 `monthly`，并改 `dependency-governance.md:363`。不改 `:358`。
4. 推送并打开指向 `main` 的 PR。required check 变绿后 `gh pr merge --merge`。
5. 从 `origin/dev` 建 `fix/required-checks-dev`。复现同一组 Rust 失败，另加 `docs` 的 crate map 审计。把 `crates/ccr-app` 补进中英文 crate map。同样改 monthly 和 `:363`。
6. 推送并合并到 `dev`。然后重跑 `#114`。冲突时在干净克隆里把 `dev` 合并进 `ci/faster-checks` 再推送。变绿后 `gh pr merge --merge 114`。
7. `#114` 进入 `dev` 后，把 `dependency-governance.md:358` 改成实际触发范围：pull request 到 `main` / `develop` / `dev`，push 只到 `main`，`cancel-in-progress` 只对 pull request。
8. `main` 变绿后，按 `research/pr-inventory.md` 的锁文件分组依次合并 `#90`–`#113`。每组合并一个就 rebase 其余。类型、dirs 7 或治理 SHA 失败时，把修复提交推到该 Dependabot 分支，不手改锁文件里已有的 bump。
9. 记录仍打不开的 PR。其余已合并头分支从 origin 删除。删除 `audit-remediation-hosted-20260727` 和已合并的 `ci/faster-checks`。
10. 回到当前检出。仅当脏文件能无冲突跟随切换时，切到 `dev` 并删除本地 `ci/faster-checks`。否则写明保留原因。

## 验证

- `python scripts/ci/check_workflow_governance.py` 在 `main` 修复分支和 `dev` 修复分支上各跑一次。
- 复现过的窄测试在修复后通过。平台专有失败以 GitHub required check 为准，不拿本地 WSL bash 失败当结论。
- `git ls-remote --heads origin` 对照验收标准。
- `git status --short` 在原检出上仍能看到原来的桌面端改动。

## 风险文件

- `Cargo.lock`、`ccr-ui/src-tauri/Cargo.lock`、`ccr-ui/bun.lock`、`ccr-vscode/package-lock.json`：只接受 Dependabot diff 和为通过审计所做的范围内升级。
- `.github/workflows/*.yml`：`#90`–`#92` 与 `#114` 都会碰。不要在 `main` 上提前套用 `#114`。
- `crates/ccr-config/src/managers/config/repository_tests.rs`：当前检出是脏的，只在干净克隆里改。
- `docs/reference/internals/crate-map.md` 与英文对应页：只在 `dev` 补 `crates/ccr-app`。

## 开始前

`task.py start` 只能在用户确认本规划之后执行。开始时不要把当前脏检出设为实施分支。
