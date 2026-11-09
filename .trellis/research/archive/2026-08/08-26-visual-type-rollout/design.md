# 操作页迁移 — 设计

## 策略

优先替换 class 字符串，其次才改 JSX 为 `<Button>`。`Link` 一律 `className={buttonClass({ variant: 'primary' })}`。

`ui-classes.ts` 删按钮导出后，该文件只留 input / panel / tone。若文件因此过瘦，仍保留文件，避免无意义的跨文件搬家。

逐文件清单以 `prd.md` Closed inventory §A–§D 为准，实施时按该列表勾选，禁止改写成「所有消费方」。

## 变体映射

| 旧 | 新 | 适用文件 |
| --- | --- | --- |
| `primaryBtnClass` | `primary` | §A 全部消费方 |
| `secondaryBtnClass` | `secondary` | §A |
| `ghostBtnClass` | `ghost` | §A |
| `dangerBtnClass` / Grok danger CTA | `danger` | §A |
| `bg-accent-primary px-4 py-2`（添加/保存） | `primary` | BaseMcp、BaseSettings、BaseAgents、BaseCommands、BasePlugins、SyncAccountDialog |
| 同文件 `border border-border-default` 取消/次按钮 | `ghost` | 上述 Base 与 AgentEditModal / McpPresets 取消 |
| `bg-accent-secondary` **保存**（AgentEditModal footer） | `primary` | 弹层唯一提交；旧色不是 `secondary` 变体 |
| `bg-accent-secondary` **Add tool** | `secondary` | 行内添加，不得升为 primary |
| `bg-accent-secondary` **确认安装** | `primary` | McpPresets 弹层唯一确认 |
| `.add-btn` / `.action-btn.primary` / `.checkin-providers__primary-button` / `.form-button--primary` / `.oauth-wizard__button--primary` / `.pricing-button--primary` / `.sync-hero-button--primary` | `primary` | §D |
| `.action-btn` 无 `.primary` / `.sync-hero-button--ghost` | `ghost` | Checkin 仪表盘非刷新；Sync 刷新 |
| `.sync-hero-button--warning` | `warning` | Sync 强制同步 |
| 卡片 hover 才出现的 Edit | `quiet`（保留原 opacity 包装 class） | Configs `edit-btn`，若迁 |

缺变体则停下来回父任务规划，不在 rollout 加第八个 variant。尤其不得为 `--color-accent-secondary` 新增变体。

## 冲突

不得修改 `components/profiles/**`。不得修改 `src/ui/button.tsx` 的 API。

## 测试

在 `tests/ui/` 或现有域 smoke 中：

- 三份 `ui-classes.ts` 源码不含 `primaryBtnClass` 等四个导出
- `BaseCommands.tsx` / `BasePlugins.tsx` 不含 `bg-accent-primary px-4 py-2`
- AgentEditModal：保存 primary、Add tool secondary、取消 ghost
- McpPresets：确认 primary、取消 ghost

不要为每个改动的按钮新写视觉回归套件。
