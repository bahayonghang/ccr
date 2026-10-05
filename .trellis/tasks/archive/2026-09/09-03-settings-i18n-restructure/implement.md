# Implement — 设置页重构与全局中文化

> 必读：`prd.md`、`design.md`、`research/settings-i18n-analysis.md`（父任务 research）、`.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md`、方向契约 surface brief。

## 执行清单

### Step 1 — stale memo 修复
- [ ] `features/configs/hooks/useAppSettings.ts:181-184`：去 memo 或 `useAppT()` + locale 依赖（优先去 memo）。
- [ ] 验证：手动/测试确认切换语言后分区选择列文案实时更新。

### Step 2 — zh-CN 全局中文化
- [ ] 按 design.md §1(b) 清单翻译 `i18n/locales/zh-CN.ts` 与 `i18n/bootMessages.ts` 两层（settings 域 + 全局 eyebrow + 零散标签；逐条评审技术名词去留）。
- [ ] 验证：`bun run test:i18n` 绿；zh 值扫描脚本/评审记录进报告。

### Step 3 — 死键清理 + 计数结算
- [ ] 删除 `dashboard.usage.peakLabel/hoverHint/metricSelectLabel/metricPlatforms`（双 locale + bootMessages 若有）+ 设置页 eyebrow key（随 Step 4 元素移除）。
- [ ] `EXPECTED_LEAF_COUNT` 两处同步结算（`scripts/check-i18n.mjs`、`tests/i18n.test.cjs`）。
- [ ] 验证：`bun run scripts/check-i18n.mjs` 绿。

### Step 4 — 设置页终端化重构
- [ ] `AppSettingsView.tsx` + `styles/app-settings.css` + `settings/*.tsx` 按 design.md §2：去 hero kicker、meta 改等宽状态读出、分区列改命令行列表、选择卡选中态收敛。
- [ ] 验证：dev:web 截图评审四组合之一（dark-neutral）+ 浅色抽查；`tests/configs/app-settings-view.smoke.test.tsx` 既有断言适配更新。

### Step 5 — 回归测试新增
- [ ] zh 值 CJK 断言（settings 域 + eyebrow 模式）；设置页 live 切换 smoke。
- [ ] 验证：`bunx vitest run --config vitest.smoke.config.ts tests/i18n/ tests/configs/` 绿。

### Step 6 — spec 沉淀 + 全量门禁
- [ ] `react-rerender-discipline.md` 增补 memo 条款；`frontend/index.md` 登记。
- [ ] `cd ccr-ui && bun run type-check && bun run lint && bun run test && bun run build`。
- 已知非阻塞：`tests/shell/route-view-mount.smoke.test.tsx` 2 个 pre-existing 失败（clean HEAD 复现），不要追。

## 回滚点

- Step 2/3 是 locale 数据改动，单独可 revert；Step 4 视觉重构单独可 revert。

## Review gates

- Step 1-3 完成后 i18n 门禁必须全绿才进 Step 4。
- 全部完成后对照父任务验收标准第 4 条（zh-CN 零英文残留、切换实时、29 处中文化、测试落地）。
