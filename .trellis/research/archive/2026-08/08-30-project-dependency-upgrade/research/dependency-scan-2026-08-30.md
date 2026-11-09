# 依赖扫描基线（2026-08-30）

## 1. 扫描边界与方法

本报告只记录规划期的只读扫描结果。没有修改产品 `Cargo.toml`、`package.json`、锁文件或源码，也没有安装 `cargo-outdated` / `cargo-deny`。动态版本与公告在实施前必须刷新。

覆盖面：

| 依赖面 | 权威清单 | 锁/解析依据 | 扫描方式 |
|---|---|---|---|
| 根 Rust workspace（13 crates） | `Cargo.toml` + `crates/*/Cargo.toml` | `Cargo.lock`（488 packages） | `cargo metadata/tree/audit`、crates.io API |
| Tauri Rust workspace（desktop + proc-macro） | `ccr-ui/src-tauri/Cargo.toml`、`command-macros/Cargo.toml` | `ccr-ui/src-tauri/Cargo.lock`（678 packages） | `cargo metadata/tree/audit`、crates.io API |
| React UI | `ccr-ui/package.json` | `ccr-ui/bun.lock` | `bun outdated/audit`、仓库 `audit:dependencies` |
| VS Code extension | `ccr-vscode/package.json` | `ccr-vscode/package-lock.json` | `npm outdated/audit/explain` |
| VitePress docs | `docs/package.json` | CI 权威为 `docs/bun.lock` | `bun outdated/audit/why` |
| CI Actions | `.github/workflows/*.yml` | pinned commit SHA | `git ls-remote` 对照 major tag/branch |
| 工具链 | `rust-toolchain.toml`、Cargo `rust-version`、CI workflow、`packageManager` | CI 固定值 | 官方发布计划/发布说明 |

`docs/package-lock.json` 也被跟踪，但 `just docs-check` 与文档 README 都使用 Bun；根 `just docs` 仍使用 `npm install`，形成双包管理器/双锁源漂移风险。没有发现仍在使用的 Cargo git/rev 依赖；`llmusage` 当前是 crate-free CLI + read-only SQLite adapter，不再是 AGENTS/旧 code map 所述的 pinned Rust git crate（`ccr-ui/src-tauri/src/llmusage_adapter/mod.rs:1-4`、`crates/ccr-usage/Cargo.toml`）。

## 2. 当前安全发现

### 2.1 Rust

| 锁文件 | 结果 | 引入路径 | 处置边界 |
|---|---|---|---|
| 根 `Cargo.lock` | 0 vulnerability；`chacha20 0.10.0` yanked | `rand 0.10.2 -> uuid/rand` | 2026-08-30 动态刷新确认 crates.io 已有兼容 `chacha20 0.10.2`；先做定向 lock refresh，仍不得把 yanked 自动等同为可利用漏洞 |
| Tauri `Cargo.lock` | `chacha20 0.10.0` 同样被 yanked | `rand 0.10.2 -> ccr-db/uuid` | 根/Tauri 锁图独立；S0a 不会修复此解析，需用独立 S0c 定向刷新到兼容的 `0.10.2` |
| Tauri `Cargo.lock` | `RUSTSEC-2026-0204`：`crossbeam-epoch 0.9.18`，修复于 `>=0.9.20` | `rayon` 与 `moka` | 根锁已是 0.9.20；Tauri 可先尝试锁文件级定向更新 |
| Tauri `Cargo.lock` | `RUSTSEC-2026-0221` warning：`event-listener 5.4.1` 的 `StackSlot` 可使 `!Send` tag 跨线程 | `moka 0.12.15 -> async-lock/event-listener` | crates.io 已有兼容 `event-listener 5.4.2`；先做独立定向 lock refresh，并复扫确认 warning 消失 |
| Tauri `Cargo.lock` | `RUSTSEC-2026-0194/0195`：`quick-xml 0.38.4` 两项 CVSS 7.5 DoS，修复于 `>=0.41.0` | `tauri-utils 2.9.2 -> plist 1.8.0 -> quick-xml` | 依赖路径存在，但 CCR 没有直接调用 `quick-xml`；远程不可信 XML 可达性未证实，标记 `UNVERIFIED`，仍需消除 vulnerable resolution |
| Tauri `Cargo.lock` | GTK3 bindings、`fxhash` 等 unmaintained informational warnings | Tauri Linux backend / `tauri-utils` 构建链 | 不是 Windows 当前运行路径的已证实漏洞；需要上游迁移，不能用简单 lock bump 保证消失 |

