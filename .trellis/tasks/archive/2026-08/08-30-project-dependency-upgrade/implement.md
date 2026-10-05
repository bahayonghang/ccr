# 全项目依赖审计与分批升级 — Implementation Plan

> 状态：用户已于 2026-08-30 明确批准，任务已进入实施。执行顺序固定为安全修复优先；每批稳定前不得进入下一批。

## Phase 0 — 刷新与基线（无产品版本变更）

- [x] 刷新 RustSec/npm/Bun/GitHub tag/crates.io 动态数据并更新研究表。
- [x] 记录 `rustc/cargo/node/bun/npm/just` 版本；本机为 Rust/Cargo 1.95.0、Node 26.7.0、Bun 1.4.0、npm 12.0.2、just 1.58.0；测试仍以仓库/CI pin 为权威。
- [x] 运行 `just dependency-governance-check`、`just fmt-check`，确认升级前工作树只有任务工件与为恢复基线新增的最小修复。
- [x] 基线 `just ci` 首先暴露 Windows GBK 输出和用户会话/进程测试隔离问题；均在任何依赖更新前定位、修复并独立复核。最终完整 `just ci` 13 阶段全绿（10:37.229）。

## Security Batch S0a — 根锁 yanked `chacha20` 定向刷新

- [x] 仅把根 `Cargo.lock` 的 `chacha20 0.10.0 -> 0.10.2` 及 checksum 纳入本批，未改变 direct manifest 或其他节点。
- [x] 根 Cargo audit 扫描 488 crates：0 vulnerability，且不再报告 yanked `chacha20 0.10.0`。
- [x] 聚焦：`just lint-strict`、`just test`、根 Cargo audit 全部通过；独立检查无 findings。
- [x] 完整：`just ci` 13 阶段全绿（11:55.465）。

## Security Batch S0b — Tauri `event-listener` unsound warning 定向刷新

- [x] 仅把 Tauri 独立锁图的 `event-listener 5.4.1 -> 5.4.2`、checksum 及不再需要的 `concurrent-queue 2.5.0` 删除纳入本批。
- [x] Tauri Cargo audit 不再命中 `RUSTSEC-2026-0221`；独立复核确认依赖路径与最小差异。
- [x] 聚焦：`just tauri-ci` 通过。
- [x] 完整：`just ci` 13 阶段全绿。

## Security Batch S0c — Tauri 锁 yanked `chacha20` 定向刷新

- [x] 仅把 Tauri 独立锁图的 `chacha20 0.10.0 -> 0.10.2` 及 checksum 纳入本批，未改变 direct manifest。
- [x] Tauri Cargo audit 不再报告 yanked `chacha20`；仍待 S2/S3 处理 `crossbeam-epoch` 与 `quick-xml` 共 3 项 vulnerability。
- [x] 聚焦：`just tauri-ci` 与 Tauri Cargo audit 验收通过；独立复核无 findings。
- [x] 完整：`just ci` 13 阶段全绿。

## Security Batch S1 — docs 低风险漏洞锁刷新

- [x] 仅把 `docs/bun.lock` 中 `nanoid 3.3.18`、`postcss 8.5.23`、`preact 10.28.2`、`rollup 4.59.0` 及 Rollup 必需平台条目纳入本批；VitePress 保持 `1.6.4`。
- [x] `bun audit` 从 8 findings 降为 0；冻结安装、docs audit/build、`just docs-check` 与独立复核全部通过。
- [x] 完整：`just ci` 13 阶段全绿。

## Security Batch S2 — Tauri `crossbeam-epoch` 定向锁修复

- [x] 仅更新 Tauri 独立锁图的 `crossbeam-epoch 0.9.18 -> 0.9.20` 及 checksum，无伴随节点升级。
- [x] 根锁 audit 为 0 vulnerability；Tauri 锁不再命中 `RUSTSEC-2026-0204`，仅剩 S3 的两项 `quick-xml` vulnerability。
- [x] 聚焦：`just tauri-ci`、Tauri Cargo audit 与独立复核通过。
- [x] 完整：`just ci` 13 阶段全绿。

