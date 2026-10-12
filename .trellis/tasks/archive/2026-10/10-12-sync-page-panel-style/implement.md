# 实施清单

按这个顺序改。每步保持同步命令的入参和禁用条件不动。

## Checklist

1. 在 `SyncView.tsx` 删除 `ScopeStrip` 和 `sync-console-intro`。把数量和敏感值提示放进 `SyncPageHeader` 的 status。连接芯片逻辑不改。
2. 分组头去掉 `sync-eyebrow` 和 `groupLabel`。标题、描述、数量留下。
3. `SyncAssetCard` 接受 `showTitle`。父层用 `asset.name !== group.title` 计算。去掉 `asset.description` 的段落。规范名仅在与 `asset.name` 不同时渲染。补上缺失行的状态发丝，或删除无样式的 `sync-asset-card--missing` 类并改用现有失败文案加发丝。
4. `SyncInfoSidebar` 去掉连接芯片。未配置时不要琥珀色主按钮。编辑改为 secondary。测试、断开保持次级。对话框开关函数不改。
5. 门控块改成中性横幅加 2px 状态 tick，主按钮仍调用现在的 `onConfigure` / `onRetest`。
6. 改 `sync-view.css` 与 `sync-output.css`：删掉页面 elevated 背景、透明边、`1rem` 圆角、胶囊页头按钮、眉题字距。面板使用 `var(--color-bg-surface)`、`var(--color-border-default)`、`var(--radius-xl)`。行分隔使用 `var(--color-border-subtle)`。输出卡的失败/部分成功不再把整张边框涂成状态色。
7. 从 `zh-CN.ts` 和 `en-US.ts` 成对删除不再引用的 `sync.assets` 与 `sync.assetGroups.*.label` key。
8. 更新 `tests/quality/worst-views.smoke.test.tsx` 里的 `SyncAssetCard` 调用，补上 `showTitle`。新增一条同步页结构冒烟：mock `listSyncAssets` 返回五份资产，断言没有「配置资产控制台」和范围条文案，CCR 标题只出现一次，Claude 行能看到 `settings.json` 与 `CLAUDE.md`。

## Validation

```
cd ccr-ui && bun run type-check
cd ccr-ui && bun run lint
cd ccr-ui && bun run test:i18n
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/sync tests/quality/worst-views.smoke.test.tsx
```

视觉：暗色桌面宽度、宽度小于 `64rem`、浅色，各看一次同步页。核对页头、三块分组、右栏、未配置横幅。有浏览器时走实际点击：刷新、打开账号对话框、确认 Sync 在未配置时禁用。

## Risky Files

- `ccr-ui/src/features/sync/SyncView.tsx`：删组件时不要把门控、输出锚点和口令弹层一起删掉。
- `ccr-ui/src/features/sync/SyncAssetCard.tsx`：保持 `memo` 和稳定回调。
- `ccr-ui/src/features/sync/SyncInfoSidebar.tsx`：断开确认框的文案和 `clearWebdavConfig` 调用保持原样。
- `ccr-ui/src/i18n/locales/zh-CN.ts`
- `ccr-ui/src/i18n/locales/en-US.ts`

不要改 `ccr-ui/src-tauri/src/commands/sync.rs`。

## Rollback

改动都在前端渲染和词条。还原上述文件即可，没有数据迁移。

## Before task.py start

本文件和 `design.md`、`prd.md` 已对齐。实现前再读 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`、`sync-security-contracts.md`、`react-rerender-discipline.md`。用户确认这份计划之后才能 `task.py start`。
