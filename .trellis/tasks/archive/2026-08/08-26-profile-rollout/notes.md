# 08-26-profile-rollout notes

## R4 外壳对比

| 维度 | `SurfacePage`（claude / codex） | `PageShell`（grok） |
| --- | --- | --- |
| loading | `AsyncStatePanel state="loading"`，隐藏 children | 页头仍在，内容区另挂 loading 面板 |
| 错误 | `AsyncStatePanel state="error"` + Retry | toast（`surfaceNotify.error`），不换整页 |
| runtime-unavailable | `AsyncStatePanel`，无 subnav | `PageShell` + `GrokSubnav` + 同款面板 |
| subnav | 无 | 有（`GrokSubnav` → `PlatformSubnav module="grok"`） |

差异不只是 subnav：Grok loading 是叠加，Claude/Codex 是互斥面板。本任务给 `SurfacePage` 增加可选 `subnav`，三平台都走 `SurfacePage`；Grok 的 `localOnly` 映射 `runtime-unavailable`；loading 统一为互斥面板。错误仍由控制器 toast，不把 Grok 改成整页 error。

## `can_off`

两个 list 返回都已有该字段，不新增后端字段。

| 平台 | 字段 | 取值 |
| --- | --- | --- |
| Claude | `ClaudeProfilesResponse.can_off?: boolean`；domain `profilesFrom` 已 `can_off: source.can_off === true` | `payload.can_off === true` |
| Codex | `CodexProfilesResponse.can_off?: boolean` | `payload.can_off === true` |
| Grok | 无 `can_off`；用激活信封 | `activation !== 'inactive'` |

回退 `current !== null` 未使用：Claude/Codex 字段存在。缺省（`undefined`）按 `=== true` 为 false。

## Raw 命令

| 平台 | get | save | wrapper |
| --- | --- | --- | --- |
| Claude | `claude_get_profiles_raw` | `claude_save_profiles_raw` | `src/api/domains/claude.ts` `getClaudeProfilesRaw` / `saveClaudeProfilesRaw` |
| Codex | `codex_get_profiles_raw` | `codex_save_profiles_raw` | `src/api/domains/codex.ts` `getCodexProfilesRaw` / `saveCodexProfilesRaw` |
| Grok | 无 | 无 | 不传 `rawSource` |

`refreshAll`：`queryClient.invalidateQueries({ queryKey: ['platform-profiles', cacheKey] })`。

## Grok 能力承接（改造前位置 → 改造后）

| 能力 | 现位置 | 承接 | 验证 |
| --- | --- | --- | --- |
| `profile_kind` 展示 | `GrokProfileCard` 文案 | `grokProfilePresentation.project` → `badges` | wiring + 卡片/表格 |
| 启用 / 停用 | `GrokProfileCard` 溢出菜单 → `handleToggle` | 统一 `ProfileOverflowMenu` → `onToggle` | 三平台溢出菜单 |
| recovery 提示条 | `GrokProfilesPage` 内联条 | `ProfilesSurface.notice` | `data-testid=profiles-notice` |
| delete `active\|drifted` 单次 force | `deleteProfile` | 原样 | `tests/grok-profiles-view.smoke.test.ts` |
| delete `unsafe_missing_entry_state` | 原先对所有 blocked 都 offer force | **补 reason 短路**：unsafe 不 confirm force（AC5；active/drifted 判定不变） | 同上 |
| blocked force 不循环 | `if (force) throw` | 原样 | 同上 |
| Local-only fail-closed 与 pin 保留 | `probeQuery` + `enabled: !localOnly` | 原样；pin store 不清理 | 同上 |
| activation drifted Off 文案 | `handleOff` | 原样 | 走查 |
| official / third-party | `GrokProfileEditorModal` | `grokProfileEditorAdapter.visible` | `grok-profile-editor.smoke.test.ts` |

## antigravity vs gemini

层二 key 为 `antigravity`（`profilesConfigs` / `profilePresentations`）。descriptor id 仍是 `gemini`，`rootPath` `/antigravity`。本任务不改 `platformDescriptors.ts`，不加路由。命名不一致上报，不在本任务“修正”。

## BaseProfiles 处置

接线后 Claude/Codex 不再消费 `BaseProfiles`。删除该文件，而不是薄封装：它仍渲染 `ProfileListRow` + `toProfile()` 七字段快照，会把 `profile_kind` / 高级字段丢掉；薄封装会把已删除的列表路径留在平台层。

## 删除清单执行条件

接线提交且 `grok-profiles-view` / wiring smoke 通过后再删：`GrokProfilesPage.tsx`、`GrokProfileCard.tsx`、`GrokProfileEditorModal.tsx`、`ProfileListRow.tsx`、`BaseProfiles.tsx`。保留 `useGrokProfilesPage.ts`、`grokEditorValidation.ts`、QuickRail / Palette / Inspector / Diff / Raw。

