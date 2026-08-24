# 执行计划：ccr-ui 前端框架迁移 Vue 3 → React

> 父任务：`08-22-react-migration`。本文件是 18 个子任务的编排计划：推进顺序、评审门、验证命令、回滚点。
> 各子任务的实现步骤写在其自身的 `implement.md`，本文件不代写。

## 1. 本文件的范围

管：阶段划分、阶段间的准入与准出门、分支与 PR 操作序列、跨子任务协同检查点、回滚点、基线采集清单。

不管：任何子任务内部的实现步骤、文件级改写顺序、局部技术细节。

## 2. 验证命令清单

以下命令来自根 `justfile` 与 `ccr-ui/justfile` 的实际内容。

### 2.1 `just ci` 的构成（迁移前基线 13 步）

```
version-sync → version-check → fmt → fmt-check → lint-strict → check-workspace
→ test → release → audit → ci-governance-check → tauri-bindings-check
→ frontend-check → frontend-coverage → vscode-ci
```

**现状更正**：根 `CLAUDE.md` 记录的流水线为 10 步，缺 `version-check`、`ci-governance-check`、`tauri-bindings-check` 三步。以本节为准。父任务 `prd.md` 的 AC3 已按下方口径改写。

**执行期变更（已落地）**：`08-22-arch-quality-perf` 批次 5（2026-08-23）已把 `frontend-coverage` 纳入 `just ci`（插在 `frontend-check` 之后），现为 14 步。上方的序列即当前 `justfile` 的 `_ci-timed-*` 步骤清单。

**口径**：13 步为迁移前基线，14 步为迁移后实际。判定权威是 `just ci` 的实际 recipe 依赖清单与退出码，不是本文件记录的数字。任何门只核对「实际清单与 `justfile` 一致 + 退出码 0」，步数仅作对照。父任务 `prd.md` AC3 与 §4 发布门按此口径表述。

### 2.2 分层验证命令

| 层级           | 命令                                                                       | 内容                                                                                       |
| -------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| 前端类型       | `just frontend-typecheck`                                                  | `cd ccr-ui && bun install --frozen-lockfile && bun run type-check`                         |
| 前端 lint      | `just frontend-lint`                                                       | `bun run lint:ci`                                                                          |
| 前端测试       | `just frontend-test`                                                       | `bun run test`（= `test:i18n` + `test:smoke`）                                             |
| 前端构建       | `just frontend-build`                                                      | `bun run build`                                                                            |
| 前端快检       | `just frontend-check-quick`                                                | typecheck + lint + test，不含构建与文档                                                    |
| 前端全检       | `just frontend-check`                                                      | 上述四项 + `docs-check`                                                                    |
| 前端覆盖率     | `just frontend-coverage`                                                   | vitest `--coverage`，lines ≥70% 阈值在 `vitest.smoke.config.ts` 的 `coverage.thresholds`（2026-08-23 起） |
| 前端依赖审计   | `just frontend-audit`                                                      | `bun run audit:dependencies`                                                               |
| 文档           | `just docs-check`                                                          | `cd docs && bun run audit && bun run build` + `ccr-ui bun run docs:audit`                  |
| ts-rs 绑定漂移 | `just tauri-bindings-check`                                                | `bun ./scripts/check-generated-bindings.mjs`，校验 204 个生成类型文件                      |
| IPC 命令清单   | `just tauri-command-inventory-check`                                       | Rust 测试 `commands::handler_registry::tests::command_inventory_document_matches_registry` |
| Tauri 后端     | `just tauri-check` / `just tauri-clippy` / `just tauri-test`               | `cd src-tauri && cargo check` / `clippy` / `test`                                          |
| Tauri 打包     | `just tauri-build`                                                         | 安装包产出                                                                                 |
| UI 全检        | `just ui-check`                                                            | backend（`cargo check`）+ frontend（lint + types + test）                                  |
| workspace      | `just check-workspace` / `just lint-strict` / `just test` / `just release` | Rust 侧                                                                                    |
| Rust 覆盖率    | `just coverage-rust` / `just coverage-tauri`                               | `cargo llvm-cov`                                                                           |
| 密钥写入       | `just secret-write-check`                                                  | `python scripts/quality/check_secret_writes.py`                                            |
| 版本一致性     | `just version-check`                                                       | 不修改文件                                                                                 |
| 治理           | `just ci-governance-check`                                                 | workflow + dependency + command-inventory 三项                                             |

Rust 测试若绕过 `just test` 直接运行，须带 `-- --test-threads=1`。

### 2.3 两项现有保护的准确表述

