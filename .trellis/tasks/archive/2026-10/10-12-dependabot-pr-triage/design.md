# 处理 Dependabot 依赖 PR

## Boundary

改动只发生在本地分支 `dev` 的依赖清单、lock 和 `release.yml` 的 action SHA。不把 Dependabot PR 合并进 `main`。`dev` 比 `main` 超前，两边的依赖版本号目前一致；把小升级写进 `dev`，等以后 `dev` 合入 `main` 时一起带上。

不提交、不推送。关闭 GitHub PR 使用 `gh pr close` 加评论，不使用 `@dependabot recreate`，也不对红检查做管理员合并。

## Land on dev

| 表面 | 文件 | 目标 |
| --- | --- | --- |
| 根 Rust lock | `Cargo.lock` | `thiserror` 2.0.20 → 2.0.21（保留 1.0.69 那条）；`clap` 4.6.6 → 4.6.7 |
| UI manifest / lock | `ccr-ui/package.json`、`ccr-ui/bun.lock` | `postcss` `^8.5.28`；`stylelint` `^17.16.0`；`@iconify-json/solar` `^1.2.13`；`react-hook-form` `7.89.0`；`@tanstack/react-virtual` `3.14.13` |
| 扩展 manifest / lock | `ccr-vscode/package.json`、`ccr-vscode/package-lock.json` | `smol-toml` `^1.9.0` |
| Release workflow | `.github/workflows/release.yml` | `softprops/action-gh-release` 两处换成 `efb35369e0ad2afab669f228072c1b0d510eae64`；`tauri-apps/tauri-action` 换成 `84b9d35b5fc46c1e45415bdb6144030364f7ebc5`。`with:` / `env:` 不动 |

根清单里的 `thiserror = "2.0.20"` 和 `clap = { version = "4.6.6", ... }` 保持不动。Cargo 把它们当成兼容范围，这次只跟 Dependabot 一样动 lock。

`ccr-ui/src-tauri/Cargo.lock` 的 `thiserror` 保持 2.0.20。Tauri 不是根 workspace member，根目录的 `cargo update` 不得改到那份 lock。

## Do not change

- `ccr-vscode` 的 `engines.vscode` 与 `@types/vscode`（都保持 `^1.85.0`）
- `ccr-ui` 的 `@types/node`（保持 `^24`）
- `eslint`、`typescript`、`vite`、`esbuild`、`@vscode/vsce`
- `actions/checkout`、`actions/setup-node`、`actions/upload-artifact` 的 v6 SHA
- `.github/dependabot.yml`
- 任何产品源码

## Close without recreate

关闭时评论写原因，不发 `@dependabot recreate`。

已在 `main` 达到或超过目标，或 diff 基于已删除的 Vue 树：`#83` `#77` `#75` `#74` `#70` `#66` `#65` `#64` `#62` `#61` `#55` `#53` `#51` `#50` `#49` `#48`。

另案大版本：`#54` `#69` `#46` `#45`。

10 个已落地的小升级 PR（`#89` `#88` `#87` `#86` `#85` `#84` `#82` `#81` `#80` `#44`）在本地检查通过后再关。评论说明版本差在 `dev` 工作区，不合并该 PR。检查失败则这 10 个保持开放。

## Rollback

落地组是一组 lock/manifest/SHA 编辑。检查失败时用 `git checkout --` 还原该组文件，不关这 10 个 PR。已经关闭的过期 PR 可以再 `gh pr reopen`，但只有在关错了的时候才重开。
