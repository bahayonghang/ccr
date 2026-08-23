# smoke 测试与前端契约重建

> 父任务：`08-22-react-migration`

## Goal

重写 122 个 smoke 测试与前端契约文档（基线 16 份，迁移后 19 份），恢复迁移期失效的质量门，并尽早交付最小测试集以缩短 IPC 行为回归的保护空窗。

## Scope

### smoke 测试（122 个）

| 类型 | 数量 | 处理方式 |
|---|---|---|
| 挂载组件断言 | 63 | 从 Vue Test Utils 改为 React 测试工具，断言重写 |
| 读源码文本断言 | 19 | 断言中的文件路径与 SFC 模式改为新目录结构与 React 形态 |
| 其他（API 覆盖、状态、契约类） | 40 | 逐个判定，多数可保留断言意图，改写实现 |

配置文件：`vitest.smoke.config.ts`、`vitest.shims.d.ts`。

### 前端契约文档（基线 16 份 → 最终 19 份）

下表为基线 16 份。另有 3 份由其他子任务新增并登记进本任务范围：`react-rerender-discipline.md`、`layering-contracts.md`（`08-22-arch-quality-perf`）、`platform-surface-contracts.md`（`08-22-platform-unify`）。

| 文档 | 体积 | 关联子任务 |
|---|---|---|
| `theme-token-contracts.md` | 31.5 KB | `08-22-design-system` |
| `profiles-page-contracts.md` | 19.9 KB | `08-22-views-profiles-config` |
| `dashboard-presentation-contracts.md` | 10.9 KB | `08-22-views-usage` |
| `api-facade-boundary.md` | 9.0 KB | `08-22-react-foundation` |
| `provider-template-contracts.md` | 8.9 KB | `08-22-views-profiles-config` |
| `index.md` | 8.3 KB | 本任务 |
| `raw-config-editor-contracts.md` | 7.4 KB | `08-22-views-sync-tools` |
| `checkin-ux-contracts.md` | 7.2 KB | `08-22-views-checkin` |
| `usage-chart-stability-contracts.md` | 7.1 KB | `08-22-views-usage` |
| `grok-settings-contracts.md` | 5.5 KB | `08-22-views-secondary-platforms` |
| `development-resource-contracts.md` | 5.3 KB | `08-22-views-claude` |
| `environment-scoped-dashboard-contracts.md` | 5.0 KB | `08-22-views-usage` |
| `brand-asset-pipeline.md` | 4.4 KB | `08-22-design-system` |
| `sync-security-contracts.md` | 4.2 KB | `08-22-views-sync-tools` |
| `confirm-interaction-contracts.md` | 3.6 KB | `08-22-shell-port` |
| `monitoring-log-contracts.md` | 1.1 KB | `08-22-views-sync-tools` |

### 新增契约登记（基线 16 → 18 → 19）

子任务新增并登记进本任务范围的 3 份契约。三份均已落盘（2026-08-24），纳入本任务的重写范围。

| 文档（路径） | 新增子任务 | 状态 |
|---|---|---|
| `.trellis/spec/ccr-ui/frontend/layering-contracts.md` | `08-22-arch-quality-perf` | 已落盘（2026-08-23） |
| `.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md` | `08-22-arch-quality-perf` | 已落盘（2026-08-23） |
| `platform-surface-contracts.md` | `08-22-platform-unify` | 已落盘（2026-08-24）：`.trellis/spec/ccr-ui/frontend/platform-surface-contracts.md` |

## Requirements

- R1 122 个 smoke 测试全部重写并通过。
- R2 测试覆盖范围不低于迁移前，按被测组件数与被测契约条目数计数比对。
- R3 契约文档重写，无残留 Vue 文件路径与 SFC 模式引用。基线 16 份，最终 19 份（含其他子任务新增的 3 份）。
- R4 契约文档中的每条断言对应至少一个可执行测试，或明确标注为人工验证项。
- R5 最小测试集在 `08-22-shell-port` 完成后即交付，优先覆盖：`api-facade-coverage`（IPC 命令名与 wrapper 的对应关系，334 base / 342 含 Windows）、`api-facade-boundary`（门面边界，并把该测试的遍历后缀集合扩到 `.tsx`）、Tauri Event 名清单、路由清单。
- R6 `vitest.smoke.config.ts` 适配 React 测试环境，`bun run test:smoke` 命令名不变。
- R7 `tests/artifacts/` 的产物结构与 `bun run docs:audit` 的校验保持可用。
- R8 `theme-token-contracts.md` 重写时保留已登记的 `0.75rem` 字号例外（Profiles 共享层密集元信息，低于 Label 下限 `0.8125rem` 一档）。
- R9 契约文档保留三层主题模型语义：`data-theme` 控制明暗与 system 解析，`data-flavor` 控制色板族，`data-accent` 控制强调色。