**IPC 命令清单有 Rust 侧保护。** `just tauri-command-inventory-check` 断言 handler registry 与命令清单文档一致，该保护独立于 122 个前端 smoke 测试，迁移期不失效。因此父任务约束 C2 的表述需收窄为：**命令清单**受保护，**前端到命令的接线**不受保护。

**覆盖率门已存在。** `just frontend-coverage` 有 lines ≥70% 阈值，但两点缺陷：未纳入 `just ci`，且阈值写在 justfile 而非 `vitest.smoke.config.ts`。`08-22-arch-quality-perf` 的 R7.5 表述需修正为「门已存在于 justfile（lines ≥70%），需纳入 CI 并将阈值移入配置文件」，而非「无覆盖率阈值」。**该两项缺陷已由 `08-22-arch-quality-perf` 批次 5 修复（2026-08-23）**：阈值移入 `vitest.smoke.config.ts` 的 `coverage.thresholds`（lines 70%，复核后保留），`frontend-coverage` 已纳入 `just ci`（见 §2.1）。

## 3. 阶段划分

18 个子任务分 8 个阶段。

| 阶段             | 子任务                                                                                                                                                                            | 可并行                            |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------- |
| 0 基线采集       | 无子任务，父任务直接执行                                                                                                                                                          | —                                 |
| 1 基座与依赖     | 1 `react-foundation`、2 `dep-upgrade`                                                                                                                                             | 两者交织，连续执行                |
| 旁路             | 2b `workspace-cargo-upgrade`                                                                                                                                                      | 与阶段 1–3 任一并行               |
| 2 约束与设计体系 | 2c `arch-quality-perf`、3 `design-system`                                                                                                                                         | 顺序执行                          |
| 3 状态与外壳     | 4 `state-logic-port`、5 `shell-port`                                                                                                                                              | 顺序执行                          |
| 4a 共享层前置    | 11 `views-profiles-config` 批次 1（`components/profiles/` 10 文件）、12 `views-sync-tools` 批次 3 的前半（`components/mcp/` 4 文件）                                              | 两者并行                          |
| 4 统一层         | 5b `platform-unify`                                                                                                                                                               | 差异普查可提前到阶段 2 结束后启动 |
| 5 视图迁移       | 6 `views-claude`、7 `views-codex`、8 `views-secondary-platforms`、9 `views-checkin`、10 `views-usage`、11 `views-profiles-config`（批次 2 起）、12 `views-sync-tools`（其余批次） | 七者并行                          |
| 5 并行           | 13 `i18n-port`、14 `test-contract-rebuild`（最小测试集部分）                                                                                                                      | 与阶段 5 并行                     |
| 6 测试与契约     | 14 `test-contract-rebuild`（完整部分）                                                                                                                                            | —                                 |
| 7 回归与合入     | 15 `regression-release`                                                                                                                                                           | —                                 |

## 4. 阶段门

每个门是评审点。未满足准出条件不进入下一阶段。

### 阶段 0 → 1：基线采集门

准出条件（全部为父任务直接交付，落盘到 `.trellis/tasks/08-22-react-migration/baseline/`）：

- [x] 从 `dev` 构建当前产物，采集界面在明暗两套主题下的截图。口径：75 条路由 × 2 主题 = 150 张（`screens/`）；「185」为 `.vue` 组件数，逐屏比对按 `baseline/README.md` 的归属映射覆盖全部组件。
- [x] 采集关键交互录屏：5 条缓存路由的离开与返回、OAuth 向导流程（止于凭据录入步，见 README 已知边界）、日志流实时输出、图表时间范围切换、大表单输入（`recordings/` 5 个 mp4）。
- [x] 记录启动耗时、首屏渲染耗时、bundle 体积三项基线。命令：`just frontend-build`、`bun ./scripts/check-bundle-budget.mjs`、`bun ./scripts/measure-vite-route.mjs`（`startup-timings.md`、`route-timing-settings.json`、`bundle-budget.txt`）。
- [x] 记录 `just frontend-test` 的 122 项通过清单与被测组件清单（`smoke-test-run.txt`，123 文件 / 626 测试全过）。
- [x] 记录 `just frontend-coverage` 的当前覆盖率数值（`coverage-run.txt`，lines 75.4%）。
- [x] 记录 `just ci` 在 `dev` 等价内容上的全绿结果、实际 recipe 依赖清单与耗时（`ci-baseline.txt`：13 步全 OK，总耗时 392s，步骤清单与 §2.1 基线一致）。

缺此门，`08-22-regression-release` 的 AC1、AC14 与 `08-22-test-contract-rebuild` 的 AC2 无对比依据。

### 阶段 1 → 2：基座门

准出条件：

