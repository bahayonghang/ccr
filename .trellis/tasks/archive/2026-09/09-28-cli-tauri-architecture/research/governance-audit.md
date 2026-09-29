# 架构规模、规范与验证边界审查

日期：2026-09-28。基线：`34d8a85e0e48b793733835e0304c8ed33940fcee`，分支 `dev`。本文件只记录当前仓库事实及规划建议。

## 方法和范围

- 阅读根与 `ccr-ui/` code map、根与嵌套 AGENTS、Trellis 包规范、Cargo 清单、Just recipes、GitHub 工作流、命令 manifest、现有质量脚本。
- 执行不自动修复的检查，完整输出保存在同目录 `baseline-*.log`，退出码和耗时保存在同名 JSON。
- 原有未跟踪文件为 `ccr-ui/.tmp-desktop-probe.mjs`、`ccr-ui/.tmp-insights-visual.mjs`。不删除、不改写、不纳入产品缺陷修复。
- 原有 `09-24-home-insights-frontend` 处于 `in_progress`，其父任务处于 `planning`。本轮不重设这些任务、不归档、不重复承担首页功能。
- 根 `CONTEXT.md` 不存在；没有创建替代文件。当前架构事实使用清单、代码、规范与测试。

## 当前结构与规模

`inventory.json` 保存计算口径、完整 local crate 依赖边及主要文件。根 workspace 有 13 个 crate；Tauri 是独立 workspace，另有 `command-macros`。Tauri 同时依赖 `ccr` 和多个领域 crate。

| 扫描范围 | 源文件数 | 行数 | 口径 |
| --- | ---: | ---: | --- |
| `crates/` | 432 | 161672 | `.rs/.ts/.tsx`，包括测试、注释与空行 |
| `ccr-ui/src/` | 696 | 95914 | 同上；排除名为 generated 的目录 |
| `ccr-ui/src-tauri/src/` | 90 | 48109 | 同上 |

行数仅用于定位复核区域，不作为缺陷严重性或拆分依据。现有 `module-decomposition.md` 已要求按职责抽取并保留测试和兼容接口。

命令 manifest v2 记录 340 个基础命令、348 个 Windows 命令、277 个 generated typed 命令，277 个均有 exact wire types；另外 71 个属于 legacy schema。来源：`ccr-ui/src/api/generated/command-manifest.json:1`。未逐一证明 71 个 legacy 命令存在类型错误；这个数字仅代表契约迁移范围。

## G-01 / P2：本地完整门禁与独立 Tauri 门禁的覆盖不同

- **事实**：根 `just ci` 的 Windows 列表包含 root workspace lint/test、Tauri inventory/bindings 与前端检查，但没有 `tauri-ci`。见 `justfile:570-584`；Linux/macOS 列表同样见 `justfile:639`、`justfile:700`。
- **Tauri 权威门禁**：`justfile:1545-1551` 执行独立 manifest 的 fmt/check/clippy/test、bindings 和 inventory。`ccr-ui/justfile:741-770` 的 `check` 只用 `cargo check` 检查后端。
- **反证和范围**：托管工作流确实执行 `just tauri-ci`，见 `.github/workflows/tauri-rust-ci.yml:27-49`，最终聚合器检查平台与覆盖率结果，见同文件 `106-127`。不能据此声称 GitHub CI 缺少桌面检查。
- **根因**：root 与 desktop 是两套 Rust workspace；“本地完整 CI”标签没有表达真实组合。绑定测试会构建桌面测试目标，但不执行所有桌面行为测试。
- **影响**：仅依照根 `just ci` 完成跨 CLI/Tauri 验收时，无法证明桌面全部行为测试和严格 Clippy 已通过。
- **Owner / 改动类别**：根验证编排与 `.trellis/spec/ccr/backend/dependency-governance.md`；规范和门禁重构。
- **建议**：确立一条可核验的跨 workspace 验收组合，复用 `tauri-ci`；将只读检查与 `version-sync`、`fmt`、bindings regeneration 的改写副作用明确分开。不得将诊断性子集通过视为完整通过。
- **验收**：解析 local aggregate 与 hosted required lanes，断言覆盖 desktop clippy 和全量测试；使用 fixture 令桌面测试失败，证明 aggregate 返回失败；保持既有平台矩阵。

