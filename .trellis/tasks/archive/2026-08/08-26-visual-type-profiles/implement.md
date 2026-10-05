# Profile 采用 — 执行

## 清单

1. 改 `ProfileFieldSlot` 与四平台 presentation 的 kind。
2. 卡片 / 表格按 kind 渲染；抽 `ProfileFieldValue` 若能去掉重复。
3. 页头、Off 横幅（按钮 + 容器 token）、空态、编辑器、overflow、**ProfilesHeader** 改 `Button`。
4. 删 `.cp-btn` / `.pe-btn`（禁止 alias）；tags、行状态徽章、`record.badges` 改 Badge static。
5. 更新 / 新增 `tests/profiles/` smoke。
6. 走查 8 组合，写 `notes.md`。
7. `just frontend-check-quick`。

## 验证

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profiles/
just frontend-check-quick
```

Web 预览（走查）：`cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`，打开 `/claude-code/profiles`。按 design.md 的 8 行表填写 `notes.md`；五格皆 PASS 才记该行 PASS。只写「已走查」不算完成。

## 风险文件

- `profilePresentation.ts`：kind 变更会破现有 `chip` 断言
- `profiles-shared.css`：删 `.cp-btn` 时确认没有残留 class

## 回滚

只触及 profiles 呈现层。不回滚 `src/ui` 原语。