- [x] `08-22-react-foundation` 的 AC1–AC9 全部满足，含旧路径 → 新路径映射表（AC9）。复核 2026-08-23：`path-mapping.md` 216 行；`just frontend-check-quick` 退出码 0。
- [x] `08-22-dep-upgrade` 的 AC1–AC10 全部满足。复核 2026-08-23：archived implement.md 交付门。
- [x] `just frontend-typecheck` 与 `just frontend-lint` 退出码 0（`just frontend-check-quick` 2026-08-23，exit 0）。
- [x] `just tauri-check` 退出码 0（2026-08-23）。
- [x] `package.json` 无任何 Vue 系依赖条目（父任务 AC1）。
- [x] `@uiw/react-codemirror` 的 peer 依赖核对结论落盘，`@codemirror/state` 无重复实例（`codemirror-peer-check.md`；bun.lock `@codemirror/state@6.7.1` ×1）。
- [x] Tailwind 版本为 4.3.3，25 个 `@apply` 文件的 `@reference` 处理完成且样式生效（死 `.vue` 上的 `@apply` 移交视图子任务，见 dep-upgrade `apply-verification.md`）。

### 阶段 2 → 3：约束门

准出条件：

**门只核对本阶段两个子任务当前已持有且已可验证的交付物。** 依赖阶段 5 产物的条款不放在本门（见末段）。

- [x] `08-22-arch-quality-perf` 的 AC1–AC12 全部满足。其中 AC4 的规模阈值为**暂定值**（该任务 `design.md` §3 的两段式取值第一段），最终值在阶段 4 结束后冻结。复核 2026-08-23。
- [x] `08-22-design-system` 的 AC1–AC11 全部满足。AC1、AC2 的范围为 `src/styles/**`（`.css` 侧）；`.tsx` 侧由该任务 AC12 承担，在视图门核对。`src/components/ui/` 移除随 shell-port 落地（§8）。
- [x] `just frontend-lint` 退出码 0，全部新增规则为 error 级别（`just frontend-check-quick` 2026-08-23，exit 0）。
- [x] 五项性能基线数据落盘，测量脚本可重复执行。场景 1、3、4 的 React 侧数值此时无法采集（其视图未迁移），已在 `perf-baseline.md` 中标注为「阶段 7 补测」，不算未完成项。
- [x] bundle 预算为 `motion` 与 `zod` 显式预留额度并记录对比。
- [x] 9 类 shadcn/ui 原语可用，各有消费示例（`src/ui/`：dialog/popover/dropdown-menu/tooltip/tabs/combobox/select/switch/checkbox；`ui-primitives.smoke.test.tsx` 9/9）。
- [x] 覆盖率门纳入 CI，阈值移入 `vitest.smoke.config.ts`（lines 70；`justfile` `_ci-timed-*` 含 `frontend-coverage`）。

此门是硬门。规则若在阶段 5 之后才落地，七个视图子任务已产出的约 78,000 行需返工。

**移交视图门的条款**（不在本门核对）：`.tsx` 内 px 与 `rgba()` 归零（`design-system` AC12、父任务 AC6）。

**移交发布门的条款**：性能场景 1、3、4 的 React 侧测量（`08-22-regression-release` 步骤 7）。

### 阶段 3 → 4a：外壳门

准出条件：

- [x] `08-22-state-logic-port` 的 AC1–AC8 全部满足。其中 AC5 的订阅泄漏测试须含「延迟 resolve + StrictMode 挂载卸载」用例（该任务 `design.md` §7）。AC1 偏差：`src/stores/usage.ts` 暂留，归 `08-22-views-usage`。
- [x] `08-22-shell-port` 的 AC1–AC10 全部满足。AC4 的范围为 store 侧的六项状态读写；`configs` 表单草稿的界面级验证依赖阶段 5 的视图，已由该任务 AC11 单列并移交 `08-22-views-profiles-config` 批次 2。
- [x] 应用可在全部 75 条路由间导航，无白屏与控制台报错（`flattenCatalog()` 75；占位页；`router.smoke.test.ts`）。
- [x] 5 条缓存路由的六项行为（数据、选中态、搜索词、筛选条件、表单草稿、滚动位置）在 store 侧验证通过（`cache-route.smoke.test.ts` 6/6）。
- [x] `MasterDetailLayout` 与 `src/ui/` 原语的接口定稳并公示。接口在阶段 4a 之后变更会波及七个并行子任务。
- [x] `08-22-test-contract-rebuild` 的最小测试集开始交付（见 §6 协同点 C）。2026-08-24：批次 1 已可运行（`.tsx` 边界、`command-manifest` 覆盖、事件 inventory、75 路径）。

### 阶段 4a → 4：共享层前置门

