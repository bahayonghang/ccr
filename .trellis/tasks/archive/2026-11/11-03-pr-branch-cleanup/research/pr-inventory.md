# 打开的 PR 与分支清单

规划时从 `gh pr list --repo bahayonghang/ccr --state open` 读取。全部 `mergeable=MERGEABLE`、`mergeStateStatus=BLOCKED`。

## 人类 PR

| PR | 头分支 | 目标 | 说明 |
| --- | --- | --- | --- |
| #114 | `ci/faster-checks` | `dev` | CI 提速。头提交 `3a44b53`。Vue/Docs、Root Workspace、Root Coverage、Root Tests windows/macos、Rust Security Audit 以及对应 required 聚合失败。VS Code 与暖运行后的 Tauri 门禁已通过。 |

## Dependabot → `main`

按锁文件分组。同组一次只合并一个，然后 rebase 其余。

根 `Cargo.lock`：

- #105 indexmap 2.14.1 → 2.14.2
- #104 toml 1.1.4+spec-1.1.0 → 1.1.8+spec-1.1.0
- #103 dirs 6.0.0 → 7.0.0
- #101 libc 0.2.189 → 0.2.190
- #97 comfy-table 8.0.0 → 8.0.1

`ccr-ui/src-tauri/Cargo.lock`：

- #102 lru 0.18.3 → 0.18.5
- #100 syn 3.0.4 → 3.0.9
- #98 reqwest 0.13.4 → 0.13.5
- #95 indexmap 2.14.1 → 2.14.2
- #93 toml 1.1.4+spec-1.1.0 → 1.1.6+spec-1.1.0

`ccr-ui` Bun：

- #110 zod 4.4.3 → 4.6.7
- #109 @codemirror/legacy-modes 6.5.3 → 6.5.5
- #108 @radix-ui/react-popover 1.1.23 → 1.3.1
- #107 @typescript-eslint/parser 8.68.0 → 8.71.2
- #106 @codemirror/commands 6.11.0 → 6.11.1

`ccr-vscode` npm：

- #113 @types/vscode 1.110.0 → 1.144.0
- #112 @vscode/vsce 4.0.0 → 4.1.0
- #111 @types/node 20.19.43 → 26.6.6
- #99 tsx 4.23.13 → 4.23.15
- #96 smol-toml 1.8.0 → 1.9.1

GitHub Actions 工作流：

- #92 actions/cache 5.1.0 → 6.1.0
- #91 actions/upload-artifact 6.0.0 → 7.0.2
- #90 actions/setup-node 6.5.0 → 7.1.0

没有 #94。

## 抽样时的失败类别

这些名字来自上一轮对 `#114`、`#113`、`#110`、`#105`、`#92`、`#90` 的 `statusCheckRollup`。实施前用当时的头 SHA 再确认，不要把脏工作区的结果当成 CI 结果。

- 根 Cargo、Actions 类 PR 共同失败：`Rust Security Audit`、`Root Tests (macos-15)`、`Root Workspace Required`。`#105` 还有 macOS 根测试。这是 `main` 上的基线红。
- `#110` 额外失败 `Vue and Docs Validation`。先看日志。`main` 没有 `crates/ccr-app`，不要套用 `dev` 的 crate map 结论。
- `#113` 额外失败 `VS Code Validation`。`@types/vscode` 跨度大，优先看类型错误。
- `#92` 抽样时没有单独的治理失败。治理脚本只要求第三方 `uses:` 是 40 位 SHA，不绑定某一个 commit。

## 基线锚点

锚点读自 `ci/faster-checks` 的 `3a44b53`。`repository_tests.rs` 在当前工作区是脏文件，实施时以干净的 `origin/dev` / `origin/main` 为准。

- `crates/ccr-cli/src/commands/lifecycle/clean_storage.rs:766` 与 `:769`
- `crates/ccr/tests/commands/grok_profile.rs:241`，断言辅助函数在 `:103`
- `crates/ccr/tests/commands/output_presentation.rs:564`
- `crates/ccr-config/src/managers/config/repository_tests.rs:190`
- `docs/scripts/audit-docs.mjs:278`
- `crates/ccr-core/src/core/process_gateway.rs:914` 的 `detached_terminate_tree_reclaims_unix_descendants` 在第一次冷 macOS Tauri smoke 失败，同 SHA 暖重跑通过。没有复现时不改这个测试。

## 分支与工作区

本地当前分支 `ci/faster-checks`。未提交改动包含 `ccr-desktop/**`、`crates/ccr-app/**`、若干 Tauri command、`Cargo.lock`，以及既有 GPUI Trellis 任务。这些文件不属于本任务。

远程多出来的、没有打开 PR 的头是 `audit-remediation-hosted-20260727`。
