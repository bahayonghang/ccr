# 技术设计：smoke 测试与前端契约重建

> 父任务：`08-22-react-migration`。本任务恢复迁移期失效的质量门。约束 C2 的空窗由 R5 的最小测试集缩短，该项优先级高于完整重写。

## 1. 保护空窗的准确边界

父任务约束 C2 已收窄为：IPC 命令的**清单**由 `just tauri-command-inventory-check`（Rust 侧测试 `commands::handler_registry::tests::command_inventory_document_matches_registry`）保护，迁移期不失效；不受保护的是**前端到命令的接线**——即某个视图是否仍调用正确的命令、参数是否正确。

**现状更正两处：**

1. **命令数不是 141+。** `ccr-ui/src/api/generated/command-manifest.json`（`schema_version` 2）实测 `base_command_count` 334、`windows_command_count` 342（Windows 专属 8 条）、`typed_command_count` 271。
2. **`generate_handler_common!` 不存在。** 实际注册表是 `ccr-ui/src-tauri/src/commands/handler_registry.rs` 内的 `macro_rules!`，对外入口为 `commands::generate_handler()`（分 `#[cfg(target_os = "windows")]` 与非 Windows 两个变体，前者多 4+ 条 `wsl_*` 命令）。`generate_handler_common` 这个名字只出现在 `docs/reports/ccr_code_audit_canvas.md` 的历史重构提案中，仓库无此符号。PRD AC6 的表述据此更正。

因此最小测试集的目标不是重复 Rust 侧已有的清单断言，而是断言前端侧的对应关系：

| 断言                                    | 与 Rust 侧的关系                                                                                                       |
| --------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `src/api` 的 wrapper 集合覆盖全部命令名 | 补充。Rust 侧断言 registry 与生成产物一致，此处断言前端 wrapper 与命令名一致                                           |
| 门面边界                                | 见下方「门面边界的两侧」                                                                                               |
| Tauri Event 名清单                      | 补充。断言前端事件 inventory 与 Rust 侧 `emit` 的名字一致。**inventory 含全局与局部两部分**，见第 1.2 节               |
| 路由清单（75 条）                       | 新增。Rust 侧无对应保护                                                                                                |

### 1.1 命令名断言的数据源（已定，不再是未决项）

数据源为 `ccr-ui/src/api/generated/command-manifest.json`。理由：

- 它是 `command_inventory_document_matches_registry` 比对的生成产物之一（该测试的 `generated_artifacts()` 同时写 `permissions/command-inventory.toml`、两份 `docs/reference/tauri-command-inventory.md`、`command-manifest.json`、`commandCapabilities.ts`、`commandExec.ts`、`sync.ts`），因此与注册表同步，漂移会被 Rust 侧测试拦住。
- 它是 JSON，字段稳定（`commands[].id` / `platform` / `input_schema` / `output_schema`），解析不依赖宏形态。
- 它已在 `src/api/` 内，前端可直接 import，无跨目录读取。

不解析 `macro_rules!`：宏形态变化时解析失效，且需处理两个 `cfg` 变体。

断言按 `platform` 字段分组：`base`（334 条）在全平台断言，`windows`（8 条）只在 Windows 分支断言。

若 `08-22-workspace-cargo-upgrade` 改动生成侧，需通知本任务（协同点 O）。

### 1.2 事件名 inventory 含局部事件

事件名**不全在** `shell/eventBridge.ts`。`08-22-views-checkin` 有意把 WAF 一次性等待保留为组件级 `listen()`（其 `design.md` §4：与向导实例绑定的一次性等待，不是全局数据流）。只扫桥接层会漏掉它；强行把它塞进桥接层又违反该组件的实例级生命周期设计。

因此建统一的前端事件 inventory，两部分合并（协同点 M）：

| 部分 | 提供者                   | 内容                                    |
| ---- | ------------------------ | --------------------------------------- |
| 全局 | `08-22-state-logic-port` | `shell/eventBridge.ts` 内的常驻订阅     |
| 局部 | `08-22-views-checkin`    | 组件级一次性订阅（WAF 等待）            |

每项的字段：事件名、所有者（`eventBridge` 或具体组件路径）、生命周期（常驻 / 一次性）、对应的 Rust `emit` 位置。

AC6 的事件名断言以合并后的 inventory 为数据源。新增局部事件时须同时登记——本任务在 `layering-contracts.md` 或事件 inventory 文档中写明该要求，否则下一次新增会再次绕过断言。

### 1.3 门面边界的两侧

| 侧     | 手段                                                                                                            | 归属                              |
| ------ | --------------------------------------------------------------------------------------------------------------- | --------------------------------- |
| 消费侧 | ESLint `no-restricted-imports`：除 `src/api/index.ts` 外禁止 import `src/api/tauri.ts`                          | `08-22-arch-quality-perf`         |
| 定义侧 | 既有 `api-facade-boundary.smoke.test.ts` 的 `freezes legacy direct invoke calls in tauri.ts`（9 条允许命令集合） | 本任务保留并维护                  |