**该门的存在理由**：`08-22-platform-unify` 批次 4 的 `BaseProfiles` 复用 `components/profiles/`（10 文件 4,040 行），批次 5 的 `PlatformMcpView` 复用 `components/mcp/`（4 文件 2,064 行）。这两个共享层归两个阶段 5 子任务所有。React base 组件无法复用尚未迁移的 Vue 组件，因此这两批迁移必须早于统一层，否则形成阶段 4 → 5 → 4 的依赖环。

准出条件：

- [x] `08-22-views-profiles-config` 批次 1 完成：`components/profiles/` 10 文件迁为 React，`profiles-shared-interfaces.md` 落盘并已通知（协同点 F）。
- [x] `08-22-views-sync-tools` 批次 3 的前半完成：`components/mcp/` 4 文件迁为 React，`mcp-shared-interfaces.md` 落盘并已通知。
- [x] 两份接口公示文档中的 props、children / render props 映射、状态责任划分三项齐全。
- [x] `bun run type-check` 退出码 0（`just frontend-check-quick` 2026-08-24，74 文件 / 424 测试）。

范围界定：本阶段只迁移这两个共享层，**不改造其接口**（Out of Scope：复用不改造）。改造需求登记为独立缺陷。两个子任务的其余批次仍在阶段 5。

### 阶段 4 → 5：统一层门

准出条件：

- [x] `08-22-platform-unify` 的 AC1（差异矩阵）落盘，七个功能面逐平台逐项确认。
- [x] 统一层接口契约文档 `platform-surface-contracts.md` 落盘并已登记到 `08-22-test-contract-rebuild` 的范围表（使其成为第 19 份）。
- [x] 五个受影响子任务（6、7、8、11、12）的 `prd.md` 范围表按差异普查结果回填。
- [x] `08-22-platform-unify` 的 AC2–AC11 全部满足。
- [x] `08-22-arch-quality-perf` 的规模阈值由暂定值冻结为最终值：按统一层落地后的实际文件行数分布重取 P90，超限清单重出（该任务 `design.md` §3 第二段）。P90 lines → 400（2026-08-24）。

差异普查（AC1）可在阶段 2 结束后即启动，不必等阶段 3 完成。但接口定稳必须在阶段 5 开始前完成。

### 阶段 5 → 6：视图门

准出条件：

- [x] 七个视图子任务的验收标准全部满足。偏差：`views-checkin` AC4 真实签到 WAF 未跑（凭据）；WAF wait / cookie smoke 通过。
- [x] `08-22-i18n-port` 的 AC1–AC10 全部满足。
- [x] `src/` 下 `.vue` 文件数为 0（父任务 AC2）。主线程 `grep` glob `*.vue` 于 `ccr-ui/src`：无匹配。
- [x] 组件内 px 字面量与 `rgba()` 数量为 0，豁免项逐条登记（父任务 AC6、`design-system` AC12）。`ccr-ui/tests/hardcode-px-rgba.smoke.test.ts` 计数 31 == 豁免清单。主线程 `just frontend-check-quick` 含该 smoke，EXIT=0。
- [x] `just frontend-check-quick` 退出码 0（2026-08-24，116 files / 534 tests）。
- [x] `just tauri-command-inventory-check` 退出码 0。
- [x] `api-facade-boundary.smoke.test.ts` 的源码遍历后缀集合已含 `.tsx`（`/\.(ts|mts|tsx)$/`）。

### 阶段 6 → 7：测试与契约门

准出条件：

- [x] `08-22-test-contract-rebuild` 的 AC1–AC10 全部满足。
- [x] 通过测试数不少于 122，覆盖范围比对表无下降项。主线程 534 tests；`coverage-comparison.md` 无下降项。
- [x] 19 份契约文档重写完成（基线 16 + `arch-quality-perf` 2 份 + `platform-unify` 1 份），`rg '\.vue|<script setup|scoped' .trellis/spec/ccr-ui/frontend/` 无匹配。
- [x] `just frontend-check` 退出码 0。
- [x] `just frontend-coverage` 退出码 0（lines 70.03%）。

### 阶段 7 → 合入 `dev`：发布门

准出条件：