`checkin-oauth-wizard.smoke.test.tsx` 对 `GrokProfileCard` 的渲染改为 `ProfileCardGrid`。

## 前序待决项（AC14）

### registry-tokens

- 明色平台色：三角色已按 12%/32% 混合并对 `-surface` ≥ 4.5:1；本任务不改 token。
- `--color-platform-*` 消费点：registry notes 表已确认跟随新 dot；本任务 profiles 页头 glyph 走 `var(--color-platform-${key}-*)`。
- Claude slot3：`provider`（填充率 93.1%，避免 account PII）。
- token 治理：R10 已写入冻结段，不另立任务。

### list-surface

- 共享原子：`cp-btn` / `cp-chip` / `cp-pill` / `cp-label` / `cp-input` 已在 `profiles-shared.css`。
- 在线旧类名：删除 `ProfileListRow` 后 `cp-row*` 仅剩 Header/Section 等仍用者；行类随 ListRow 删除。
- 搜索热键：仓库绑定 `/`，不抄 ⌘K。
- Inspector 两列：`cp-card-grid--inspector` 已有。
- 旧 Claude/Codex 页：接线后被 `ProfilesSurface` 取代，不再走查 BaseProfiles。

### editor

- Codex `auth_token` 缺席保留原值：editor notes 已对照 `apply_profile_config`。
- Grok dirty：`useProfileEditor.setField` 一律记 dirty，与 RHF 同值回写仍 dirty 等价；不在 adapter 内保留 RHF。
- Claude 高级区：adapter `advanced: true` 分段，外壳折叠。
- `profile-editor-shell.css` 的 `pe-*` 留在该文件（`ProfileEditorModal` 导入）；没有可合并的重复 `cp-*`。共享按钮/chip 已在 `profiles-shared.css`。

## 导出与 sentinel

列表导出走 `exportClaudeProfiles(false)` / `exportCodexProfiles(false)`，再 `downloadTextFile`。Grok 摘要 JSON 本身不含密钥字段。

## 确认对话框

Grok 控制器原样在 hook 内 `surfaceNotify.confirm`。Claude/Codex 与其对齐（破坏性操作为 `danger`，apply/off/toggle 为 `warning`/`info`）。

## 删除结果

已删除：`GrokProfilesPage.tsx`（接线后只剩 re-export）、`GrokProfileCard.tsx`、`GrokProfileEditorModal.tsx`、`ProfileListRow.tsx`、`BaseProfiles.tsx`、`profiles-model.ts`。

保留：`useGrokProfilesPage.ts`、`grokEditorValidation.ts`、QuickRail / Palette / Inspector / Diff / Raw。

`checkin-oauth-wizard.smoke.test.tsx` 的 `GrokProfileCard` 渲染改为 `ProfileCardGrid`。

产品提交：`231102cb` 接线；`9a4790cc` 删除；`abad81b3` 表格名称列省略号。

## 零消费清单（AC11 / 父 AC21）

对 `components/profiles/index.ts` 每个导出，排除 barrel（`index.ts` / `shared.ts`）后的非定义消费方：

| 导出 | 非 barrel 消费方 |
| --- | --- |
| ProfileDiffRows | `ProfilesInspectorPreview.tsx` |
| ProfileCardGrid | `ProfilesSurfaceRecords.tsx` |
| ProfileTable | `ProfilesSurfaceRecords.tsx` |
| ProfilesCommandPalette | `ProfilesSurface.tsx` |
| ProfilesEmptyState | `ProfilesSurfaceRecords.tsx` |
| ProfilesNotice | `GrokProfilesScreen.tsx` |
| ProfilesInspector | `ProfilesSurfaceRecords.tsx` |
| ProfilesOffBanner | `ProfilesSurface.tsx` |
| ProfilesPageHeader | `ProfilesSurface.tsx` |
| ProfilesQuickRail | `ProfilesSurface.tsx` |
| ProfilesRawEditorPanel | `ProfilesSurface.tsx` |
| ProfilesStatStrip | `ProfilesSurface.tsx` |
| ProfilesToolbar | `ProfilesSurface.tsx` |
| ProfileEditorFields | `ProfileEditorModal.tsx` |
| ProfileEditorModal | `ClaudeProfilesScreen` / `CodexProfilesScreen` / `GrokProfilesScreen` |

**上报（不在本任务删除）**：`ProfilesHeader`、`ProfilesSection` 在退役 `GrokProfilesPage` / `BaseProfiles` 后只剩测试与 CSS。约束要求不得静默删除遗留组件，保留并上报为独立产品决策。

`ProfileOverflowMenu` 未进 barrel，由 Card/Table 内部使用。

## `rg -l "smoke.test" ccr-ui/src`

无 `*.smoke.test.*` 文件。命中为既有注释：`shellPreferences.ts`、`eventBridge.ts`、`types/checkin.ts`。

## 硬编码 hex（AC18 / 父 AC27）

