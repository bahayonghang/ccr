# 三平台接线与集成验收

父任务：`08-26-profile-design-language`
依赖：`08-26-profile-registry-tokens`、`08-26-profile-list-surface`、`08-26-profile-editor`

## Goal

为三个平台各建一个平台控制器，把统一呈现层与平台 adapter 接线起来，退役被取代的旧呈现文件，完成父任务的整体验收。

父任务决策 4：`useGrokProfilesPage` 保留为 Grok 平台控制器，不退役。决策 5：已规格化的能力（QuickRail、命令面板、Filters 的 provider 与排序维度、Inspector、Off 横幅）全部接入而非删除。因此本任务的删除面只覆盖被统一呈现层直接取代的文件。

## Requirements

- R1：Claude 与 Codex 平台控制器。新建 `useClaudeProfilesPage` / `useCodexProfilesPage`，各自负责 typed 列表 `useQuery`、`select` 中的 `stripCredentials` + `presentation.project`、`canOff` 取值、apply / off / delete、raw-source capability 组装。
- R2：Grok 平台控制器改造。`useGrokProfilesPage` 保留全部现有状态与分支，只增加 `stripCredentials` + `project` 的投影输出与 `ProfilesSurface` 所需的 props 组装。删除、改名 recovery、Local-only fail-closed、activation 信封、blocked force 不循环的行为与文案不变。
- R3：三平台视图接线。`ClaudeProfilesView` / `CodexProfilesView` / `GrokProfilesView` 各自退化为组装控制器输出 + `ProfilesSurface` + `ProfileEditorModal`。`GrokProfilesView` 额外带 `PageShell` + `GrokSubnav`。
- R4：外壳统一。比较 `SurfacePage`（claude / codex）与 `PageShell`（grok）在 loading / 错误 / runtime-unavailable 三态的语义差异。差异仅为 subnav 时统一到一种外壳并把 subnav 作为 `ProfilesSurface` 的可选 props；差异涉及态语义时保留两种外壳的选择权，结论写入 `notes.md`。
- R5：Grok 呈现层退役。`GrokProfilesPage.tsx`、`GrokProfileCard.tsx`、`GrokProfileEditorModal.tsx` 在能力全部承接并验证通过后删除。`useGrokProfilesPage.ts` 与 `grokEditorValidation.ts` 保留。
- R6：`ProfileListRow` 退役。`ProfileTable` 在三平台生效后删除。
- R7：`BaseProfiles` 退役。被 `ProfilesSurface` 取代后删除，或在有 profiles 之外的消费方时退化为薄封装。二选一，结论与理由写入 `notes.md`。
- R8：零消费清零。`components/profiles/` 下每个导出组件至少有一个非 barrel 消费方。
- R9：antigravity 层二注册。新增 `antigravityProfilesConfig` 与 `antigravityProfilePresentation` 的注册项，测试按 key 从注册表取出后渲染 `ProfilesSurface`。不改 `platformSurfaceDescriptors`，不加路由，不上线页面。
- R10：前序待决项清结。三个前序子任务 `notes.md` 中的全部待决项逐条给出结论。
- R11：整体验收。完成父任务 `prd.md` Acceptance Criteria 一节的全部 28 条。
- R12：测试落位。新增测试位于 `ccr-ui/tests/*.smoke.test.ts(x)`。

## Acceptance Criteria