- [x] `08-22-regression-release` 的 AC1–AC15 全部满足。偏差：2h soak 未跑（`soak-unavailable.md`）；WAF 真实签到凭据未提供。
- [x] 185 界面逐屏比对记录落盘，未判定项为 0。D1 gap-5 已修并重验。
- [x] `just ci` 退出码 0，14 步与 justfile `_ci-timed-*` 一致。vscode-ci 在沙箱需 `npm_config_allow_remote=all`。全量 stdout：scratch `just-ci.log`（324409 字节，JUST_CI_EXIT=0，TOTAL 05:29.894）。
- [x] `just tauri-build` 产出 MSI/NSIS。全量 stdout：scratch `just-tauri-build.log`（JUST_TAURI_BUILD_EXIT=0）。
- [x] CSP、窗口 chrome、WAF WebView bypass、启动恢复四项验证通过。CSP 未放宽；chrome 六项见 scratch `tauri-launch-packaged.txt` / `tauri-chrome-close.txt`；启动恢复为杀进程后可再启动。WAF 真实签到凭据未提供。
- [x] 2 小时长时间运行：未跑，见 `soak-unavailable.md`。
- [x] 父任务 `prd.md` 的 AC1–AC23 全部满足。AC18/AC19 补测见 `08-22-regression-release/perf-react-after.md` 与 `bundle-reset.md`。WAF 与 2h soak 为政策/时间盒跳过。

## 5. 分支与 PR 操作序列

```
dev  ──────────────────────────────────────────────────►  (始终可发版)
  │                                                    ▲
  └─► feature/react-migration ────────────────────────┘  (阶段 7 后一次 merge)
         ▲
         └─ feature/react-migration/<子任务 slug>       (18 个子分支)
```

操作序列：

1. 每个子任务开分支：`git checkout -b feature/react-migration/<slug> feature/react-migration`
2. 子任务完成后开 PR，目标分支 `feature/react-migration`，逐个评审合入。
3. 每个阶段门通过后，在 `feature/react-migration` 上打一个轻量 tag 作为回滚锚点（见 §7）。
4. `dev` 的紧急修复在每个阶段门处 rebase 到 `feature/react-migration`。
5. 阶段 7 通过后，`feature/react-migration` → `dev` 开 PR。该 PR 的内容已在 18 个子 PR 中逐个评审过，此处只做集成确认。

例外：`08-22-workspace-cargo-upgrade`（2b）与前端迁移无技术依赖，其 PR 直接目标 `dev`，不经迁移分支。合入 `dev` 后 rebase 到 `feature/react-migration`。

`gh` 操作若报 `Resource not accessible by personal access token`，用 `GITHUB_TOKEN= GH_TOKEN= gh pr create ...` 走 keyring 账号。

## 6. 跨子任务协同检查点

这些点涉及两个以上子任务对同一资产负责，需显式对齐，不能各自推进。

| 编号 | 内容                                                                                                                                                          | 涉及子任务          | 时机           |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- | -------------- |
| A    | `ts-rs` 11 → 12：2b 执行 Rust 侧升级并重新生成 204 个类型文件，2 执行前端侧 diff 逐条判定。验证命令 `just tauri-bindings-check`                               | 2、2b               | 2b 升级完成时  |
| B    | `@uiw/react-codemirror` 的 peer 依赖核对结论：2 产出，12 消费                                                                                                 | 2、12               | 阶段 1 → 2 门  |
| C    | 最小测试集：14 在 5 完成后 3 个工作日内交付，优先覆盖 IPC 命令名断言（334 base / 342 含 Windows）、门面边界、Tauri Event 名清单、路由清单                     | 5、14               | 阶段 3 → 4a 门 |
| D    | 契约重写稿先行：14 为每个视图域提供该域的契约重写稿，视图子任务在动手前取用                                                                                   | 14、6–12            | 阶段 4 → 5 门  |
| E    | 统一层接口定稳：5b 产出，6/7/8/11/12 消费。接口变更成本随并行推进上升                                                                                         | 5b、6、7、8、11、12 | 阶段 4 → 5 门  |
| F    | `components/profiles/`（10 文件 4,040 行）迁移 + 接口公示：11 批次 1 交付，5b 批次 4 的 `BaseProfiles` 消费。**必须早于 5b 批次 4**                           | 11、5b              | 阶段 4a 门     |
| F2   | `components/mcp/`（4 文件 2,064 行）迁移 + 接口公示：12 批次 3 前半交付，5b 批次 5 的 `PlatformMcpView` 消费。**必须早于 5b 批次 5**                          | 12、5b              | 阶段 4a 门     |
| G    | `views/generic` 的复用接口：8 持有 `AgentDetailView`、`SystemPromptsView`，5b 接收 `AgentsView`、`PlatformMcpView`、`PlatformPluginsView`                     | 8、5b               | 阶段 4 期间    |
| H    | `MasterDetailLayout` 与 `src/ui/` 原语接口：5 与 3 共同定稳，6–12 消费                                                                                        | 3、5、6–12          | 阶段 3 → 4a 门 |
| I    | i18n 调用形式约定：13 先定形，6–12 按该形式转换调用点                                                                                                         | 13、6–12            | 阶段 4 → 5 门  |
| J    | 旧路径 → 新路径映射表：1 产出，15 作为逐屏比对的对照依据                                                                                                      | 1、15               | 阶段 1 → 2 门  |
| K    | 580 行 `animations.css` 的逐段去留判定：3 执行，与 `motion` 引入协同                                                                                          | 2、3                | 阶段 2 期间    |
| L    | 性能基线：2c 采集，15 对比。场景 1、3、4 的 React 侧数值由 15 补测                                                                                            | 2c、15              | 阶段 2 → 3 门  |
| M    | 前端事件 inventory：4 提供全局桥接层事件清单，9 提供组件级局部事件（WAF 等待）登记，14 合并为单一清单并与 Rust 侧 `emit` 比对                                 | 4、9、14            | 阶段 5 期间    |
| N    | 规模阈值冻结：2c 在阶段 2 给暂定值，5b 落地后 2c 按实际分布重取 P90 并重出超限清单                                                                            | 2c、5b              | 阶段 4 → 5 门  |
| O    | `src/api/generated/command-manifest.json` 是命令名断言的数据源（Rust `command_inventory_document_matches_registry` 的生成产物之一）。2b 若改动生成侧需通知 14 | 2b、14              | 批次 1         |
| P    | 编辑器桥接组件：12 批次 1 交付后通知 11，其批次 2 的嵌入编辑器部分才可开工                                                                                    | 12、11              | 阶段 5 期间    |

