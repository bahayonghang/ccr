# 三平台接线与集成验收 — 技术设计

## 平台控制器

三个平台各有一个控制器 hook，是呈现层与平台 DTO 之间的唯一转换点。

```ts
// 三平台控制器的共同输出形状（不是共享实现，各自写）
interface ProfilesControllerResult {
  records: readonly ProfileDisplayRecord[];
  current: string | null;
  canOff: boolean;
  loading: boolean;
  error: string | null;
  unavailable: boolean; // Local-only 环境不可用
  rawSource?: ProfileRawSourceCapability;
  onApply(name: string): Promise<void>;
  onDelete(name: string): Promise<void>;
  onToggle(name: string, enabled: boolean): Promise<void>;
  onOff(): Promise<void>;
  onExport?(): Promise<void>;
  onReload(): Promise<void>;
  /** 平台自有的提示条内容，如 Grok 的 recovery */
  notice?: { tone: "warning" | "danger"; message: string; actions?: ReactNode };
}
```

这是形状约定，不抽公共基类。三个平台的状态机不同，强行共享会把平台分支塞回共享层，违反 `platform-surface-contracts.md` 的无平台名分支规则。

### useClaudeProfilesPage / useCodexProfilesPage（新建）

```ts
useQuery({
  queryKey: ["platform-profiles", config.cacheKey],
  queryFn: () => listClaudeProfiles(), // 或 listCodexProfiles
  select: (payload) => ({
    records: payload.profiles
      .map((p) => stripCredentials(p, ["auth_token"]))
      .map((p) => presentation.project(p, { current: payload.current })),
    current: payload.current,
    canOff: payload.can_off ?? false,
  }),
});
```

`select` 中完成剥离与投影，剥离前的对象不进入 React state。

`canOff` 的来源：`listClaudeProfiles` / `listCodexProfiles` 的原始 payload。实施第一步确认该字段是否存在于两个命令的返回中；不存在时以 `current !== null` 作为回退条件并在 `notes.md` 记录字段缺口，不新增后端字段。

raw-source capability：

```ts
rawSource: {
  getRaw: getClaudeProfilesRaw,      // 或 codex 对应命令
  saveRaw: saveClaudeProfilesRaw,
  refreshAll: () => queryClient.invalidateQueries({ queryKey: ['platform-profiles', config.cacheKey] }),
}
```

Grok 不提供该字段。

### useGrokProfilesPage（改造）

保留全部现有内容：`readProfilesSnapshot` 的四态信封、`runProfileRecovery`、`handleSave` 的 recovery 分支、`deleteProfile` 的 blocked / force 单次重试、`handleToggle`、`handleOff` 的 drifted 文案分支、`localOnly` 的 fail-closed 守卫。

只增加：

1. 投影输出：`profiles.map(p => grokPresentation.project(p, { current: currentProfile }))`。`GrokProfileDto` 不含凭据，`stripCredentials` 传空数组，保留调用点以保持三平台一致。
2. props 组装：把现有 state 映射到 `ProfilesControllerResult` 的形状，`recovery` 映射到 `notice`。
3. 表单状态迁移：`useForm` + `GrokProfileEditorModal` 换为 `useProfileEditor` + `grokProfileEditorAdapter`。dirty 集合的来源从 `formState.dirtyFields` 换为 adapter 内部维护，语义须等价（见 `08-26-profile-editor` 的风险项）。

不改：删除分支的响应判定、recovery 的时序、`actionUnsupported` 的守卫、全部文案 key。

**能力承接检查表**（删除 `GrokProfilesPage.tsx` 前必须全部为真）：

| 能力                              | 承接位置                                          | 验证                                                     |
| --------------------------------- | ------------------------------------------------- | -------------------------------------------------------- |
| `profile_kind` 展示               | `project()` 的 `badges`                           | 卡片与表格上可见                                          |
| 启用 / 停用切换                   | 统一卡片溢出菜单 → `onToggle`                     | 三平台通用，基于 `ProfileDisplayRecord.enabled`           |
| recovery 提示条                   | `ProfilesControllerResult.notice`                 | `rename_apply_failed` / `rename_cleanup_failed` 两种各一次 |
| delete `active|drifted` 单次 force | `useGrokProfilesPage.deleteProfile` 原样           | `tests/grok-profiles-view.smoke.test.ts`                  |
| delete `unsafe_missing_entry_state` | 同上，不提供 force                               | 同上                                                      |
| blocked force 不循环              | 同上                                              | 同上                                                      |
| Local-only fail-closed 与 pin 保留 | 同上                                              | 同上                                                      |
| activation 信封（`drifted` 文案）  | `handleOff` 原样                                  | 手工走查                                                  |
| official / third-party 分支        | `grokProfileEditorAdapter.visible`                | `tests/grok-profile-editor.smoke.test.ts`                 |

## 接线后的视图形态

```tsx
// features/claude/ClaudeProfilesView.tsx
export function ClaudeProfilesView() {
  const ctrl = useClaudeProfilesPage();
  return (
    <ProfilesSurface
      presentation={claudeProfilePresentation}
      adapter={claudeProfileEditorAdapter}
      config={claudeProfilesConfig}
      {...ctrl}
    />
  );
}
```

`GrokProfilesView` 额外传 `shell={<PageShell subnav={<GrokSubnav />} />}` 或等价的 subnav props，由 R4 的外壳结论决定具体形态。

## 外壳统一（R4）

对比维度：

| 维度                | `SurfacePage`（claude / codex） | `PageShell`（grok） |
| ------------------- | ------------------------------- | ------------------- |
| loading 态          | ?                               | ?                   |
| 错误态              | ?                               | ?                   |
| runtime-unavailable | ?                               | ?                   |
| subnav 槽位         | 无                              | 有                  |