权威公告：

- <https://rustsec.org/advisories/RUSTSEC-2026-0194.html>
- <https://rustsec.org/advisories/RUSTSEC-2026-0195.html>
- <https://rustsec.org/advisories/RUSTSEC-2026-0204.html>
- <https://rustsec.org/advisories/RUSTSEC-2026-0221.html>

注意：实施 Phase 0 已联网刷新 RustSec advisory database（1226 条）并复扫两套锁图；动态结果仍需在各安全批次完成后再次确认。

### 2.2 JavaScript

| 依赖面 | 结果 | 直接/传递 | 修复形状 |
|---|---|---|---|
| React UI | Bun audit 0；仓库 dependency audit `0 reported advisories, 0/0 active exceptions` | — | 保持每批复扫 |
| VitePress docs | 8 advisories / 4 transitive packages：`nanoid 3.3.11`、`postcss 8.5.6`、`preact 10.28.0`、`rollup 4.53.3` | 全部来自 `vitepress 1.6.4` 构建链 | `bun audit fix --dry-run` 可在现有直接版本范围内更新到 3.3.18 / 8.5.23 / 10.28.2 / 4.59.0；优先低风险锁刷新 |
| VS Code extension | 13 package findings：1 low、5 moderate、7 high | 12 项来自 `@vscode/vsce 3.7.1` 打包/发布链；直接 `esbuild 0.27.4` 命中 Windows dev-server 任意文件读取（low） | `@vscode/vsce 3.9.2` 需独立验证；`esbuild >=0.28.1` 修复，但 0.27 -> 0.28 是 pre-1.0 breaking 边界，npm 也标记 `--force` |

`esbuild` advisory：<https://github.com/advisories/GHSA-g7r4-m6w7-qqqr>。CCR extension 仅用 esbuild bundle/watch，未配置 `servedir`；因此仓库使用点对该具体 dev-server 漏洞的可达性未证实，但 vulnerable direct dependency 仍应升级。

## 3. 过期直接依赖摘要

### 3.1 Rust（63 个唯一 registry 直接 crate）

低风险候选（同兼容线 patch/minor，仍需锁图复核）：

- `arc-swap 1.9.1 -> 1.9.2`
- `filetime 0.2.27 -> 0.2.29`
- `indexmap 2.14.0 -> 2.14.1`
- `libc 0.2.183 -> 0.2.189`
- `lru 0.18.2 -> 0.18.3`
- `moka 0.12.15 -> 0.12.16`
- `open 5.4.1 -> 5.4.2`
- `quote 1.0.45 -> 1.0.47`
- `rayon 1.11.0 -> 1.12.0`
- `tracing-appender 0.2.4 -> 0.2.5`
- `tracing-subscriber` 统一到 `0.3.23`
- `uuid 1.25.0 -> 1.26.0`
- exact pin `whoami 2.1.0 -> 2.1.3`

需要聚焦验证的同-major/Tauri 协同候选：

- `tauri =2.11.2 -> =2.11.5`、`tauri-build =2.6.2 -> =2.6.3`、`tauri-plugin-dialog 2.6.0 -> 2.7.2`
- `sysinfo` 根 workspace `0.38.4` 与 Tauri `0.39.6` 统一（0.x minor 视为 breaking-risk）

明确跨兼容边界：

- `aes-gcm 0.10.3 -> 0.11.1`
- `argon2 0.5.3 -> 0.6.0`
- `comfy-table 7.2.2 -> 8.0.0`
- 根 `fs4 0.13.1 -> 1.1.0`（Tauri 已是 1.1.0）
- proc-macro direct `syn 2.0.117 -> 3.0.4`

### 3.2 React UI

可先做的范围内 patch/minor：CodeMirror `commands/lang-markdown/view`、`@iconify-json/solar`、`dompurify`、`@types/react-dom`、TypeScript-ESLint 8.x、Vite React plugin 6.x、Vitest/coverage 4.x、Playwright 1.x。