## Acceptance Criteria

- [ ] AC1 `bun run test:smoke` 退出码 0，通过测试数不少于 122。最新全量：通过数 ≥490，16 条失败集中在 `@tauri-apps/api/core` mock 未拦住真实 `invoke`（`ssh-hardening` / `typed-*-client` / `api-facade-coverage` 执行侧等）。契约重写与本任务新增/改写的文件在隔离运行下通过。
- [x] AC2 覆盖范围比对表落盘：迁移前后的被测组件数与被测契约条目数逐项对照，无下降项。
- [x] AC3 19 份契约文档重写完成，`rg '\.vue|<script setup|scoped' .trellis/spec/ccr-ui/frontend/` 无匹配。
- [x] AC4 契约断言与测试的对应表落盘，无未映射的断言。
- [x] AC5 最小测试集在 `08-22-shell-port` 完成后 3 个工作日内可运行，含全量 IPC 命令名断言（334 base / 342 含 Windows）。
- [ ] AC6 IPC 命令名与全部 Tauri Event 名的清单断言通过。命令名的数据源为 `ccr-ui/src/api/generated/command-manifest.json`（`base_command_count` 334 / `windows_command_count` 342，按 `platform` 字段分组断言；该文件是 Rust 测试 `command_inventory_document_matches_registry` 的生成产物之一，与 `handler_registry.rs` 的 `macro_rules!` / `commands::generate_handler()` 同步）。事件名的数据源为统一前端事件 inventory（全局桥接层 + 组件级局部事件，见 `design.md` §1.2）。
      注：先前工件写「141+ 命令」与「`generate_handler_common!` 注册表」，两处均不准确——实测命令数为 334 / 342，且仓库无 `generate_handler_common` 符号。
      批次 1：命令名断言已落地。事件名断言覆盖全局集合相等 + inventory ⊆ Rust emit。CheckIn WAF 局部事件已登记为 `views-checkin` 所有、尚未完整迁移，故本条保持未勾。
- [ ] AC7 `bun run docs:audit` 退出码 0。
- [x] AC8 `.trellis/spec/ccr-ui/frontend/index.md` 更新，反映重写后的 19 份文档结构。
- [ ] AC9 `bun run test:i18n` 退出码 0。
- [ ] AC10 `just frontend-check-quick` 退出码 0。

## 前置与后续

- 前置：`08-22-shell-port`（最小测试集可开始交付）。完整重写需等各视图子任务交付对应实现。
- 后续：`08-22-regression-release`。

## Out of Scope

- 新增测试用例覆盖迁移前未覆盖的功能。等价重建，不扩大覆盖面。
- 端到端测试框架引入。当前 `playwright` 已在 devDependencies 中但未构成 e2e 套件，本任务不改变该状态。
- `crates/` 下的 Rust 测试。
- `ccr-vscode` 测试。
- Storybook 引入。`storybook-static/` 为历史产物，`package.json` 无 Storybook 依赖，本任务不引入。

## Notes

- 父任务约束 C2 指出迁移期 122 个测试与 16 份契约同时失效，IPC 命令的行为回归无自动化保护。R5 的最小测试集是缩短该空窗的唯一手段，优先级高于完整重写。
- 另有一处易被忽略的失效：`ccr-ui/tests/api-facade-boundary.smoke.test.ts` 的 `walkSourceFiles` 只遍历 `.ts` / `.mts` / `.vue`。迁移后组件是 `.tsx`，不扩后缀集合则其三个用例对全部 React 组件失效且**静默通过**。该动作在批次 1 第一项执行。
- 契约文档与视图子任务一一对应。建议各视图子任务在动手前先由本任务提供该域的契约重写稿，避免实现完成后再对齐契约导致返工。
- `theme-token-contracts.md` 为 31.5 KB，是全仓最大的契约文档，需与 `08-22-design-system` 协同重写，不宜独立完成。