## Security Batch S3 — Tauri 安全链与 2.11 patch family

- [x] 协调更新 `tauri 2.11.2 -> 2.11.5`、`tauri-build/codegen/macros 2.6.2 -> 2.6.3`、JS API `2.11.1` 与 CLI `2.11.4`；非必要 dialog/fs 插件保持 `2.6.0/2.4.5`，避免生成 ACL/schema 越界变化。
- [x] `tauri-utils 2.9.3 -> plist 1.10.0 -> quick-xml 0.41.0`，未使用 Cargo patch 强压版本。
- [x] 聚焦：`just tauri-ci`、`just frontend-check`、冻结安装、UI audit、最终 Windows debug MSI/NSIS 通过；release MSI/NSIS 仅在核心安全版本相同的中间插件图通过，未冒充最终图证据。
- [x] 安全：Tauri Cargo audit 扫描 677 crates，0 vulnerability；19 条 allowed warnings 另列 deferred；独立复核无 findings。
- [x] 完整：`just ci` 13 阶段全绿。

## Security Batch S4 — VS Code packaging security

- [x] 更新 `@vscode/vsce 3.7.1 -> 3.9.2`，原 vsce 传递安全链全部清空。
- [x] 本批只保留 vsce 必需的 package-lock 变化；direct `esbuild` 维持 `0.27.4`，审计从 13 项降至仅剩其 1 项 low。
- [x] 聚焦：`just vscode-ci`、51 项测试、lint/build/package 与 `.vsix` 生成通过；install-script 集合及授权未扩大；独立复核无 findings。
- [x] 安全：剩余 `GHSA-g7r4-m6w7-qqqr` 精确归属已计划的 S5，没有以例外放行普通更新。
- [x] 完整：`just ci` 13 阶段全绿。

## Security Batch S5 — `esbuild 0.27 -> 0.28` 安全/Breaking

- [x] 更新 direct `esbuild 0.27.4 -> 0.28.2`；因 `tsx 4.21.0` 约束 `~0.27` 会保留嵌套漏洞解析，同批最小升级到 `tsx 4.23.13`，最终仅单一 `esbuild 0.28.2`。
- [x] 聚焦：VS Code lint、51 项测试、build/package/VSIX、`just vscode-ci` 与 Windows 非网络 watch 通过；现有脚本无 `--serve/servedir`，无需迁移。
- [x] 安全：`npm audit --package-lock-only` 为 0，GHSA-g7r4-m6w7-qqqr 不再命中；install-script 集合/授权未扩大；独立复核无 findings。
- [x] 完整：`just ci` 13 阶段全绿；安全队列稳定，允许进入普通 Low 更新。

## Ordinary Batch L1 — Rust 同兼容线 patch/minor

- [x] 定向更新 13 个同兼容线候选：`arc-swap/filetime/indexmap/libc/lru/moka/open/quote/rayon/tracing-appender/tracing-subscriber/uuid/whoami`；未触碰 `aes-gcm/argon2/comfy-table/fs4/syn/sysinfo` 或 S3 Tauri family。
- [x] 已逐项解释根/Tauri 两锁解析；Tauri `indexmap 1.9.3` 为 `schemars 0.8.22` 的独立传递图，其余目标一致或明确为 Tauri-only。
- [x] 聚焦：dependency governance、`just lint-strict`、`just test`、`just tauri-ci`、两套 Cargo audit 全部通过；独立复核无 findings。
- [x] 完整：`just ci` 13 阶段全绿。

## Ordinary Batch L2 — UI 同兼容线 patch/minor