同 major 但 manifest 当前 exact pin、需单独审查意图：`@tanstack/react-query 5.101.4 -> 5.102.8`、`@tauri-apps/api 2.11.0 -> 2.11.1`、`react-hook-form 7.86.0 -> 7.87.0`、`react-router 8.3.0 -> 8.3.1`、`zod 4.4.3 -> 4.5.4`、Tauri CLI 2.11.2 -> 2.11.4、Testing Library 16.3.2 -> 16.3.3。

明确 major / 高风险：

- `apexcharts 5.16.0 -> 7.0.0`：仓库依赖 `apexcharts/core`、类型/feature 子路径、CSS/DOM 合同与性能脚本（`ccr-ui/src/utils/apexChartsCore.ts:3-24`、`ccr-ui/tests/usage/apexcharts-style-contract.smoke.test.ts:13-24`），且上游 6.x/7.x 引入模块、行为与许可相关变化；必须单独做可视/性能/许可审查。
- ESLint 9 -> 10（含 `@eslint/js`）：官方迁移指南列出 Node 最低版本、config lookup、recommended 规则和移除 deprecated context/SourceCode API 等 breaking changes。
- `jsdom 26.1.0 -> 30.0.1`：当前测试显式按 jsdom 26 缺失的 PointerEvent/ResizeObserver/scrollIntoView 打桩（`ccr-ui/tests/ui/ui-primitives.smoke.test.tsx:49-94`），升级可能改变测试语义。
- TypeScript 5.9 -> 7.0：TypeScript 7 是 Go 原生重写且 7.0 不提供编译器 API；官方明确说明 typescript-eslint 等依赖 programmatic API 的工具需要 TS 6 compatibility package。当前仓库使用 typescript-eslint，因此 7.0 暂不具备直接升级条件。
- `globals 16 -> 17`、`@types/node 22 -> 26` 需要与运行时/CI Node 策略一起处理，不能作为普通 devDependency bump。

### 3.3 VS Code / docs / Actions

- VS Code 可更新：`@types/node 20.19.37 -> 20.19.43`、`@vscode/vsce 3.7.1 -> 3.9.2`、`smol-toml 1.6.1 -> 1.8.0`、`tsx 4.21.0 -> 4.23.13`、`esbuild 0.27.4 -> 0.28.2`。
- `@types/vscode` 当前 range `^1.85.0` 已解析 1.110 且可继续漂到 1.134，但 extension engine 仍是 `^1.85.0`。继续升级类型可能允许误用旧 VS Code 不存在的 API；应先决定是把类型 pin 到最低支持版本，还是提高 engine 下限。
- docs 的直接 `vitepress ^1.6.4` 没有稳定直接更新；问题是传递锁与双包管理器漂移。
- Actions：checkout v6、setup-node v6、upload-artifact v6、cache v5、setup-bun v2、tauri-action v0 均与当前 major tag SHA 一致；`softprops/action-gh-release@v3` pinned `c125...` 落后 tag `5113...`，`dtolnay/rust-toolchain@stable` pinned `4cda...` 落后 branch `4360...`。

## 4. 废弃 API 与 Breaking Change 证据

- 当前基线通过：根 `just lint-strict`；Tauri Clippy `-D warnings`；UI typecheck + ESLint/Stylelint；VS Code lint；VitePress build。未出现新的 compiler/type deprecation warning。
- 已知实际废弃使用：`crates/ccr-checkin/src/core/crypto.rs:4-5` 明确记录 `aes_gcm` 经 generic-array 0.x 暴露的 `from_slice/as_slice` 已废弃；相同 `Nonce::from_slice` 还在 `ccr-codex` 与 `ccr-sync`。`aes-gcm 0.11` / `aead 0.6` 迁移需改为 fallible nonce/tag construction，并做密文兼容回归。
- 已知确定 breaking：comfy-table 8 官方 changelog 删除 `Table::load_preset`；仓库在 CLI 命令层有大量真实调用，例如 `crates/ccr-cli/src/commands/common/table.rs:30`、`profile/current.rs:40`，不能锁文件直升。
- ESLint 10 官方 breaking guide：<https://eslint.org/docs/latest/use/migrate-to-10.0.0>。
- TypeScript 7 官方说明：<https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/>。
- comfy-table 8 changelog：<https://github.com/Nukesor/comfy-table/blob/main/CHANGELOG.md>。
- Tauri 2.11.5 release：<https://github.com/tauri-apps/tauri/releases>（2.11.5 主要为解除 `time` pin，仍需整套门禁）。