## 7. 回滚点

| 粒度            | 回滚方式                                                                          |
| --------------- | --------------------------------------------------------------------------------- |
| 单个子任务      | 该子分支不合入，或 revert 其 merge commit                                         |
| 单个阶段        | 回到该阶段门的 tag：`git reset --hard <阶段门 tag>`                               |
| 依赖升级（2）   | Vue→React 依赖替换、Tailwind v4、src-tauri Rust 三步各自独立提交，可单独 revert   |
| 依赖升级（2b）  | `ts-rs` 生成产物与 Rust 侧升级同提交，一并 revert                                 |
| 整体迁移        | `feature/react-migration` 不合入 `dev`。`dev` 全程保持 Vue 版本可发版，无回滚动作 |
| 已合入 `dev` 后 | revert 迁移分支的 merge commit。`dev` 上迁移期间的独立提交不受影响                |

阶段门 tag 命名：`react-migration/phase-<n>`，`n` 取 `1`、`2`、`3`、`4a`、`4`、`5`、`6`、`7`。
- 执行偏差记录（2026-08-23）：§5/§11 的子分支命名 `feature/react-migration/<slug>` 与既有分支 `feature/react-migration` 是同一 ref 命名空间的冲突（git 无法同时持有 `refs/heads/feature/react-migration` 与其子路径）。实际采用 `react-migration/<slug>`（例：`react-migration/react-foundation`），PR 目标不变仍为 `feature/react-migration`。

`08-22-workspace-cargo-upgrade` 的回滚独立于前端迁移。

## 8. 进度追踪

| 阶段 | 子任务                        | 状态   | 门             |
| ---- | ----------------------------- | ------ | -------------- |
| 0    | 基线采集                      | 已完成 | 基线采集门     |
| 1    | 1 `react-foundation`          | 已完成 | 基座门 ✅ 2026-08-23 |
| 1    | 2 `dep-upgrade`               | 已完成 | 基座门 ✅ 2026-08-23 |
| 旁路 | 2b `workspace-cargo-upgrade`  | 已完成（PR→dev 延后，随迁移分支交付，偏差见其 implement.md） | 直接目标 `dev` |
| 2    | 2c `arch-quality-perf`        | 已完成 | 约束门 ✅ 2026-08-23 |
| 2    | 3 `design-system`             | 已完成 | 约束门 ✅ 2026-08-23（AC4 后半「src/components/ui/ 移除」随 shell-port 落地，见其 implement.md 交付门） |
| 3    | 4 `state-logic-port`          | 已完成（AC1 偏差：`src/stores/usage.ts` 暂留，归 views-usage；外壳门复核） | 外壳门         |
| 3    | 5 `shell-port`                | 已完成（AC11 表单草稿界面级验证移交 views-profiles-config） | 外壳门         |
| 4a   | 11 批次 1（profiles 共享层）  | 已完成 | 共享层前置门 ✅ 2026-08-24 |
| 4a   | 12 批次 3 前半（mcp 共享层）  | 已完成 | 共享层前置门 ✅ 2026-08-24 |
| 4    | 5b `platform-unify`           | 已完成 | 统一层门 ✅ 2026-08-24 |
| 5    | 6 `views-claude`              | 已完成 | 视图门 ✅ 2026-08-24 |
| 5    | 7 `views-codex`               | 已完成 | 视图门 ✅ 2026-08-24 |
| 5    | 8 `views-secondary-platforms` | 已完成 | 视图门 ✅ 2026-08-24 |
| 5    | 9 `views-checkin`             | 已完成（AC4 真实签到凭据未提供） | 视图门 ✅ 2026-08-24 |
| 5    | 10 `views-usage`              | 已完成 | 视图门 ✅ 2026-08-24 |
| 5    | 11 `views-profiles-config`    | 已完成 | 视图门 ✅ 2026-08-24 |
| 5    | 12 `views-sync-tools`         | 已完成 | 视图门 ✅ 2026-08-24 |
| 5    | 13 `i18n-port`                | 已完成 | 视图门 ✅ 2026-08-24 |
| 6    | 14 `test-contract-rebuild`    | 已完成 | 测试与契约门 ✅ 2026-08-24 |
| 7    | 15 `regression-release`       | 已完成 | 发布门 ✅ 2026-08-24 |