```
rg -n "#[0-9a-fA-F]{3,8}" ccr-ui/src/components/profiles ccr-ui/src/features/platform/profiles ccr-ui/src/features/grok/profiles
```

结果为空。

## Sentinel 六处（AC17 / 父 AC5）

`SENTINEL = sentinel-auth-token-9f3c2a1b`，`tests/profiles-platform-wiring.smoke.test.tsx` + adapter / editor-shell：

1. `ProfileDisplayRecord`：`JSON.stringify(projected)` 不含 sentinel
2. DOM：`document.body.textContent` 不含
3. `console`：log/info/warn/error spy 不含
4. toast/错误：editor-shell 错误文案与 data 属性不含；adapter 提交错误路径同样剥离
5. 导出：`exportClaudeProfiles(false)`（脱敏）
6. 编辑器提交：`profile-editor-adapters.smoke.test.ts` 编辑时空密钥省略 `auth_token` / `api_key`

## 24 次走查（AC15 / 父 AC25）

条件：zoom 100%；`data-accent=clay`；夹具 `tests/fixtures/profiles.ts` 每平台 7 条。Web 无 Tauri invoke，故用同一套 `ProfilesSurface` + 平台 presentation + 夹具挂载（非桌面壳路由）。截图 `ccr-ui/tests/__screenshots__/profiles-{platform}-{theme}-{flavor}-{w}x{h}.png`，24 张，不进 CI。

对照 `research/design-source.md` 结构：页头（glyph + 路径 + 新建/导出）✓；统计四卡 ✓；筛选栏（搜索 / tag pill / 视图）✓；列表卡片或表格 ✓；Off 横幅 ✓；Claude/Codex 有源文件入口、Grok 无 ✓。面包屑由 `SurfacePage`/`PageHeader` 承担，不是设计稿独立 60px 栏。搜索热键仓库为 `/`，不抄 ⌘K。

`abad81b3` 后表格名称列 `scrollWidth === clientWidth === 216`，120 字描述省略号，不再盖住相邻列。

## 900×800 表格滚动（AC16 / 父 AC26）

测量点：`[data-testid=profiles-table-scroll]` 与 `document.body`。四主题组合数值相同：

| 平台 | table scrollWidth | table clientWidth | body scrollWidth | body clientWidth |
| --- | --- | --- | --- | --- |
| claude | 1034 | 900 | 900 | 900 |
| codex | 1034 | 900 | 900 | 900 |
| grok | 1034 | 900 | 900 | 900 |

判据：`table.scrollWidth > table.clientWidth` 且 `body.scrollWidth <= body.clientWidth`。

## 本任务 AC

- [x] AC1 三平台 `ProfilesSurface`；`*ProfilesView.tsx` 薄壳
- [x] AC2 同一 `ProfileEditorModal`
- [x] AC3 Grok `profile_kind` / toggle / notice
- [x] AC4 `grok-profiles-view.smoke.test.tsx`
- [x] AC5 unsafe 不提供 force
- [x] AC6 `canOff` 与 Off 横幅
- [x] AC7 raw-source；Grok 无入口
- [x] AC8 外壳结论见上文 R4
- [x] AC9 旧文件已删；控制器保留
- [x] AC10 BaseProfiles 删除
- [x] AC11 零消费清单；Header/Section 上报
- [x] AC12 导出与 CSS 级联；i18n 4327
- [x] AC13 antigravity 注册表渲染
- [x] AC14 前序待决项
- [x] AC15 24 走查
- [x] AC16 滚动实测
- [x] AC17 sentinel 六处
- [x] AC18 hex 为空
- [x] AC19 wiring 测试；src 无 smoke 测试文件
- [x] AC20 `just frontend-check-quick` 通过（141 files / 651 tests）；`just ui-check` 通过（`ccr-desktop` cargo check + 同上前端门）
- [x] AC21 父任务 28 条见下

## 父任务 AC（prd.md 1–28）

- [x] AC1–AC4 呈现层 / unify 测试 / project 往返 / Grok kind（前序 + wiring）
- [x] AC5 sentinel 六处
- [x] AC6–AC8 token（registry-tokens）
- [x] AC9–AC13 统计 / 筛选 / 空态 / 双视图 / 持久化（list-surface + 走查）
- [x] AC14–AC17 编辑器外壳与 adapter（editor）
- [x] AC18 Grok 写入分支（grok-profiles-view）
- [x] AC19 raw-source
- [x] AC20 vendor key 等价类（registry-tokens）
- [x] AC21 零消费（Header/Section 上报保留）
- [x] AC22 Grok 旧呈现已删
- [x] AC23 antigravity 层二
- [x] AC24 测试均在 `ccr-ui/tests/*.smoke.test.ts(x)`；门禁见 AC20
- [x] AC25–AC26 走查与滚动表
- [x] AC27 hex
- [x] AC28 i18n 双 locale，叶数 4327

