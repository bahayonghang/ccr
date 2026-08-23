# 执行计划：smoke 测试与前端契约重建

> 父任务：`08-22-react-migration`。跨阶段 3–6：最小测试集在阶段 3 后即交付，契约重写稿在阶段 4 → 5 门前交付，完整重写在阶段 6。
> 分支：`feature/react-migration/test-contract-rebuild`，PR 目标 `feature/react-migration`。

## 外壳完成通知（协同点 C）

`08-22-shell-port` 已交付 React 外壳与 75 条路由。请开始最小测试集（批次 1，3 个工作日内）。路由表入口：`ccr-ui/src/shell/router.tsx`。

## 前置确认

- [ ] `08-22-react-foundation` 批次 4 已完成 `vitest.smoke.config.ts` 的 React 适配，且 `path-mapping.md` 已落盘。
- [ ] `08-22-shell-port` 已交付（最小测试集的启动信号，协同点 C）。
- [ ] `08-22-arch-quality-perf` 已登记两份新契约（`react-rerender-discipline.md`、`layering-contracts.md`）。契约基线 16 → 18，加 `08-22-platform-unify` 的 `platform-surface-contracts.md` 后为 19。
- [ ] `08-22-state-logic-port` 已交付前端事件 inventory 的全局部分（协同点 M）。
- [ ] `git checkout -b feature/react-migration/test-contract-rebuild feature/react-migration`

## 批次 1：最小测试集（限时 3 个工作日）

`08-22-shell-port` 交付后 3 个工作日内可运行（AC5）。缩短约束 C2 的保护空窗，优先级高于完整重写。

- [ ] **第一项，先做**：把 `tests/api-facade-boundary.smoke.test.ts` 的 `walkSourceFiles` 后缀集合从 `/\.(ts|mts|vue)$/` 扩到含 `.tsx`。不改则该文件的三个用例对全部 React 组件失效且静默通过（`design.md` §1.3）。同时核对 `INVOKE_ALLOWED_PATHS`——`src/utils/logger.ts` 若已按 `utils-disposition.md` 移入 `shell/`，同步改路径。
- [ ] 保留该文件既有的 `freezes legacy direct invoke calls in tauri.ts` 用例（9 条允许命令集合）。它是门面**定义侧**的唯一强制手段，lint 规则只管消费侧。
- [ ] `api-facade-coverage`：`src/api` 的 wrapper 集合覆盖全部命令名（AC6 的前半）。数据源为 `src/api/generated/command-manifest.json`，按 `platform` 字段分组：`base` 334 条全平台断言，`windows` 8 条只在 Windows 分支断言（`design.md` §1.1）。**不解析 `handler_registry.rs` 的宏**。
- [ ] Tauri Event 名清单断言：数据源为合并后的前端事件 inventory（全局部分来自 `08-22-state-logic-port`，局部部分来自 `08-22-views-checkin`），与 Rust 侧 `emit` 一致（AC6 的后半、`design.md` §1.2、协同点 M）。
- [ ] 在事件 inventory 文档中写明「新增局部事件须同时登记」，否则下一次新增会再次绕过断言。
- [ ] 路由清单断言：75 条路径，数据源为 `08-22-shell-port` 的路由表。
- [ ] 不含组件挂载断言（组件在阶段 5 才迁移）。

验证：`bun run test:smoke` 中这些项通过；`just tauri-command-inventory-check` 退出码 0（Rust 侧的独立保护仍在）。

## 批次 2：契约重写稿（阶段 4 → 5 门前交付）

按 `design.md` §6 的逐域表，一次性交付全部域的重写稿。

- [ ] 与 `08-22-design-system` 协同重写 `theme-token-contracts.md`（31.5 KB）。保留 `0.75rem` 字号例外与三层主题模型语义（R8、R9）。
- [ ] `grok-settings-contracts.md` 分割为 base 侧与 Grok 侧两部分（`08-22-views-secondary-platforms` §5）。
- [ ] 三处已由子任务转为可执行断言的契约，本任务只重写文档不重复写断言：`usage-chart-stability-contracts.md`、`raw-config-editor-contracts.md`、`checkin-ux-contracts.md`。
- [ ] 其余契约逐份重写。
- [ ] 交付给对应视图子任务，供其动手前取用（协同点 D）。

此时重写稿是「预期形态」——实现尚未完成。稿件在阶段 6 按实际实现回填修正。

## 批次 3：63 个挂载测试重写（阶段 6）

- [ ] `@vue/test-utils` → `@testing-library/react` 16.3.2。
- [ ] 断言从查 DOM 结构改为查可访问性角色与文本，语义保持等价。
- [ ] 不扩大覆盖面。
- [ ] 按视图域分组提交，与七个视图子任务的交付顺序对齐。

CheckIn 域的 8 个测试例外：由 `08-22-views-checkin` 在其各批次内同步推进（其 `design.md` §8），本任务提供重写稿并回验，不重复实现。

## 批次 4：19 个源码文本断言重写（阶段 6）

- [ ] 断言中的文件路径按 `path-mapping.md` 改为新路径。不靠搜索猜测。
- [ ] SFC 模式（`<script setup>`、`scoped`）改为 React 形态（`.tsx`、`.module.css`）。