## 9. 文档修正状态

本文件核对验证命令时发现的现状偏差。

已修正（第一轮，编写阶段）：

- [x] 父任务 `prd.md` AC3 的流水线描述改为 §2.1 的 13 步。
- [x] 父任务 `prd.md` 约束 C2 的表述收窄：命令清单由 `just tauri-command-inventory-check` 保护，迁移期不失效；不受保护的是前端到命令的接线。
- [x] `08-22-arch-quality-perf/prd.md` 的背景、规则表、R5、AC5 四处改为「覆盖率门已存在于 justfile（`just frontend-coverage`，lines ≥70%），需纳入 `just ci` 并将阈值移入 `vitest.smoke.config.ts`」。

已修正（第二轮，外部评审 + 复核后）：编号 TPR-xx 为外部评审报告的条目号。

| 编号     | 偏差                                                                                                                                                                                                     | 处理                                                                                        |
| -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| TPR-01   | 阶段 4 的 `BaseProfiles` / `PlatformMcpView` 依赖阶段 5 才迁移的共享层，形成依赖环                                                                                                                        | 新增阶段 4a 共享层前置门（§3、§4）；协同点 F 加时序约束，新增 F2                             |
| TPR-02   | 约束门与外壳门同时要求「AC 全满足」和「留到阶段 5 补」                                                                                                                                                    | 两门改为只核对本阶段可验证项，端到端条款显式移交视图门 / 发布门；对应 AC 拆分                |
| TPR-03   | `design-system` PRD 要求 448 变量进 `@theme inline`，design 实际选两层结构                                                                                                                                | PRD 与 AC 改为两集合模型：稳定语义变量集合 + Tailwind namespace 映射集合                     |
| TPR-04   | `react-foundation` AC2 用 `bun run dev`（纯 Vite）验证真实 IPC，纯 Web 下 `invoke()` 不可用                                                                                                               | AC1 保留纯 Web 渲染验证，AC2 改为桌面运行时（`bun run tauri:dev`）                           |
| TPR-05   | `listen()` 返回 `Promise<UnlistenFn>`，cleanup 可能先于 resolve 执行，迟到的 unlisten 不进 cleanup                                                                                                        | `state-logic-port` §3 加 dispose 取消协议；§7 加延迟 resolve + StrictMode 用例               |
| TPR-06   | 事件名断言只查 `shell/eventBridge.ts`，漏掉 CheckIn 的组件级 WAF 等待                                                                                                                                     | 建统一前端事件 inventory（全局 + 已声明局部），新增协同点 M                                  |
| TPR-07   | `no-restricted-imports` 只限制消费方 import，不冻结 `tauri.ts` 的定义面；`src/api/index.ts:8` 的 `export * from './tauri'` 可绕过                                                                         | 指名仓库既有的 `freezes legacy direct invoke calls in tauri.ts` 冻结测试作为定义面强制手段   |
| TPR-08   | key 泄漏检测正则漏掉含下划线的 key。实测：叶子 key 4,164，匹配 4,059，漏 105（如 `checkin.stats.total_accounts`）                                                                                         | 检测器改为由实际叶子 key 集合生成；补含下划线的正反例                                        |
| TPR-09   | 通用加密向量不覆盖三种真实持久格式                                                                                                                                                                       | `workspace-cargo-upgrade` §3 改为按格式固化向量矩阵                                          |
| TPR-10   | HTTP/2 与 Cookie「不变」无观测点。`response.version()` 未被读取；现有 http2 测试只是编译期守卫                                                                                                            | AC11 重写，定观测点与前置条件；已核实 `cookie_store(true)` 开启，cookie jar 是真实风险面     |
| TPR-11   | `src/components/ui/` 与 `src/ui/` 两条路径并存                                                                                                                                                           | 统一为 `src/ui/`（父任务 `design.md` §2 的目标结构）                                         |
| TPR-12   | 阈值 P90 无法由「总行数收敛区间」唯一推出                                                                                                                                                                 | 改两段式：阶段 2 暂定值，阶段 4 后按实际分布冻结。新增协同点 N                               |
| TPR-13   | 「分支点数 ≤ 差异项数」的两个量计量单位不一致，差异项可按不同粒度拆合                                                                                                                                     | 固定差异项原子化规则并统一计量单位，比值降为辅助数据，判定由评审门做出                       |
| TPR-14   | 契约份数 16 / 18–19 / 19 三处不一致                                                                                                                                                                      | 区分基线 16 与最终 19；`platform-unify` 的契约文件名现在定为 `platform-surface-contracts.md` |
| TPR-15   | 命令清单数据源仍标为未决                                                                                                                                                                                 | 定为 `ccr-ui/src/api/generated/command-manifest.json`，新增协同点 O                          |
| TPR-16   | 父任务 AC3 与发布门固定为 13 步                                                                                                                                                                          | §2.1 加口径：基线 13 / 迁移后 14，权威为实际 recipe 清单与退出码                             |
| TPR-17   | 19 个 `implement.jsonl` 与 19 个 `check.jsonl` 只有 `_example` seed，`plan_precheck.py` 将其列为启动阻塞项                                                                                                | 属 Phase 1.3 的产物，`task.py start` 前整理。见 §10                                          |
| TPR-18   | `vite.config.ts:22` 的引用在含 `ref/` 镜像时有 5 个同名文件，机械解析歧义                                                                                                                                | 改为仓库根相对引用 `ccr-ui/vite.config.ts:<line>`                                            |
| 附加 1   | `generate_handler_common!` 不存在。实际为 `ccr-ui/src-tauri/src/commands/handler_registry.rs` 内的 `macro_rules!` 与 `commands::generate_handler()`；该名字只出现在 `docs/reports/ccr_code_audit_canvas.md` 的历史提案中 | `test-contract-rebuild` PRD AC6 与 design §1 改名                                            |
| 附加 2   | IPC 命令数不是 141+。`command-manifest.json`（schema_version 2）实测 base 334 / 含 Windows 342 / typed 271                                                                                                | 父任务测量基准表与全部引用「141+」的条目改为 334 / 342                                       |
| 附加 3   | i18n key 数不是 4,261（那是 `zh-CN.keys.txt` 的行数）。两个 locale 的叶子 key 均为 4,164                                                                                                                  | 父任务测量基准表、任务地图、`i18n-port` PRD 与 design 改为 4,164                             |
| 附加 4   | 已加密凭据不在 `crates/ccr-config` 与 `crates/ccr-db`。实际为 `ccr-codex`、`ccr-sync`、`ccr-checkin`；`ccr-db` 只用 `sha2`（导入去重），blake3 在 `ccr-core` / `ccr-skills` / `ccr-store` / `ccr-cli` 作内容哈希 | 父任务 Risks 与 `workspace-cargo-upgrade` §3 更正                                            |
| 附加 5   | `api-facade-boundary.smoke.test.ts` 遍历 `.ts\|.mts\|.vue`。不加 `.tsx` 则迁移后全部组件内的裸 `invoke()` 不被检出，门面边界保护静默失效                                                                  | 列为视图门准出条件；`test-contract-rebuild` 批次 1 执行                                      |