## G-02 / P3：架构规范存在相互冲突的已过期事实

- 命令注册规范同一文件写 340/348，另写 336：`.trellis/spec/ccr/backend/tauri-handler-registry.md:59`、`:150`、`:199`。
- 依赖治理规范另写 315/323 个命令及 30 个模块：`.trellis/spec/ccr/backend/dependency-governance.md:361`。
- 依赖治理仍用已经移除的 Vue MainLayout 路径作为当前正确示例：`.trellis/spec/ccr/backend/dependency-governance.md:31-32`、`:47-50`。实际 React 路由与入口见 `ccr-ui/code_map.md:17-19`。
- code map 仍警告 lint 可能自动修复：`ccr-ui/code_map.md:52`；当前 `package.json` 的 `lint`/`lint:ci` 无 `--fix`，`lint:fix` 才改写。
- `ccr-usage` 无独立包规范目录，但其 SQL/投影职责已在 AGENTS 与 `llmusage-provider-adapter.md` 规定。这里是导航/归属覆盖缺口，不能推导为该包完全无规范。
- **影响**：代理或维护者可能依照过期数字、路径、包归属设计重构和验收。
- **Owner / 改动类别**：现有 owner 下的 spec、code map 和 docs；规范同步。
- **建议/验收**：命令计数从 registry/manifest 验证；当前示例路径可解析；规范入口明确 `ccr-usage` 归属；保留有意冻结的命令 count 测试；不按相同数字机械替换历史案例。

## G-03 / P2：格式与结构守卫无法验证跨客户端行为一致性

- `scripts/quality/check_secret_writes.py:23-29` 只列出 5 个敏感模块；`:30-35` 的正则检查直接 async write 和一部分 AtomicWriter 链。通过结果不覆盖 Tauri OAuth pending 等其他写入口。
- `ccr-ui/scripts/check-arch-boundaries.mjs:65-91` 验证几个违规 import fixture，循环扫描只证明导入图无循环。两者不验证 Settings 能力、后台任务重入、CLI/TUI/Tauri 操作副作用是否一致。
- 实际通过的检查与各域审查发现并存，说明关键应用用例缺少端到端一致性契约。具体反例见 `cli-audit.md`、`tauri-audit.md`、`frontend-audit.md`。
- **Owner / 改动类别**：共享业务用例测试、Tauri fixture 和 React 行为 smoke；测试机制补齐。
- **建议**：将相同临时目录 fixture 输入分别送入 CLI/TUI/Tauri 适配器，比较持久状态、结果及错误；设置页通过 DOM 交互验证 capability；后台 job 使用可控 barrier 验证 start/cancel/terminal 顺序。保持现有静态守卫作为补充。
- **验收**：每个 P1 反例至少一个行为测试，测试在旧行为下失败；不通过提高覆盖率数字或替换整页 mock 来替代行为验收。

## 验证结果和未验证项

已运行版本、格式、工作流治理、敏感写入守卫、TypeScript、循环依赖、架构违规 fixture 和正式前端 lint。详细状态以 `baseline-*.json` 为准。

正式 `bun run lint:ci` 返回 1：原有 2 个临时 `.mjs` 共 5 条 `no-console`。没有清理原文件；排除这两个文件的 ESLint 仅作为受控诊断，不改写正式结果。

本轮不执行 `just ci`：该命令主动运行 `version-sync`、`fmt`，不满足本次只读产品审查边界。未执行 full Rust/Tauri 全量测试、发布构建、原生桌面或视觉验收、远端 SSH/WSL/WebDAV、真实用户配置切换及真实 llmusage 操作。本轮不能授予发布或重构完成结论。
