# Implement — 设置坞（左下角）排版与样式优化

> 必读：`prd.md`、`design.md`、`research/settings-i18n-analysis.md` 第 4 节（父任务 research）、方向契约 surface brief。

## 执行清单

### Step 1 — 结构与样式
- [ ] `ccr-ui/src/shell/MainLayoutChrome.tsx:74-97`：按 design.md 重排为两行（标题+右对齐版本 / meta 状态串）。
- [ ] `ccr-ui/src/shell/shell.css:199-251`：active = overlay + 2px 琥珀左 tick；hover/focus 按 design；meta 单行 ellipsis + title；200px/480px 两端不凌乱。
- [ ] 验证：dev:web 截图（窄/宽/active 三态）。

### Step 2 — flavor 描述文案修正
- [ ] `settings.appearance` 的 flavor 描述（zh+en，bootMessages 含则同步）：中性=暖纸灰表述，与新 token 一致。
- [ ] 验证：`bun run test:i18n && bun run scripts/check-i18n.mjs` 绿（计数不变，只改值）。

### Step 3 — 门禁
- [ ] `cd ccr-ui && bun run type-check && bun run lint && bun run test && bun run build`。
- [ ] 相关 smoke（`tests/shell/` 若有 settings-dock 断言）适配更新。
- 已知非阻塞：`tests/shell/route-view-mount.smoke.test.tsx` 2 个 pre-existing 失败，勿追。

## 回滚点

- 全部改动集中在两个文件 + locale 值，整体可 revert。

## Review gates

- 对照父任务验收标准第 5 条（200px 排版整齐、meta 全中文、三态正确）。