## 5. 工具链

- 仓库/CI：Rust 1.95.0、Bun 1.3.10、Node 24.18.0；本机为 Rust 1.95.0、Bun 1.4.0、Node 26.7.0。
- 2026-08-30 官方 stable Rust 已到 1.98.0；升级 compiler 可能触发新 lint，但 MSRV `1.95` 是独立兼容合同，不能因本地 compiler 升级而默默抬高。
- Node 24 当前是 Active LTS；Node 26 在 2026-10-28 前仍为 Current。仓库 release/CI 不应仅因本机已有 Node 26 就跨 major。
- Bun 1.4 重写核心实现（Zig -> Rust）；即使 package manager lock 格式宣称兼容，也应作为工具链独立批次跑全门禁，而非与普通 npm package bump 混合。

## 6. 当前证据缺口

- Rust advisory 全库的联网 fresh scan 尚未运行；不修改用户全局 advisory DB 的前提下，本轮只用了缓存并逐项核验已发现公告。
- `quick-xml` 两项漏洞的 CCR 远程输入可达性未证实；存在 vulnerable resolution 是事实，可利用性为 `UNVERIFIED`。
- 未在规划期生成假想升级锁文件，因此 `@vscode/vsce 3.9.2` 能消除多少传递漏洞、Tauri 2.11.5 是否把 `plist/quick-xml` 推到修复线，需要在对应实施批次由真实 diff + 复扫确认。
- 未运行整套 `just ci` 作为规划基线；只运行了与 deprecated API/构建链相关的静态门禁和 docs build。每个实施批次仍必须运行完整 `just ci`。

## 7. 实施后复扫（2026-08-31）

本节覆盖并关闭第 6 节的规划期缺口；基线内容保留用于说明升级前状态。所有安全修复均先于普通更新完成，随后按低、中、高风险逐项推进。每个独立批次都完成聚焦检查、安全复扫、独立复核和完整 `just ci`，具体版本、迁移与日志见 `implement.md`。

### 7.1 最终安全状态

| 生态 | 最终结果 | 处置边界 |
|---|---|---|
| 根 Rust 锁图 | 491 crates，0 vulnerability、0 warning、无 yanked current crate | `.cargo/audit.toml` 的 `ignore = []`，没有新增人工豁免 |
| Tauri Rust 锁图 | 0 vulnerability；19 条既有 warning（17 unmaintained、2 unsound） | 漏洞与 yanked 解析均已修复；warning 为不可由当前 direct compatible bump 消除的传递生态状态，未加入 ignore |
| React UI | Bun audit：668 packages，0 vulnerability；custom audit：0 advisory / 0 active exception | 无安全例外 |
| docs | Bun audit：179 packages，0 vulnerability；docs audit 通过 | 无安全例外 |
| VS Code | npm audit：0 vulnerability | `esbuild` advisory 已随 S5 消除，未扩大 install-script 授权 |

安全批次实际清除了：两套锁图的 yanked `chacha20 0.10.0`、Tauri `event-listener` unsound warning、Tauri `crossbeam-epoch` 与 `quick-xml` 漏洞、docs 传递漏洞、UI `axios` 漏洞、VS Code `@vscode/vsce` 与 `esbuild` 漏洞。最终未发现仍可由已批准直接升级安全消除的已知漏洞。

### 7.2 最终 outdated / current 状态

