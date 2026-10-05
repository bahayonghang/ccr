# 操作页按钮类迁移

父任务：`08-26-profile-visual-types`
依赖：`08-26-visual-type-primitives`
规格：`../08-26-profile-visual-types/research/visual-language.md`

## Goal

把下列逐文件清单里的动作按钮迁到 `Button` / `buttonClass()`，删掉三份重复的 `primaryBtnClass` 族。不重做各页布局，只统一动作外观。清单外的同类按钮（如 Sync 侧栏、WSL/SSH）明确不迁。

## Closed inventory

必须迁移的文件（不是「所有消费方」概括）。

### A. `ui-classes` 定义与 23 个消费方

定义（删除按钮导出，保留 input/panel/tone）：

- `features/codex/ui-classes.ts`
- `features/opencode/ui-classes.ts`
- `features/grok/ui-classes.ts`

Codex 消费方：`CodexView.tsx`、`CodexAuthView.tsx`、`CodexSessionsView.tsx`、`CodexSlashCommandsView.tsx`、`CodexSystemPromptsView.tsx`、`sessions/SessionDetailPanel.tsx`、`home/CodexHomePanels.tsx`、`auth/SaveCodexSessionModal.tsx`、`auth/RenameCodexAccountModal.tsx`、`auth/CodexAuthProvidersTab.tsx`、`auth/CodexAuthAccountsTab.tsx`、`auth/AuthOffBanners.tsx`、`auth/AddCodexAccountModal.tsx`、`auth/AddAccountTokenStep.tsx`、`auth/AddAccountOauthStep.tsx`、`auth/AddAccountLocalStep.tsx`、`auth/AddAccountApiStep.tsx`

OpenCode 消费方：`OpenCodeView.tsx`、`OpenCodeProvidersView.tsx`、`OpenCodePageShell.tsx`、`providers/OpenCodeProviderForm.tsx`、`providers/OpenCodeProviderCard.tsx`

Grok 消费方：`GrokView.tsx`

这些文件的 `primaryBtnClass` → `primary`，`secondaryBtnClass` → `secondary`，`ghostBtnClass` → `ghost`，`dangerBtnClass` 与 Grok danger CTA → `danger`。`<Link className={primaryBtnClass}>` 改 `buttonClass({ variant: 'primary' })`。

### B. Platform Base（含遗漏的 Commands/Plugins）

`bg-accent-primary px-4 py-2` 的添加/保存 → `primary`。同文件 `border border-border-default` 的取消或次按钮 → `ghost`。

- `features/platform/mcp/BaseMcp.tsx`（表单提交 `primary`；页头 newStdio/newHttp 描边 → `ghost`）
- `features/platform/settings/BaseSettings.tsx`（页头保存 → `primary`）
- `features/platform/agents/BaseAgents.tsx`（添加、表单保存 → `primary`；取消 → `ghost`）
- `features/platform/commands/BaseCommands.tsx`（添加、表单保存 → `primary`；取消 → `ghost`）
- `features/platform/plugins/BasePlugins.tsx`（添加、表单保存 → `primary`；取消 → `ghost`）

### C. `bg-accent-secondary` 弹层（不得按「primary 一次性 class」推断）

| 文件 | 动作 | 旧 class | 语义 | 新 variant |
| --- | --- | --- | --- | --- |
| `features/platform/agents/AgentEditModal.tsx` | 保存 | `bg-accent-secondary px-4 py-3` | 弹层唯一提交 | `primary` |
| 同上 | Add tool | `bg-accent-secondary px-6 py-3` | 行内添加，不是弹层主 CTA | `secondary` |
| 同上 | 取消 | `border border-border-default bg-bg-elevated` | 放弃 | `ghost` |
| `features/mcp/McpPresetsPanel.tsx` | 确认安装 | `bg-accent-secondary px-4 py-2` | 弹层唯一确认 | `primary` |
| 同上 | 取消 | `border border-border-default bg-bg-elevated` | 放弃 | `ghost` |

旧色是 `--color-accent-secondary` token，不是 Button 的 `secondary` 变体。保存/确认安装升为 `primary` 是有意统一，不是漏映射。

### D. 其它域 CTA