- [x] 升级 12 组 manifest range 已允许的 CodeMirror/Iconify/DOMPurify/React DOM types/TS-ESLint/Vite React/Vitest/coverage/Playwright 兼容版本。
- [x] exact pins 与 major/breaking 候选保持排除；S3 的 Tauri exact pins 未重复改动。
- [x] 聚焦：冻结安装、UI audit 0、`just frontend-check`、701 项 coverage 测试和 bundle budget 通过；独立复核无 findings。
- [x] 测试失败修复：定位升级前已有入口 bundle 超预算，按首次使用懒加载 About/Confirm modal chain，使入口 `259.88/82.26 -> 217.17/68.33 KiB`；补充 chunk reject 收敛，Confirm 返回 `false`、About 局部隔离且不伪重试。
- [x] 完整：`just ci` 13 阶段全绿。

## Ordinary Batch L3 — VS Code 非 breaking minor + Actions SHA

- [x] 锁刷新 `@types/node 20.19.37 -> 20.19.43`、`smol-toml 1.6.1 -> 1.8.0`；`tsx 4.23.13` 已由 S5 安全闭环完成，本批保持。
- [x] 更新 11 处 `dtolnay/rust-toolchain@stable` 与 2 处 `softprops/action-gh-release@v3` pinned SHA，live refs 验证一致；其他 Actions 无 diff。
- [x] `@types/vscode`、vsce、esbuild、TypeScript 保持；install-script 边界未扩大。
- [x] 聚焦：npm audit 0、51 项测试/VSIX、workflow validator、`just vscode-ci`、`just ci-governance-check` 通过；同步既有治理规范计数 `52 -> 43`。
- [x] 完整：`just ci` 13 阶段全绿。

## Ordinary Batch L4 — docs 单一包管理器治理

- [x] 根 `just docs` 改为 Bun frozen install + build，与 `docs-check`/README 一致；删除 ignored 历史残留 `docs/package-lock.json` 并新增拒绝复发门禁。
- [x] 全仓搜索确认 docs npm install/lock authority 仅剩禁止性合同与负向测试；`docs/bun.lock` 为唯一权威，VS Code npm 流程未误伤。
- [x] 聚焦：Bun audit 0/build、`just docs/docs-check`、9 项 drift 测试、version/dependency/CI governance 全部通过；独立复核无 findings。
- [x] 完整：`just ci` 13 阶段全绿。

## Ordinary Batch M1 — UI exact-pin 同 major 更新

- [x] 接受四组 exact pins：TanStack Query/Core `5.102.8`、React Hook Form `7.87.0`、React Router `8.3.1`、Testing Library `16.3.3`；Tauri API/CLI 已由 S3 完成并保持。
- [x] Zod `4.5.4` 经隔离 A/B 构建确认单 chunk `81.81 KiB` 超过 `80 KiB` raw 预算，回退 `4.4.3`（`66.41/17.96 KiB`）并 deferred，未调预算。
- [x] 聚焦：冻结安装、audit 0、`just frontend-check`、701 项 coverage、bundle budget、`just tauri-ci` 全部通过；独立复核无 findings。
- [x] 完整：`just ci` 13 阶段全绿。

## Ordinary Batch M2 — 运行时/工具链（每项独立子批）

- [x] M2a Rust：开发/普通 CI pin `1.95.0 -> 1.98.0`；修复 4 个新 Clippy 点，新增 required `workspace-msrv` lane 使用 `1.95.0`，14 个 manifest 的 `rust-version = 1.95` 不变；双工具链与完整 `just ci` 通过。
- [x] Bun `1.3.10 -> 1.4.0`：UI/docs/workflow pin 与治理契约同步；两套 frozen install 无 lock 漂移，修复 lazy-shell 测试 effect 时序竞态和 ts-rs bindings 并发生成竞态；独立复核与完整 `just ci` 通过。
- [x] Node 保持 24 LTS，`24.18.0 -> 24.20.0`：同步 3 个 hosted CI pin，并将治理精确绑定到 `actions/setup-node` 步骤；Node 26 deferred，本机 Node 24.20.0 运行仍为 hosted `UNVERIFIED`。
- [x] 每个工具链子批均单独运行 `just ci`（Rust 1.98、Bun 1.4.0、Node 24.20.0 均全绿）。