lint 单独不足以冻结定义侧：`src/api/index.ts:8` 有 `export * from './tauri'`，新 wrapper 加进 `tauri.ts` 后经 `@/api` 消费，import 规则全绿。

**该测试文件的遍历后缀集合必须扩到 `.tsx`。** 现为 `/\.(ts|mts|vue)$/`（`walkSourceFiles`）。迁移后组件是 `.tsx`，不改则其三个用例（`keeps invoke() usage inside the API layer across all of src/`、`freezes legacy direct invoke calls in tauri.ts`、`keeps manifest-typed commands behind generated clients`）对全部 React 组件失效并静默通过。该动作列为批次 1 的第一项，并作为父任务视图门的准出条件。

`INVOKE_ALLOWED_PATHS` 白名单（`src/api/domains/`、`src/api/generated/`、`src/api/runtime/`、`src/api/invokeRuntime.ts`、`src/api/tauri.ts`、`src/utils/logger.ts`）在迁移后核对：`src/utils/logger.ts` 若按 `utils-disposition.md` 移入 `shell/`，该条目需同步改路径。

## 2. 最小测试集的交付时限

R5：在 `08-22-shell-port` 完成后即交付。AC5：3 个工作日内可运行（协同点 C）。

四项优先内容按第 1 节的表。其中路由清单断言依赖 `08-22-shell-port` 的路由表，因此该子任务交付即为本任务的启动信号。

最小测试集不含组件挂载断言——组件在阶段 5 才迁移。它只覆盖框架无关的接线面（`src/api`、事件名、路由表），因此能在视图迁移前交付。

## 3. 122 个测试的三类重写

| 类型                           | 数量 | 重写方法                                                                                                                           |
| ------------------------------ | ---- | ---------------------------------------------------------------------------------------------------------------------------------- |
| 挂载组件断言                   | 63   | `@vue/test-utils` → `@testing-library/react` 16.3.2。断言从「查 DOM 结构」改为「查可访问性角色与文本」。语义保持等价，不扩大覆盖面 |
| 读源码文本断言                 | 19   | 断言中的文件路径改为 `path-mapping.md` 的新路径；SFC 模式（`<script setup>`、`scoped`）改为 React 形态（`.tsx`、`.module.css`）    |
| 其他（API 覆盖、状态、契约类） | 40   | 逐个判定。多数可保留断言意图，改写实现                                                                                             |

第 2 类的 19 个测试是「断言源码里存在某个模式」的形态。这类断言在目录全量重组（父任务 `design.md` §2）后必须改路径。改路径的依据是 `path-mapping.md`（`08-22-react-foundation` AC9），不靠搜索猜测。

Out of Scope 明确：等价重建，不扩大覆盖面。因此重写时不因「顺手」而增加新断言。

## 4. 覆盖范围比对表（AC2）

计数方法两项，逐项对照迁移前后：

| 计数维度       | 迁移前的取数方式                                | 迁移后   |
| -------------- | ----------------------------------------------- | -------- |
| 被测组件数     | 63 个挂载测试涉及的组件去重计数                 | 同法计数 |
| 被测契约条目数 | 19 份契约中被至少一个测试覆盖的断言条目数（迁移前按基线 16 份计） | 同法计数 |

「无下降项」（AC2）的含义：迁移前被覆盖的组件与契约条目，迁移后仍被覆盖。允许上升（重写时发现某断言可顺带覆盖更多），不允许下降。

比对表落盘为 `coverage-comparison.md`。

## 5. 契约文档：基线 16 份 → 最终 19 份

| 阶段     | 份数 | 增量                                                                                            |
| -------- | ---- | ----------------------------------------------------------------------------------------------- |
| 基线     | 16   | PRD Scope 表的 16 份                                                                            |
| +2       | 18   | `08-22-arch-quality-perf` 的 `react-rerender-discipline.md`、`layering-contracts.md`（其 §10、AC12） |
| +1       | 19   | `08-22-platform-unify` 的 `platform-surface-contracts.md`（其 AC9，文件名已在该任务 `design.md` §2 定） |

**最终份数为 19，不是「18 或 19」。** 三个增量都是已确定的交付物，`platform-unify` 的文件名先前标为待定，现已定，因此份数不再有分支。AC3 的 `rg` 检查覆盖全部 19 份。

`08-22-shell-port` 的 `shared-interfaces.md`、`08-22-views-profiles-config` 的 `profiles-shared-interfaces.md`、`08-22-views-sync-tools` 的 `mcp-shared-interfaces.md`、`08-22-views-secondary-platforms` 的 generic 接口公示，四份是任务目录内的公示文档，不进 spec 目录，不计入 19 份。是否提升为长期契约由本任务判定并记录；若判定提升，份数相应增加并在收尾时更新 `index.md`。