- `features/configs/components/ConfigFilters.tsx` + `styles/config-filters.css`：`.add-btn` → `primary`
- `features/checkin/CheckinAccountDashboardView.tsx` + `styles/dashboard.css`：`.action-btn.primary` → `primary`；无 `.primary` 的 `.action-btn` → `ghost`。`.nav-btn` 不迁。
- `features/checkin/tabs/CheckinProvidersTab.tsx` + `styles/providers.css`：`.checkin-providers__primary-button` → `primary`
- `features/checkin/components/AccountFormModal.tsx` + `styles/form.css`：`.checkin-accounts-tab__form-button--primary` → `primary`
- `features/checkin/components/OAuthWizardModal.tsx`、`OAuthWizardBody.tsx` + `styles/oauth.css`：`.oauth-wizard__button--primary` → `primary`
- `features/sync/SyncAccountDialog.tsx`：`bg-accent-primary px-4 py-2` 保存 → `primary`
- `features/sync/SyncView.tsx` + `styles/sync-view.css`：`--primary` → `primary`；`--ghost` → `ghost`；`--warning` → `warning`
- `features/usage/pricing/PricingView.tsx` + `styles/pricing-view.css`：`.pricing-button--primary` → `primary`
- `features/usage/platform/PlatformUsageInsightPanel.tsx` + `styles/platform-usage-insight-panel.css`：primary `Link` → `buttonClass({ variant: 'primary' })`，保持 `Link`
- `features/claude/SkillsMigrationView.tsx`：primary 动作 → `primary`，保留 `data-testid="skills-migration-primary"`

保留不迁：`fieldInputClass`、`panelCardClass`、tone 图标 class、窗口控件、托盘、分页、`PillToggleGroup`、`FilterChip`、Codex 账号卡 icon-only `ActionButton`、`.nav-btn`、`SyncInfoSidebar.tsx`、`WslManagementView.tsx`、`SshManagementView.tsx`。

## Requirements

- R1：三份 `ui-classes.ts` 不再定义按钮 class；§A 列出的 23 个消费方改 `Button` 或 `buttonClass`。
- R2：§B–§D 每个列出的动作按上表变体迁移，并删除只服务这些动作的一次性 Tailwind / 域 CSS 按钮规则。
- R3：`<Link>` 与 `<a>` 不用 `Button` 组件，只用 `buttonClass`。
- R4：不改各页信息架构、路由、表单校验与 testid（除非 testid 打在被替换的 class 上，则改选择器）。
- R5：测试：对 `ui-classes.ts` 做「不再导出 primaryBtnClass」的静态断言；按 §C 三行分别断言 AgentEditModal 保存/Add tool 与 McpPresets 确认的 variant；更新被 class 选择器绑死的 smoke。
- R6：`just frontend-check-quick` 通过。

## Acceptance Criteria

- [ ] AC1（R1）：`rg "export const primaryBtnClass|ghostBtnClass|secondaryBtnClass|dangerBtnClass" ccr-ui/src/features` 为空。
- [ ] AC2（R2）：§B 五个 Base 文件的添加/保存按钮为 `.ui-btn--primary`（或 `buttonClass` primary）。`BaseCommands` 与 `BasePlugins` 不得仍使用 `bg-accent-primary px-4 py-2`。
- [ ] AC2b（R2）：`AgentEditModal` 保存按钮 `.ui-btn--primary`，Add tool `.ui-btn--secondary`，取消 `.ui-btn--ghost`。不得三者同为 primary。
- [ ] AC2c（R2）：`McpPresetsPanel` 确认安装 `.ui-btn--primary`，取消 `.ui-btn--ghost`。
- [ ] AC3（R3）：`CodexSlashCommandsView` 与 `PlatformUsageInsightPanel` 的 Link 仍是 Link，class 来自 `buttonClass`。
- [ ] AC4（R4）：既有 testid 不删。
- [ ] AC5（R5、R6）：相关 smoke 与 `just frontend-check-quick` 通过。

## Out of scope

- Profile 呈现层（归 profiles 子任务）
- 输入框、卡片容器、筛选 pill
- 新页面或文案改写
- `SyncInfoSidebar.tsx`、`WslManagementView.tsx`、`SshManagementView.tsx`

## Notes

- Configs `edit-btn` / `switch-btn` 带 hover-reveal，迁 `quiet` 时必须保留 opacity 行为，否则算回归。
- Checkin `.nav-btn`（月份翻页）不是动作 CTA，可留。