实施第一步读两个组件填表。差异仅为 subnav 时统一到一种外壳并把 subnav 作为 `ProfilesSurface` 的可选 props；差异涉及态语义时保留两种外壳的选择权，由 props 决定。结论与填好的表写入 `notes.md`。

## 删除清单与前置条件

只删除被统一呈现层直接取代的文件。父任务决策 5 下，命令面板、快捷栏、Inspector、Raw Editor 全部接入而非删除。

| 目标                                                | 前置条件                                                        |
| --------------------------------------------------- | ---------------------------------------------------------------- |
| `features/grok/profiles/GrokProfilesPage.tsx`       | `GrokProfilesView` 已切到 `ProfilesSurface` 且能力承接表全为真   |
| `features/grok/profiles/GrokProfileCard.tsx`        | 同上                                                             |
| `features/grok/profiles/GrokProfileEditorModal.tsx` | Grok 新建 / 编辑已走统一模态并验证通过                           |
| `components/profiles/ProfileListRow.tsx`            | `ProfileTable` 已在三平台生效                                    |
| `features/platform/profiles/BaseProfiles.tsx`       | claude 与 codex 已切到 `ProfilesSurface`；有外部消费方时改薄封装 |

**不删除**：`useGrokProfilesPage.ts`、`grokEditorValidation.ts`、`ProfilesQuickRail.tsx`、`ProfilesCommandPalette.tsx`、`ProfilesInspector*.tsx`、`ProfileDiffRows.tsx`、`ProfilesRawEditorPanel.tsx`、`utils/{platform}Profiles.ts`、`utils/{platform}ProfileEditor.ts`。

删除后级联清理：`components/profiles/index.ts`、`features/platform/profiles/shared.ts`、`features/platform/index.ts` 的导出，以及被删组件独有的 CSS 类与 i18n key。

## antigravity 层二注册（R9）

新增两条注册项：

- `configs/profiles.ts` 的 `profilesConfigs` 增加 `antigravity` 键（`list` 返回空快照即可，不接后端命令）。
- `configs/profilePresentation.ts` 的注册表增加 `antigravity` 键（由 `08-26-profile-registry-tokens` 已建立）。

测试从注册表按 key 取出后渲染 `ProfilesSurface`，断言页壳、四卡统计、筛选栏、两个视图、空态均可达。

不改 `config/platformDescriptors.ts`：给 `surfaces` 加 `'profiles'` 会改变导航与 `flattenCatalog()` 的 75 条路径，`tests/platform-surface-unify.smoke.test.ts` 会失败。descriptor 改动与该平台是否上线是同一个决策，留给后续任务。

命名不一致（设计稿 antigravity vs 代码 descriptor id `gemini`）记入 `notes.md` 上报。

## 前序子任务待决项清结

逐条确认并记录结论：

- `registry-tokens`：明色平台色观感、`--color-platform-*` 消费点变更确认、Claude slot3 字段选择、token 名称治理登记方式。
- `list-surface`：共享原子类清单、在线旧类名保留范围、搜索热键提示文案、Inspector 展开时的两列偏差、旧 Claude / Codex 页面走查结论。
- `editor`：Codex 后端 `auth_token` 缺席语义、Grok dirty 集合与 `formState.dirtyFields` 的语义差异、Claude 高级区排布。

## 测试

- `tests/profiles-platform-wiring.smoke.test.tsx`：三平台控制器各渲染一次，断言 `canOff` 传递、raw-source 入口存在性、Grok `notice` 渲染、antigravity 从注册表取值渲染。
- 既有测试必须继续通过：`tests/grok-profiles-view.smoke.test.ts`、`tests/grok-profile-editor.smoke.test.ts`、`tests/profiles-quick-switch.smoke.test.ts`、`tests/profiles-hotkeys.smoke.test.ts`、`tests/profiles-quick-rail.smoke.test.ts`、`tests/platform-surface-unify.smoke.test.ts`、`tests/theme-switch.smoke.test.tsx`、`tests/token-single-point.smoke.test.tsx`。
- 零消费组件检查：对 `components/profiles/index.ts` 每个导出 `rg` 确认存在非 barrel 消费方。

focused 命令：

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profiles-platform-wiring.smoke.test.tsx tests/grok-profiles-view.smoke.test.ts tests/grok-profile-editor.smoke.test.ts tests/platform-surface-unify.smoke.test.ts
```

## 走查条件

按父任务 `design.md`「视觉与响应式验收条件」：viewport `1440×900` 与 `900×800`，zoom 100%，`light|dark` × `neutral|clay` 四组合，夹具 `ccr-ui/tests/fixtures/profiles.ts`，滚动判据为表格容器 `scrollWidth > clientWidth` 且 body 不横向滚动，截图落 `ccr-ui/tests/__screenshots__/`。

三平台 × 2 viewport × 4 主题组合 = 24 次走查。每次记录结论，不合格项写明现象与位置。

## 风险

- 删除步骤可能暴露前三个子任务未覆盖的依赖。发生时回到对应子任务修补，不在本任务内堆积临时兼容代码。
- Grok 的表单状态从 `react-hook-form` 迁到 `useProfileEditor` 是本任务风险最高的一步。若 dirty 语义无法等价，保留 `react-hook-form` 作为 `grokProfileEditorAdapter` 的内部实现，只换外壳，并在 `notes.md` 记录。
- `canOff` 字段可能不存在于 Claude / Codex 的 list 返回中。回退条件与字段缺口按 R1 处理，不新增后端字段。
- `just ui-check` 耗时较长，安排在接线与清理全部完成后跑，不在中途反复触发。