## 6. 契约重写稿的交付顺序（协同点 D）

PRD Notes：各视图子任务在动手前先由本任务提供该域的契约重写稿，避免实现完成后再对齐契约导致返工。

交付顺序按视图子任务的启动顺序，即阶段 4 → 5 门前一次性交付全部域的重写稿。逐域的对应关系（PRD Scope 表）：

| 域                          | 契约                                                                                                                     | 体积                |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------- |
| `design-system`             | `theme-token-contracts.md`、`brand-asset-pipeline.md`                                                                    | 31.5 + 4.4 KB       |
| `views-profiles-config`     | `profiles-page-contracts.md`、`provider-template-contracts.md`                                                           | 19.9 + 8.9 KB       |
| `views-usage`               | `dashboard-presentation-contracts.md`、`usage-chart-stability-contracts.md`、`environment-scoped-dashboard-contracts.md` | 10.9 + 7.1 + 5.0 KB |
| `react-foundation`          | `api-facade-boundary.md`                                                                                                 | 9.0 KB              |
| `views-sync-tools`          | `raw-config-editor-contracts.md`、`sync-security-contracts.md`、`monitoring-log-contracts.md`                            | 7.4 + 4.2 + 1.1 KB  |
| `views-checkin`             | `checkin-ux-contracts.md`                                                                                                | 7.2 KB              |
| `views-secondary-platforms` | `grok-settings-contracts.md`                                                                                             | 5.5 KB              |
| `views-claude`              | `development-resource-contracts.md`                                                                                      | 5.3 KB              |
| `shell-port`                | `confirm-interaction-contracts.md`                                                                                       | 3.6 KB              |
| 本任务                      | `index.md`                                                                                                               | 8.3 KB              |

**两处需协同不宜独立完成**：

1. `theme-token-contracts.md`（31.5 KB，全仓最大）与 `08-22-design-system` 协同（PRD Notes）。
2. `grok-settings-contracts.md` 需按 `08-22-views-secondary-platforms` §5 分割为 base 侧与 Grok 侧两部分。

**三处的断言已由对应子任务转为可执行形式**，本任务不再重复写断言，只重写文档：`usage-chart-stability-contracts.md`（`views-usage` 批次 0）、`raw-config-editor-contracts.md`（`views-sync-tools` 批次 0）、`checkin-ux-contracts.md` 及 8 个 CheckIn 测试（`views-checkin` 同批次推进）。与这三处对齐，避免产生两份断言。

## 7. 契约断言与测试的对应表（AC4）

每条断言对应至少一个可执行测试，或明确标注为人工验证项（R4）。

表的列：契约文件、断言编号或摘要、对应测试文件与用例名、或「人工验证」标记 + 验证归属子任务。

标注为人工验证的典型项：WAF WebView bypass（依赖 WebView 实际行为）、窗口 chrome 操作、真实签到请求、视觉对比度。这些项的验证归 `08-22-regression-release` 或对应视图子任务。

无未映射的断言（AC4）。

## 8. 三层主题模型与字号例外（R8、R9）

`theme-token-contracts.md` 重写时保留：

- `0.75rem` 字号例外（Profiles 共享层密集元信息，低于 Label 下限 `0.8125rem` 一档）。
- 三层主题模型语义：`data-theme` 控制明暗与 system 解析，`data-flavor` 控制色板族，`data-accent` 控制强调色。

## 9. 测试配置

`vitest.smoke.config.ts` 适配 React 测试环境（R6）——该项由 `08-22-react-foundation` 批次 4 完成，本任务在其基础上补充。

`08-22-arch-quality-perf` 批次 5 把覆盖率阈值移入该配置文件。本任务的 122 个测试是覆盖率的分母来源，重写完成后需复核阈值仍可达（其 R5 的取值复核依据）。

`bun run test:smoke` 命令名不变（R6）。`tests/artifacts/` 的产物结构与 `bun run docs:audit` 的校验保持可用（R7、AC7）。

## 10. 不在范围内

- 新增测试覆盖迁移前未覆盖的功能（等价重建）。
- e2e 框架引入。`playwright` 已在 devDependencies 但未构成 e2e 套件；`08-22-arch-quality-perf` 只把它作为性能测量驱动，本任务不改变该状态。
- `crates/` 的 Rust 测试与 `ccr-vscode` 测试。
- Storybook 引入。

## 11. 未决项

- 40 个「其他」类测试的逐个判定结果。
- 四份接口公示文档是否提升为长期契约（第 5 节末段）。

已定，不再列为未决：

- 命令名断言的数据源为 `ccr-ui/src/api/generated/command-manifest.json`（第 1.1 节）。
- 契约总份数为 19（第 5 节）。
- 事件名 inventory 由全局 + 局部两部分合并（第 1.2 节）。