- Rust：90 个 direct manifest rows / 63 个唯一 registry crates 均解析在当前 manifest range 可达到的 crates.io 最新版本；根/Tauri 没有 git、rev 或 branch 依赖。Tauri 复扫遗漏的 `tauri-plugin-dialog 2.7.2` 及其传递 `tauri-plugin-fs 2.5.1` 已补升。
- React UI：仅剩 `apexcharts 7.0.0`、`zod 4.5.4`、ESLint 10、`@types/node 26.4.0`、TypeScript 7.0.2；均为下面明确的 NO-GO/deferred，而非遗漏的安全更新。
- docs：`bun outdated` 无输出，直接依赖 current。
- VS Code：仅剩 `@types/node 26.4.0`、`@types/vscode 1.134.0`、TypeScript 7.0.2；运行时目标和 extension engine 边界阻止直接跟 latest。
- 工具链：Rust 1.98.0、Node 24.20.0 LTS、Bun 1.4.0、just 1.58.0、cargo-llvm-cov 0.9.0 均对齐 2026-08-31 的已核验目标；crate MSRV 仍为 Rust 1.95。
- GitHub Actions：checkout v6、setup-node v6、upload-artifact v6、cache v5、setup-bun v2、tauri-action v0、action-gh-release v3 与 rust-toolchain stable 均固定到复扫时对应官方 tag/branch SHA。

### 7.3 明确 deferred / Breaking / deprecated

| 项目 | 最终决策 | 理由与解锁条件 |
|---|---|---|
| ApexCharts 5 -> 7 | NO-GO | 第三方分发的 OEM/Redistribution 许可边界尚无商业许可/法律确认；未试装、未接受条款。解锁后仍需真实路由视觉、bundle 和性能门禁 |
| ESLint 9 -> 10 | NO-GO | `eslint-plugin-react 7.37.5` 稳定 peer 排除 v10，且试验会触发两条现有 error 规则；等待插件兼容后独立迁移 |
| Zod 4.4.3 -> 4.5.4 | deferred | 隔离构建 raw chunk `81.81 KiB` 超过 `80 KiB` 预算；等待上游/拆包方案，不调宽预算掩盖回归 |
| TypeScript 6 -> 7 | deferred | 7.0.2 尚无 compiler API，且 typescript-eslint peer `<6.1.0`；等待正式工具链兼容 |
| UI `@types/node` 24 -> 26 | deferred | Node 26 当前不是仓库 LTS runtime；待 runtime major 决策 |
| VS Code `@types/node` 20 -> 26 | deferred | extension bundle target 为 Node 18，保留 20.x 类型是兼容边界 |
| VS Code `@types/vscode` 1.110 -> 1.134 | deferred / 治理债务 | extension engine 仍为 `^1.85.0`；先决定提高 engine 下限或将类型 pin 到最低支持面，避免类型允许旧宿主不存在的 API |
| Tauri `toml 0.9.12 -> 1.1.4` | deferred major | `0.9.12` 已是 `^0.9.8` 范围最新；跨根/Tauri major 的 desktop parser 迁移由治理例外跟踪，owner `desktop-platform`，到期 `2026-10-01` |
| `serde_yaml 0.9.34+deprecated` | deprecated crate，deferred migration | 这是 0.9 系列最终版本，不存在安全的同 crate 版本升级；替换实现会新增依赖并改变序列化边界，应另立迁移批次 |

代码/API 方面，AES-GCM 暴露的 deprecated generic-array 调用已在 0.11 迁移中消除；comfy-table 8 的删除 API、fs4 1 的错误类型、syn 3 宏解析、TypeScript 6 配置与 jsdom 30 原生事件行为均已完成相应迁移和回归。Tauri dialog 生成 schema 中 `ask`/`confirm` 只是 deprecated aliases，产品代码已使用 `message`，没有废弃 API 调用。

### 7.4 剩余证据边界

- 本轮已在 Windows 本机执行完整 `just ci`、debug MSI/NSIS 构建、两套 Rust coverage 与所有前端/扩展门禁；没有执行 GitHub hosted workflow，因此 Linux 预编译工具安装、macOS/Linux 打包和远端 Actions 运行仍为 `UNVERIFIED`。
- Tauri audit 的 19 条 warning 是透明保留的传递生态风险，不等同于已验证可利用；仓库没有用 ignore 隐藏它们。
- 所有 NO-GO 项均保持原版本与锁图，未把许可证、peer、bundle 或宿主兼容问题误写成“升级成功”。