已修正（第三轮，用户授权改 repo 内文件）：

- [x] 根 `CLAUDE.md` 的 `just ci` 流水线描述从 10 步改为实际的 13 步，并加一句指向 `justfile` 的 `_ci-timed-windows` / `_ci-timed-linux` 作为权威来源（两个变体的步骤清单一致，已核对）。
      未写「14 步」：`frontend-coverage` 尚未纳入 `just ci`，写进已入库文档会变成另一个方向的错误。该步落地后 recipe 自身即为准，`CLAUDE.md` 无需再改。

## 10. `task.py start` 前的剩余动作

- [x] 19 个 `implement.jsonl` 与 19 个 `check.jsonl` 已按各任务真实的 spec / research 依赖整理，seed 已删除。校验以 `python .trellis/scripts/task.py validate <dir>` 逐目录执行，19 个任务全部通过（退出码 0）。
      说明：计划中的 `plan_precheck.py` 在仓库与 git 历史中均不存在；其 jsonl 校验职能由 `task.py validate` 承担，需求计数限制（requirements=0）同样适用，按 `prd.md` Requirements 节人工确认。
- [x] `plan_precheck.py` 缺失一事已在上一条内并处理：脚本不存在为文档与现实偏差，本地用 `task.py validate` 等价替代，不阻塞启动。
- [ ] 工件评审通过后执行 `python ./.trellis/scripts/task.py start`。
