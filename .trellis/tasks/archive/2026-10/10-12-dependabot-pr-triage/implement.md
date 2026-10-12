# 处理 Dependabot 依赖 PR

在 `dev` 上工作。禁止 `git commit`、`git push`、`git merge`，禁止 `@dependabot recreate`，禁止 `--admin` 合并。

## 1. 先关掉不落地的 20 个 PR

仓库：`bahayonghang/ccr`。每条评论用中文，带上 PRD 里的原因，并写明不 recreate。

```powershell
gh pr close <n> --repo bahayonghang/ccr --comment "<reason>. 不 recreate。"
```

- `#83`：`@types/vscode` 升到 1.140 但 `engines.vscode` 仍是 `^1.85.0`，这次不抬最低引擎。
- `#77` `#75` `#74` `#66` `#65` `#64` `#62` `#61`：目标版本已经在 `main` 的对应 lock / manifest 上。
- `#70`：`esbuild` 已是 `^0.28.2`，PR 基于旧清单且冲突。
- `#55` `#51`：`vue-tsc` / `vue-router` 已不在仓库。
- `#53`：当前 TypeScript 是 `^6.0.3`，不做 7。
- `#50`：`vite` 已是 `8.2.2`。
- `#49`：`@vscode/vsce` 已是 `^4.0.0`。
- `#48`：Node CI 是 24，`@types/node` 已是 `^24`，不升到 26。
- `#54`：ESLint 10 另案，当前 diff 还对着旧 Vue 清单。
- `#69` `#46` `#45`：Actions v7 另案，本次保持 v6 锁定。

## 2. 落地小升级

根目录：

```powershell
cargo update -p thiserror@2.0.20 --precise 2.0.21
cargo update -p clap --precise 4.6.7
```

确认只改了根 `Cargo.lock`。`ccr-ui/src-tauri/Cargo.lock` 的 `thiserror` 仍是 2.0.20。根 `Cargo.toml` 的版本字符串不动。

`ccr-ui/package.json` 改为：

- `postcss`: `^8.5.28`
- `stylelint`: `^17.16.0`
- `@iconify-json/solar`: `^1.2.13`
- `react-hook-form`: `7.89.0`
- `@tanstack/react-virtual`: `3.14.13`

然后只更新这五个包：

```powershell
bun update postcss stylelint @iconify-json/solar react-hook-form @tanstack/react-virtual
```

工作目录是 `ccr-ui`。不要顺手升级其它依赖。

`ccr-vscode`：

```powershell
npm install smol-toml@^1.9.0
```

`package.json` 的范围应成为 `^1.9.0`，lock 解到 1.9.x。不要改 `engines.vscode` 或 `@types/vscode`。

`.github/workflows/release.yml` 只替换这三处 SHA，注释仍是 `# v3` / `# v0`，`with:` 和 `env:` 保持原样：

- `softprops/action-gh-release@efb35369e0ad2afab669f228072c1b0d510eae64`（两处）
- `tauri-apps/tauri-action@84b9d35b5fc46c1e45415bdb6144030364f7ebc5`（一处）

## 3. 检查

```powershell
python scripts/drift/check_dependency_drift.py --verbose
just version-check
```

根目录执行。另外：

```powershell
cd ccr-ui; bun install --frozen-lockfile
cd ccr-vscode; npm ls smol-toml
```

对照 lock 确认：根 `thiserror` 2.0.21、`clap` 4.6.7；Tauri lock 的 `thiserror` 仍是 2.0.20；`postcss` 至少 8.5.28；`stylelint` 至少 17.16.0；iconify 至少 1.2.13；`react-hook-form` 7.89.0；`@tanstack/react-virtual` 3.14.13；`smol-toml` 1.9.x。

`eslint` 仍是 `^9.39.4`。三个 Actions 仍指向原来的 v6 SHA。

检查失败则还原落地组文件，不要关下面 10 个 PR：

```powershell
git checkout -- Cargo.lock ccr-ui/package.json ccr-ui/bun.lock ccr-vscode/package.json ccr-vscode/package-lock.json .github/workflows/release.yml
```

## 4. 检查通过后再关这 10 个

`#89` `#88` `#87` `#86` `#85` `#84` `#82` `#81` `#80` `#44`。

评论：`版本差已写进本地分支 dev 的工作区，不合并此 PR，也不 recreate。尚未提交或推送。`

## 5. 回报

列出改过的文件、关过的 PR 号、检查命令和结果。不要 commit。