## 批次 5：40 个其他测试（阶段 6）

- [ ] 逐个判定：保留断言意图改写实现，或判定为已被批次 1 的最小测试集覆盖。
- [ ] 判定记录落盘。

## 批次 6：覆盖范围比对与对应表

- [ ] 按 `design.md` §4 的两个维度计数，`coverage-comparison.md` 落盘（AC2）。无下降项。
- [ ] 按 `design.md` §7 产出契约断言与测试的对应表（AC4）。无未映射断言。
- [ ] 人工验证项标注归属子任务（`08-22-regression-release` 或对应视图子任务）。

## 批次 7：契约收尾与索引

- [ ] 契约总份数为 **19**（基线 16 + `arch-quality-perf` 2 + `platform-unify` 的 `platform-surface-contracts.md` 1）。不再有 18 / 19 的分支。
- [ ] `index.md`（8.3 KB）更新，反映重写后的 19 份文档结构（AC8）。
- [ ] `rg '\.vue|<script setup|scoped' .trellis/spec/ccr-ui/frontend/` 无匹配（AC3）。
- [ ] 四份接口公示文档是否提升为长期契约，判定并记录（`design.md` §5 末段）。判定为提升则份数相应增加，`index.md` 同步。
- [ ] `tests/artifacts/` 产物结构与 `bun run docs:audit` 校验可用（R7）。
- [ ] 覆盖率阈值复核：122 个测试重写后阈值仍可达，结论给 `08-22-arch-quality-perf`（其 R5）。

## 验证命令

| 时机        | 命令                                                                                                                               |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| 批次 1 后   | `bun run test:smoke`、`just tauri-command-inventory-check`                                                                         |
| 批次 3–5 后 | `bun run test:smoke`（通过数逐步趋近 122）                                                                                         |
| 批次 6 后   | `just frontend-coverage`                                                                                                           |
| 批次 7 后   | `bun run docs:audit`（AC7）、`bun run test:i18n`（AC9）、`rg '\.vue\|<script setup\|scoped' .trellis/spec/ccr-ui/frontend/`（AC3） |
| 交付前      | `just frontend-check-quick`（AC10）                                                                                                |

## 交付门（父任务测试与契约门）

- [ ] AC1–AC10 全部满足。
- [ ] `bun run test:smoke` 通过数不少于 122（AC1）。
- [ ] 最小测试集在 `08-22-shell-port` 交付后 3 个工作日内可运行（AC5）。
- [ ] `coverage-comparison.md` 落盘，无下降项（AC2）。
- [ ] 契约断言与测试对应表落盘，无未映射断言（AC4）。
- [ ] 19 份契约重写完成，spec 目录无 Vue 路径与 SFC 模式引用（AC3）。
- [ ] `api-facade-boundary.smoke.test.ts` 的遍历后缀集合已含 `.tsx`（批次 1 第一项），且其三个用例均通过。
- [ ] `index.md` 已更新（AC8）。
- [ ] 40 个「其他」类测试的判定记录落盘。
- [ ] 覆盖率阈值复核结论已给 `08-22-arch-quality-perf`。

## 回滚点

| 批次 | 回滚方式                                                     |
| ---- | ------------------------------------------------------------ |
| 1    | 最小测试集单独提交。它是保护手段，回滚会重新打开空窗，不建议 |
| 2    | 契约重写稿，revert 无代码影响                                |
| 3–5  | 按测试分组提交，可按组回退                                   |
| 6–7  | 比对表与索引                                                 |

## 协同点

| 编号 | 内容                                    | 对方                                               | 时机         |
| ---- | --------------------------------------- | -------------------------------------------------- | ------------ |
| C    | 最小测试集在对方交付后 3 个工作日内交付 | `08-22-shell-port`                                 | 批次 1       |
| D    | 契约重写稿先行                          | 七个视图子任务 + `design-system` + `shell-port`    | 批次 2       |
| —    | `theme-token-contracts.md` 协同重写     | `08-22-design-system`                              | 批次 2       |
| —    | `grok-settings-contracts.md` 分割       | `08-22-views-secondary-platforms`                  | 批次 2       |
| —    | 三处已可执行的断言不重复写              | `views-usage`、`views-sync-tools`、`views-checkin` | 批次 2       |
| —    | 8 个 CheckIn 测试由对方同批次推进       | `08-22-views-checkin`                              | 批次 3       |
| —    | 新增契约登记；覆盖率阈值复核            | `08-22-arch-quality-perf`                          | 前置与批次 7 |
| —    | `platform-surface-contracts.md` 登记为第 19 份 | `08-22-platform-unify`                       | 批次 3 / 批次 7 |
| M    | 事件 inventory 的全局与局部两部分合并          | `08-22-state-logic-port`、`08-22-views-checkin` | 批次 1       |
| O    | `command-manifest.json` 生成侧若变动需通知     | `08-22-workspace-cargo-upgrade`              | 批次 1       |
| —    | 人工验证项归属                          | `08-22-regression-release`                         | 批次 6       |
| —    | `path-mapping.md` 是批次 4 的路径依据   | `08-22-react-foundation`                           | 批次 4       |
