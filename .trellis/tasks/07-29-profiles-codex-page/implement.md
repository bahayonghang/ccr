# Implement: Codex Profiles 页面落地

前置：`07-29-profiles-shared-layer` 已归档或其组件已可用。每步后跑 `cd ccr-ui && bun run test`，收尾跑 `just ui-check`。

## 检查清单

1. [x] `utils/codexProfileEditor.ts`：`buildCodexProfileRequest` 内聚派生字段（`requires_openai_auth` / `openai_login_method` 由 `auth_mode` 计算；`env_key` 仅 `provider_env_key` 模式序列化，其余置 null）；表单删除 `requires_openai_auth` / `openai_login_method` 两个字段与视图侧 `syncDerivedAuthFields`；补 `env_key` 模式切换回归测试。
   - 展示侧统一函数落在**新建的 `utils/codexProfiles.ts`**（`resolveCodexBaseUrl` / `codexAuthModeLabel` / row·inspector·diff 描述符），而非 design 里写的 `codexProfileEditor.ts`——与已落地的 `utils/claudeProfiles.ts` 对称，编辑器 util 只留表单序列化职责。
2. [x] `CodexProfileEditorModal.vue`：样式迁移共享编辑器基底（删除整个 `<style>` 块：`--editor-*` 令牌体系 / `!important` / 硬编码 light+dark RGBA / 独立暗色覆盖块），改 import `profile-editor-shell.css`；新增保存前校验汇总条 + 跳转第一个错误分段；`resolvedModel` 提为 prop 供校验用。
3. [x] `ProfileCard.vue`：重写为与 `ClaudeProfileRow` 同结构（状态点 + 名称 + 状态徽章 + provider + Apply/env-export/··· 菜单 + 字段 dl + tags）；px 字号全部转 rem；铺开的 `<pre>` env-export 面板收敛为操作区复制图标按钮；引用 `utils/codexProfiles` 的统一 fallback/label。
4. [x] 视图接入新 QuickRail + `useProfilesQuickSwitch('codex')` + 稳定编号 hotkeys（`getStableTargets`）+ apply 成功 `recordUse` + rename 成功 `renamePinned`。
5. [x] 视图接入四槽 StatStrip（特色槽 Config mode，第四槽换可点击 Health）。
6. [x] 视图接入新 Toolbar（`compactFilters`，标签 + 排序入 Filters 弹层）；补 stale tag filter watch。
7. [x] 视图接入 ProfilesInspector（hovered/focused 双状态预览 + Health `@locate` 滚动高亮 + tag-select 写筛选 + 会话写入时间）。
8. [x] Apply 确认框接 `ProfileDiffRows`（base_url 经 official fallback 解析）；Delete 确认框加真实备份路径 footnote。
9. [x] 补内联 loadError + refreshError + retry 状态块（对齐 Claude）。
10. [x] Header 收敛（Add / ⌘K / ··· 溢出：Reload/Export/Edit TOML），修饰键跟随 `getClientPlatform()`。
11. [x] 死代码清理：`commandPalette.actionImport` i18n 键、`statStrip.lastWrite*` 键、内联 `ProfilesSection` 函数式组件、视图内 `.cp-section*` 重复样式、`extractErrorMessage` 换共享 `getErrorMessage`、`translateWithFallback` 硬编码中文回退（`deleteConfirm` / `confirmApply` / `contextRail.issues.deprecatedAuth`）。
12. [x] i18n 对称更新（zh-CN / en-US / 两份 `.keys.txt`）：新增 `loadFailedTitle` / `refreshFailedTitle` / `refreshFailedHint` / `retry` / `validationJump` / `statStrip.configModeHint`，删除 3 个失效键。
13. [ ] 行为回归走查 + 暗/亮 × 2543px/1280px 四张走查图。

> 13 未完成，原因与 `07-29-profiles-claude-page` 一致：需要 Tauri 桌面运行态 + 真实 profile fixture，本会话没有该运行环境（web 预览拿不到 Tauri IPC 数据，页面只会停在 loadError 态）。已用类型检查、ESLint/stylelint、548 条 smoke 用例（含本页新增 9 条 DOM 级断言）与生产构建替代覆盖编译与行为面。

## 共享层顺带修复

- `profile-editor-shell.css` 补 `.pe-input--mono` / `.pe-select--mono` 规则：Claude 编辑器已在传 `pe-input--mono` 但基底里没有对应声明，等宽字体一直没生效。
- `formatClaudeBaseUrlDisplay` 提升为平台无关的 `utils/text.ts#formatBaseUrlDisplay`（函数体零改动），两页共用同一套「完整 host + 截断 path」策略；`claudeProfiles.ts` 与 `ClaudeProfileRow.vue` 同步换引用。

## 规模变化

- `CodexProfilesView.vue` 1142 → 1267 行（模板 + 逻辑 ≈ 940 行，样式 ≈ 330 行）。净增来自本次补齐的 Inspector 预览、diff 确认、错误三态、快速切换持久化——这些能力在 Claude 页是从 1971 行里瘦身出来的，Codex 页此前根本没有。
- `CodexProfileEditorModal.vue` 1021 → 838 行（删掉 247 行 `--editor-*` 样式块，加回校验汇总条）。

## 验证

- `cd ccr-ui && bun run type-check` ✅
- `cd ccr-ui && bun run lint` ✅（0 error；72 warning 全部落在共享层既有文件，本次新增/改动文件 0 warning）
- `cd ccr-ui && bun run test` ✅ 111 files / 548 tests
- `cd ccr-ui && bun run build` ✅
- `just ui-check-frontend` ✅
- `just version-check` / `just fmt-check` ❌ —— 失败项是会话开始前就存在的脏文件（`Cargo.toml` / `ccr-ui/package.json` / `ccr-ui/src-tauri/tauri.conf.json` / `ccr-vscode/package.json` 的版本与 JSON 格式），与本任务改动无关。

## 遗留给父任务集成步骤

- `ProfilesContextRail.vue` 与 `codex.profiles.contextRail` i18n 子树在本次改动后已完全无引用（Claude 页先前已切走），按 shared-layer design 的约定统一在父任务集成步骤 4 删除。
- 两页 `<style>` 块中 `.cp-state*` / `.cp-grid` / `.cp-list-head` 约 300 行完全重复，可在集成步骤抽成共享 CSS。

## 回滚点

- 步骤 1 的派生字段单源化独立可回滚（行为等价重构 + `env_key` 契约修正）。
- 步骤 2 编辑器样式迁移独立可回滚；步骤 4–8 视图接入按组件逐个可回滚。
