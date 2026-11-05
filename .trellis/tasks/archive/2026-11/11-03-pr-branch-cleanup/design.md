# 合并 PR、修检查、改 Dependabot 周期

## 边界

修复和合并都在干净克隆里做。当前 `D:/Documents/Code/Github/ccr` 检出保持在 `ci/faster-checks`，不执行 `git switch`、`git stash`、`git reset`、`git clean` 或 `git add -A`。

新克隆只用来创建 `fix/required-checks-main`、`fix/required-checks-dev`，以及在 `#114` 与 `dev` 冲突时更新 `ci/faster-checks`。修复提交不进入桌面端脏文件。

## 数据流

```text
origin/main ── fix/required-checks-main ── merge ── main 变绿
                                              │
                                              ├─ 按锁文件分组合并 #90–#113
                                              └─ dependabot.yml interval=monthly

origin/dev ── fix/required-checks-dev ── merge ── dev 变绿
                                              │
                                              └─ 重跑并合并 #114
                                                 然后 dev 上也是 monthly
```

`#114` 的 required check 使用 PR 的 merge ref。`dev` 上的修复一旦进入基线，重跑即可覆盖，不必把修复 cherry-pick 到脏检出。

`main` 与 `dev` 已经分叉：只有 `dev` 包含 `crates/ccr-app`。共享的 Rust 测试和审计修复能 cherry-pick 就 cherry-pick；文档 crate map 只改 `dev`。

## 合并合同

- 使用 `gh pr merge --merge`。不 squash 掉 Dependabot 的提交说明，不 rebase 已经在远端的 `main` / `dev`。
- required check 仍红时不合并，也不使用管理员绕过。
- 同一锁文件组内，合并一个后立刻 rebase 同组剩余 PR。rebase 只在干净克隆里做。
- 某个升级在补了调用点、类型和治理 SHA 之后仍然红，就停在该 PR，把失败检查写入任务记录。不关闭，不删除它的头分支。
- `dirs` 7、`@types/vscode` 1.144、`@types/node` 26 允许改调用点。扩展 `engines.vscode` 只有在新 API 被实际调用时才提高。

## Dependabot

六条映射保持：

| ecosystem | directory | labels |
| --- | --- | --- |
| github-actions | `/` | dependencies, ci |
| cargo | `/` | dependencies, rust |
| cargo | `/ccr-ui/src-tauri` | dependencies, tauri |
| bun | `/ccr-ui` | dependencies, frontend |
| npm | `/ccr-vscode` | dependencies, vscode |
| bun | `/docs` | dependencies, docs |

只把 `interval: weekly` 改成 `monthly`。配置先进 `main`，因为 Dependabot 读默认分支；同一改动再进入 `dev`。

规格 `.trellis/spec/ccr/backend/dependency-governance.md:363` 改成六条间隔都是 `monthly`。`dev` 在 `#114` 合并后，把同节 `:358` 的 “pull_request-only” 改成与工作流一致的触发范围。`main` 不改 `:358`。

## 删除

合并成功并且远端头已经被 GitHub 合并提交包含之后，才删除该头分支。额外删除 `audit-remediation-hosted-20260727`。保留 `main` 和 `dev`。

本地 `ci/faster-checks` 最后处理：在当前检出上试一次 `git switch --quiet dev` 的 dry-run（`git checkout -m` 不算）。只有 Git 报告脏文件可以全部带走时才切换并 `git branch -d ci/faster-checks`。有冲突就停，本地分支留下。

## 回滚

修复 PR 在合并前可以继续推修复提交。已经合并到 `main` 或 `dev` 的提交不用 force-push 撤回。若某个 Dependabot 合并后才发现破坏，另开修复提交，不重置分支。误删的远程头可以从合并提交或 `git reflog` 恢复；删除前记录每个头的 SHA。