## Ordinary Batch H1 — Rust crypto family

- [x] `aes-gcm 0.10.3 -> 0.11.1`：完成 `aead 0.6` / hybrid-array API 迁移并移除 deprecated 路径；三条由 test-only 0.10.3 harness 生成的固定向量在 0.11.1 上兼容解密，独立复核、两锁 audit 0 与完整 `just ci` 通过。
- [x] `argon2 0.5.3 -> 0.6.0` 独立升级：生产 KDF API无需迁移，固定 Argon2id/V0x13/cost/output/salt 语义；共享 0.5.3 KDF 与两条旧信封兼容测试通过。
- [x] 增加/运行旧密文解密、格式长度不变量、错误 key/nonce/ciphertext/tag、非法 KDF 参数与短盐回归；envelope version/KDF 参数语义不变。
- [x] 聚焦：crypto crates、Rust 1.95、root/Tauri consumers、两锁 audit 0 全部通过。
- [x] 完整：AES-GCM 与 Argon2 两个独立子批均通过 `just ci`。

## Ordinary Batch H2 — comfy-table 8

- [x] `comfy-table 7.2.2 -> 8.0.0`：30 个 `load_preset` 全部迁移，34 个构造责任点收敛到 ASCII/UTF-8 两个共享 helper，并显式保留旧 `...` 截断符。
- [x] 精确 UTF-8 边框/列序、12 列窄宽、颜色、非 TTY、`NO_COLOR` 与 Windows 分支回归通过；关键 v7/v8 样例 byte-identical。
- [x] 聚焦：CLI 347、TUI 219、public commands 109、public API、Rust 1.95、双锁 audit 0 全部通过。
- [x] 完整：`just ci` 13 阶段全绿。

## Ordinary Batch H3 — 其他 Rust breaking（逐项独立）

- [x] 根 `fs4 0.13.1 -> 1.1.0`，与 Tauri 统一；适配新 `TryLockError` API并以真实 Windows 子进程握手验证竞争/释放重获，锁、备份、原子写/CAS 与完整 `just ci` 通过。
- [x] command macro `syn 2.0.117 -> 3.0.4`：合法 token 展开与历史返回类型判定保持，补充宏单测/独立 compile-fail/span 诊断；Rust 1.95、Tauri inventory/audit 与完整 `just ci` 通过。
- [x] `sysinfo 0.38.4 -> 0.39.6`：根/Tauri 统一单一版本，受控 Windows 子进程与 fake backend 覆盖 identity/reuse/Term→Kill/失败/消失语义；Rust 1.95、双锁 audit 0 与完整 `just ci` 通过。
- [x] fs4、syn、sysinfo 每项均独立聚焦检查、安全复扫和 `just ci`，未合并为不可归因的大批次。

## Ordinary Batch H4 — 前端 breaking（逐项独立、有明确 go/no-go）