- [ ] AC1（R1、R3）：三个平台页面渲染同一套 `ProfilesSurface`，`features/{claude,codex,grok}/*ProfilesView.tsx` 各自只做组装，无平台专属布局代码。
- [ ] AC2（R1、R3）：三个平台均可完成新建与编辑，且使用同一个 `ProfileEditorModal`。
- [ ] AC3（R2）：Grok 的 `profile_kind` 展示、启用/停用切换、recovery 提示条在统一页面中仍然可用。
- [ ] AC4（R2）：`tests/grok-profiles-view.smoke.test.ts` 全部用例通过——Local-only fail-closed 与 pin 保留、delete blocked / force 分支、force 不循环、rename recovery 时序、enabled/total 汇总五项。
- [ ] AC5（R2）：`unsafe_missing_entry_state` 不提供 force 入口，只显示人工恢复提示，测试断言。
- [ ] AC6（R1）：Claude 与 Codex 的 `canOff` 由控制器从 typed 读取结果取得并传给 Off 横幅；`canOff === false` 时横幅不渲染。
- [ ] AC7（R1、R5）：Claude 与 Codex 的 raw-source capability 接线完成，`tests/profiles-raw-source.smoke.test.tsx` 的四项在真实控制器下再跑一次通过；Grok 页面无该入口。
- [ ] AC8（R4）：外壳结论写入 `notes.md`，三平台在 loading、错误、runtime-unavailable 三态下的表现与结论一致。
- [ ] AC9（R5、R6、R7）：`GrokProfilesPage.tsx`、`GrokProfileCard.tsx`、`GrokProfileEditorModal.tsx`、`ProfileListRow.tsx` 已删除；`useGrokProfilesPage.ts` 与 `grokEditorValidation.ts` 仍存在且被消费。
- [ ] AC10（R7）：`BaseProfiles.tsx` 的处置结论（删除或薄封装）与理由写入 `notes.md`，且 `features/platform/index.ts` 的导出面无悬空引用。
- [ ] AC11（R8）：`rg` 验证 `components/profiles/index.ts` 每个导出至少有一个非 barrel 消费方，清单与消费方位置写入 `notes.md`。
- [ ] AC12（R5、R6）：删除后级联清理完成——`components/profiles/index.ts`、`features/platform/profiles/shared.ts`、`features/platform/index.ts` 的导出，以及被删组件独有的 CSS 类与 i18n key（`zh-CN` 与 `en-US` 同步），`bun run check:i18n` 通过。
- [ ] AC13（R9）：antigravity 注册项存在，测试按 key 从注册表取出渲染出完整页面；`tests/platform-surface-unify.smoke.test.ts` 的 75 路径断言仍通过。
- [ ] AC14（R10）：三个前序子任务的待决项逐条有结论，含明色平台色观感、`--color-platform-*` 消费点确认、Claude slot3 字段选择、共享原子类合并去重、Codex 后端 `auth_token` 缺席语义、Grok dirty 集合语义差异。
- [ ] AC15（R11）：三平台在 1440×900 与 900×800 两个 viewport、`light|dark` × `neutral|clay` 四种组合下按 `tests/fixtures/profiles.ts` 夹具走查通过，对照 `research/design-source.md` 的结构清单。
- [ ] AC16（R11）：900×800 下表格容器 `scrollWidth > clientWidth` 且 `document.body.scrollWidth <= document.body.clientWidth`，三平台实测值写入 `notes.md`。
- [ ] AC17（R11）：以随机 sentinel 作为 `auth_token` 走完整 list 流程，sentinel 不出现在展示记录、DOM 文本、`console`、toast/错误文案、导出内容、提交 payload 六处。
- [ ] AC18（R11）：Profile 相关全部文件 grep 硬编码 hex，结果为空。
- [ ] AC19（R12）：新增测试文件为 `tests/profiles-platform-wiring.smoke.test.tsx`；`rg -l "smoke.test" ccr-ui/src` 为空。
- [ ] AC20（R11）：`just frontend-check-quick` 通过；`just ui-check` 通过。
- [ ] AC21（R11）：父任务 `prd.md` Acceptance Criteria 一节的 28 条逐条勾选。

## Constraints

- 不改路由路径，不改 Tauri 命令签名，不改后端凭据序列化。
- 不改 `config/platformDescriptors.ts`。给 `surfaces` 增加 `'profiles'` 会改变导航与 `flattenCatalog()` 的 75 条路径。
- 删除操作需在功能接线完成并验证通过之后执行，不得先删后接。
- 不删除 `useGrokProfilesPage.ts`、`grokEditorValidation.ts`、`ProfilesQuickRail`、`ProfilesCommandPalette`、`ProfilesInspector` 系列、`ProfileDiffRows`、`ProfilesRawEditorPanel`、`utils/{platform}Profiles.ts`、`utils/{platform}ProfileEditor.ts`。这些在父任务决策 5 下全部接入。
- 若某遗留组件在接入后仍无消费方，不得静默删除：在 `notes.md` 记录，作为独立产品决策上报用户，本任务内保留。
- 验收口径不可在本任务内改写。AC 无法达成时回到规划修订。

## Notes

- 本任务是父任务唯一的集成与验收出口，前三个子任务不做跨平台验收。
- 删除清单的权威来源是父任务 `design.md` 的组件处置决策表。
- 命名不一致上报项：设计稿的 antigravity 与代码 descriptor id `gemini` 指同一平台，在本任务 `notes.md` 记录并上报，不在本任务解决。
- 本任务是审阅报告 TPR-05、TPR-08 的主要落地入口，并承接 TPR-06、TPR-14 的集成侧验收。
