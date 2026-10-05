# 操作页迁移 — 执行

## 清单

按 `prd.md` 文件列表勾选，不要改成「该域所有按钮」。

1. Codex §A：17 个消费方改为 `buttonClass` / `Button`，再删 `ui-classes.ts` 按钮导出。
2. OpenCode §A：5 个消费方，同样。
3. Grok §A：`GrokView.tsx`，同样。
4. Platform Base §B：`BaseMcp`、`BaseSettings`、`BaseAgents`、`BaseCommands`、`BasePlugins`。
5. §C：`AgentEditModal`（保存 primary / Add tool secondary / 取消 ghost）、`McpPresetsPanel`（确认 primary / 取消 ghost）。
6. Configs `.add-btn`（及可安全迁移的 edit/switch，保留 opacity）。
7. Checkin CTA：仪表盘 `.action-btn`、Providers、AccountFormModal、OAuthWizardModal **与** OAuthWizardBody。不动 `.nav-btn`。
8. SyncAccountDialog、SyncView hero（primary/ghost/warning）。
9. Usage pricing、PlatformUsageInsightPanel Link、SkillsMigration。
10. 更新选择器依赖旧 class 的 smoke。
11. `rg` 确认四个按钮导出已空，且 BaseCommands/BasePlugins 无旧 `bg-accent-primary px-4 py-2`。
12. `just frontend-check-quick`。

## 验证

```bash
rg "export const primaryBtnClass|ghostBtnClass|secondaryBtnClass|dangerBtnClass" ccr-ui/src/features
rg "bg-accent-primary px-4 py-2" ccr-ui/src/features/platform
just frontend-check-quick
```

第二条应对 §B 五个 Base 为空（AgentEditModal 本就不是这条 class）。

## 回滚

不回滚 `src/ui` 原语。本提交只含业务调用点。