- [x] ESLint 9 -> 10：NO-GO/deferred。`eslint-plugin-react 7.37.5` 最新稳定 peer 仍排除 ESLint 10，且仓库两条 React 规则为 error；禁止 peer override/删规则。解锁后另需修复两处新 `no-useless-assignment`。
- [x] jsdom `26.1.0 -> 30.0.1`：删除 7 处旧 PointerEvent stub，保留并规范仍必需的窄 shim；新增原生事件/cookie/CSSOM/layout runtime 合同，704 项 smoke/coverage、audit 0 与完整 `just ci` 通过。
- [x] ApexCharts 5 -> 7：NO-GO/deferred。官方 `apexcharts 7.0.0` 与当前解析的 `5.16.0`、`react-apexcharts 2.1.1` 都声明安装即接受适用许可；向第三方分发可交互嵌入应用被列入付费 OEM/Redistribution 范围。未取得用户的商业许可/法律确认前未试装、未改锁、未接受条款，也未伪造 v7 视觉证据。
- [x] ApexCharts 技术预审：v7 仍公开 `core/area/line/bar/donut/heatmap/features/legend/dist/*`，wrapper peer `apexcharts >=5.10.1` 与 React 19 相容，Node 24.20 满足 v7 engine；仓库未使用 v7 拆出的 9 个 opt-in feature，也未使用已删除的 `borderRadiusWhenStacked`，现有 bar data labels 均禁用。但解锁后仍必须独立完成 CSS/DOM、真实 `/usage` 与 observer 路由视觉、bundle budget、更新/主题切换性能回归。
- [x] TypeScript `5.9.3 -> 6.0.3`：移除 deprecated `baseUrl`、显式收窄 Node types，并保持 paths/strict/target/moduleResolution/declaration 语义；UI 704 项、VS Code 51 项、Tauri 509 项、bundle budget、双前端审计、独立复核与完整 `just ci` 通过。TS7 因无 compiler API 且 typescript-eslint peer `<6.1.0` deferred。
- [x] UI `@types/node 22.20.1 -> 24.13.3` 对齐 Node 24 LTS、`globals 16.5.0 -> 17.11.0`；VS Code `@types/node 20.19.43` 因 Node 18 bundle target / VS Code 1.85 宿主边界保持。独立复核发现并修复 `ES2022.Array` 的虚假运行时承诺，将 2 个生产与 3 个测试 `.at(-1)` 改为 ES2020 等价索引。
- [x] 每个接受的前端 breaking 子批均独立通过 `just frontend-check`、coverage、bundle/相关检查与 `just ci`；ESLint 10、ApexCharts 7 为前置条件未满足的 NO-GO，未试装或伪造验证。

## Final-rescan batches — 扫描遗漏的独立升级

- [x] `just 1.57.0 -> 1.58.0`：同步 7 个 `cargo install` pin、2 个 Linux 预编译资产与治理规范；官方资产 SHA256 已核验。聚焦治理检查和独立复核无 findings；完整 `just ci` 通过（`1788130632_just_ci.log`）。
- [x] Tauri `tauri-plugin-dialog 2.6.0 -> 2.7.2`，连同其唯一消费者所需的 `tauri-plugin-fs 2.4.5 -> 2.5.1`：实际代码已使用 `message`，未命中 2.7 标记 deprecated 的 `ask`/`confirm` aliases；生成 ACL schema 差异与插件声明一致。`just tauri-ci`、`just frontend-check`、debug MSI/NSIS 构建、Rust 1.95、audit 与独立复核均通过；完整 `just ci` 通过（`1788132111_just_ci.log`）。
- [x] `cargo-llvm-cov 0.8.7 -> 0.9.0`：同步两条 hosted workflow 的版本、官方 Linux GNU 资产 SHA256 与治理规范。项目不使用唯一 breaking 的 `--show-missing-lines` 文本接口；以仓库隔离安装的 0.9.0 运行 root/Tauri JSON coverage 均通过（`1788132386_just_coverage-rust.log`、`1788132545_just_coverage-tauri.log`），独立复核无 findings；完整 `just ci` 通过（`1788132984_just_ci.log`）。

## Final integration gate

- [x] 重新运行全生态 outdated + audit；所有 remaining outdated 均已归入有明确解锁条件的 deferred/NO-GO，详见最终复扫报告。
- [x] 最终双锁 Cargo audit 为 0 vulnerability（Tauri 透明保留 19 条传递 warning、无 ignore）；UI/docs/VS Code audit 均为 0；最后一次完整 `just ci` 通过（`1788133903_just_ci.log`）。
- [x] `just version-check`、`git diff --check`、`git status --short`、5 个 authority locks 与未跟踪文件审计通过；未发现计划外工件。
- [x] `research/dependency-scan-2026-08-30.md` 和本文件记录每批目标版本、迁移、测试、失败修复、deferred 与 remaining `UNVERIFIED`；未 commit、push 或 archive。
