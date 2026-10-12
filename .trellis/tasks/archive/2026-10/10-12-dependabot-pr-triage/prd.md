# 处理 Dependabot 依赖 PR

## Goal

把 [bahayonghang/ccr](https://github.com/bahayonghang/ccr/pulls) 上仍开放的 30 个 Dependabot PR 各给出一个终态：仍落后于 `main` 的小升级落到开发分支 `dev`，已经被 `main` 覆盖、目标依赖已删除、或会把工具链抬过当前运行时的 PR 关闭。不在这次里做 TypeScript 7、ESLint 10、或把 VS Code 最低版本抬到 1.140。

用户价值：依赖队列不再堆着过期 diff；还值得进仓库的补丁和 SHA 锁定有一处落地，而不是逐个合进落后的 `main`。

## Background

核对日为 2026-10-12。30 个 PR 全部开放，目标分支都是 `main`。本地集成分支是 `dev`：相对 `origin/main` 超前 57、落后 1。两边的依赖版本号一致；树差只剩包版本 `7.4.0` → `7.4.1`、`ccr-ui` 的 `handlebars` override，以及 lockfile 里跟着走的包版本字段。

`main` 上的运行时约束：

- `ccr-vscode` 的 `engines.vscode` 是 `^1.85.0`，`@types/vscode` 也是 `^1.85.0`。
- CI 的 Node 是 `24.20.0`（`.github/workflows/ci.yml`、`vscode-ci.yml`、`release.yml`）。`ccr-ui` 的 `@types/node` 已是 `^24.13.3`。
- `ccr-ui` 已是 React 应用。`package.json` 里没有 `vue`、`vue-router`、`vue-tsc`。
- `ccr-ui` 的 `vite` 已是精确版本 `8.2.2`，`typescript` 为 `^6.0.3`。`ccr-vscode` 的 `typescript` 也是 `^6.0.3`，`esbuild` 为 `^0.28.2`，`@vscode/vsce` 为 `^4.0.0`。
- 工作区与 `ccr-ui/src-tauri` 的 lock 已经是 `sysinfo` 0.39.6、`ts-rs` 12.0.1、`blake3` 1.8.7、`rpassword` 7.5.4、`libc` 0.2.189、`serde_json` 1.0.151、`async-trait` 0.1.92。根 lock 的 `thiserror` 仍是 2.0.20，`clap` 仍是 4.6.6。

许多「可合并但 BLOCKED」的小升级，失败检查是 `Root Tests (macos-15)` 或 `Root Tests (windows-2025)`，不是对应包自己的校验。`#88`、`#82` 的检查是绿的。不要在必需检查失败时强行合并。

## Requirements

### 落到 `dev` 的小升级

这些 PR 的目标版本仍高于 `main` / `dev` 的当前 pin 或 lock。不要把它们逐个合并进 `main`。在 `dev` 上按各 PR 的版本差改 manifest 和 lock，检查通过后，用评论说明已由 `dev` 承接，再关闭原 PR。

| PR | 变更 | 当前证据 |
| --- | --- | --- |
| #89 | `thiserror` 2.0.20 → 2.0.21 | 根 `Cargo.lock` 仍是 2.0.20。PR 只改根 lock。Tauri lock 里的 `thiserror` 2.0.20 不在这 30 个 PR 里，这次不顺手改。 |
| #88 | `postcss` `^8.5.23` → `^8.5.28` | `ccr-ui/bun.lock` 解到 8.5.26。检查为 CLEAN。 |
| #87 | `smol-toml` `^1.3.0` → `^1.9.0` | lock 解到 1.8.0。 |
| #86 | `clap` lock 4.6.6 → 4.6.7 | 根 `Cargo.toml` 为 `4.6.6`（Cargo 兼容 `^4.6.6`）。PR 只改根 `Cargo.lock`。 |
| #85 | `react-hook-form` 7.87.0 → 7.89.0 | `ccr-ui/package.json` 仍钉 7.87.0。 |
| #84 | `stylelint` `^17.12.0` → `^17.16.0` | lock 解到 17.14.1。 |
| #82 | `@tanstack/react-virtual` 3.14.10 → 3.14.13 | 精确版本仍是 3.14.10。检查为 CLEAN。 |
| #81 | `@iconify-json/solar` `^1.2.10` → `^1.2.13` | lock 解到 1.2.10。 |
| #80 | `softprops/action-gh-release` 只换锁定 SHA | `release.yml` 仍是 `5113cdc90fd4d541c801c55356214017bf5ae34b`。 |
| #44 | `tauri-apps/tauri-action` 只换锁定 SHA | `release.yml` 仍是 `fce9c6108b31ea247710505d3aaaa893ee6768d4`。 |

`#80` 与 `#44` 保持「只换 SHA、不改调用方式」。若上游 tag 注释或入参和现有 workflow 调用对不上，这一条不落地，改成关闭并写明原因。

### 关闭且不 recreate

这些 PR 的 diff 基于旧树，合进去会回退、空转，或把类型/运行时抬过当前声明。关闭时评论里写原因。不发 `@dependabot recreate`。

| PR | 关闭原因 |
| --- | --- |
| #83 | `@types/vscode` 从 `^1.85.0` 抬到 `^1.140.0`，`engines.vscode` 仍是 `^1.85.0`。类型面会允许 1.140 才有的 API，最低引擎不变。这次不抬 `engines.vscode`。 |
| #77 | Tauri lock 的 `async-trait` 已是 0.1.92。 |
| #75 | 根 lock 的 `libc` 已是 0.2.189。 |
| #74 | Tauri lock 的 `blake3` 已是 1.8.7。 |
| #70 | `ccr-vscode` 的 `esbuild` 已是 `^0.28.2`。PR 基于旧清单（同时还停在 `esbuild` 0.27、`typescript` 5.7），且与 `main` 冲突。 |
| #66 | 根 lock 的 `rpassword` 已是 7.5.4。 |
| #65 | Tauri lock 的 `serde_json` 已是 1.0.151。 |
| #64 | Tauri lock 的 `sysinfo` 已是 0.39.6，且 PR 冲突。 |
| #62 | 工作区 `ts-rs` 已是 12.0.1，且 PR 冲突。 |
| #61 | `ccr-cli` / `ccr-codex` 的 `sysinfo` 已是 0.39.6。0.38 → 0.39 已经在 `main` 上。 |
| #55 | `vue-tsc` 已不在 `ccr-ui/package.json`。 |
| #53 | 当前 TypeScript 是 `^6.0.3`。PR 从旧的 `^5.7.0` 跳到 `^7.0.2`，且冲突。不在这次做 TypeScript 7。 |
| #51 | `vue-router` 已不在仓库里。 |
| #50 | `vite` 已是 `8.2.2`，高于 PR 目标 `^8.2.1`。PR 还基于旧的 Vue 清单。 |
| #49 | `@vscode/vsce` 已是 `^4.0.0`。PR 只升到 `^3.9.2`，且基于旧清单。 |
| #48 | CI Node 是 24.20.0，`@types/node` 已是 `^24.13.3`。PR 要升到 `^26.2.0`，且 diff 还带着已删除的 Vue 依赖。不对齐到 26。 |

### 大版本关闭且不 recreate

已按规划的推荐答案执行：这 4 个 PR 不合并、不 recreate、不在 `dev` 上做迁移。关闭评论注明另案，不由这次 Dependabot 队列落地。

| PR | 原因 |
| --- | --- |
| #54 | ESLint 9 → 10。当前仍是 `eslint` `^9.39.4`，但 PR diff 对着旧的 Vue 版 `package.json`，且与 `main` 冲突。 |
| #69 | `actions/upload-artifact` 6 → 7。分支落后，CI 大版本。 |
| #46 | `actions/checkout` 6 → 7。与 `main` 冲突，CI 大版本。 |
| #45 | `actions/setup-node` 6 → 7。与 `main` 冲突，CI 大版本。 |

## Acceptance Criteria

- [ ] 上表「落到 `dev`」的每一条，要么出现在 `dev` 的对应 manifest/lock/workflow 里，要么有一条写明拒绝原因的 PR 关闭评论。
- [ ] `#89` 不修改 `ccr-ui/src-tauri/Cargo.lock` 的 `thiserror`。
- [ ] `#80`、`#44` 若落地，workflow 里只改锁定 SHA，调用参数保持不动。
- [ ] `#83` 关闭。`ccr-vscode` 的 `engines.vscode` 仍是 `^1.85.0`，`@types/vscode` 仍是 `^1.85.0`。
- [ ] 「关闭且不 recreate」表中的 PR 均已关闭，评论含上表原因，且没有 `@dependabot recreate`。
- [ ] `#48` 关闭后，`@types/node` 仍覆盖 Node 24，而不是 26。
- [ ] `#54`、`#69`、`#46`、`#45` 已关闭且不 recreate。`dev` 上的 ESLint 仍是 9.x，`actions/checkout`、`actions/setup-node`、`actions/upload-artifact` 仍是 v6 锁定。
- [ ] 没有对必需检查失败的 Dependabot PR 执行管理员强行合并。

## Out of Scope

- 把 `dev` 合并回 `main`，或发布版本。
- TypeScript 7、ESLint 10、`engines.vscode` 抬到 1.140、`@types/node` 26。
- 补一个 Tauri 侧的 `thiserror` 2.0.21 PR。
- 修改 Dependabot 分组或 schedule（`.github/dependabot.yml` 保持不动）。
- ESLint 10，以及 `actions/checkout`、`actions/setup-node`、`actions/upload-artifact` 的 v7 迁移。
