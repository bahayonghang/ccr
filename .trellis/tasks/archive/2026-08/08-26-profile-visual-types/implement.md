# 全局视觉类型 — 父任务执行

父任务不改产品代码。实施只发生在子任务 `in_progress` 期间。

## 顺序

```
1. 08-26-visual-type-primitives     （必须先完成）
2. 08-26-visual-type-profiles  ┐
   08-26-visual-type-rollout   ┘  （并行，均依赖 1）
```

并行约定：`profiles` 只改 `components/profiles/**`、`features/platform/profiles/**`、`configs/profilePresentation.ts` 与对应 tests。`rollout` 不碰这些路径。双方都改 `src/ui/` 时停下来，只允许 primitives 写原语。

## 跨子任务门禁

- 不新增 token 名。
- 不改路由、Tauri、凭据剥离、Profiles 骨架顺序、表格列数。
- 新文案双语言；`bun run check:i18n`。
- 新测试在 `ccr-ui/tests/ui/` 或 `ccr-ui/tests/profiles/`。`rg -l "smoke.test" ccr-ui/src` 为空。
- 规格条款若变，在同一子任务更新 `.trellis/spec/ccr-ui/frontend/`（预期：`layering-contracts.md` 示例路径大小写、`profiles-page-contracts.md` 的字段 `kind`、`theme-token-contracts.md` 仅当误加名时回滚）。
- AC 做不到就回规划，不在 rollout 降级。

## 验证

子任务收尾：

```bash
just frontend-check-quick
```

父任务集成（rollout 完成后）：

```bash
just ui-check
```

## 集成检查

- [ ] `@/ui` 导出四原语 + `buttonClass`
- [ ] Profile 三页页头 / Off 横幅（按钮 + 容器）/ 卡片字段与状态徽章对照 `research/visual-language.md`
- [ ] 8 行走查表全部 `result=PASS`（见 profiles `notes.md`），不是仅有记录文件
- [ ] 三份 `ui-classes.ts` 不再导出按钮 class
- [ ] unique-name 452 不变
- [ ] 父任务 AC1–AC12 勾选

## 回滚点

- primitives 提交可独立 revert
- profiles 与 rollout 互不改对方文件，可单独 revert
